#!/usr/bin/env bash
# Install a verified localjev-grep release archive on macOS or Linux.
#
# This script deliberately downloads an archive and its checksum before doing
# anything with the executable. It is intended to be downloaded or run from a
# reviewed checkout; do not use `curl ... | sh`.

set -euo pipefail
IFS=$'\n\t'
umask 077

readonly DEFAULT_REPOSITORY="xxvw/localjev-grep"

repository="${JGREP_REPOSITORY:-$DEFAULT_REPOSITORY}"
version="${JGREP_VERSION:-}"
install_dir="${JGREP_INSTALL_DIR:-${XDG_BIN_HOME:-}}"
asset_dir="${JGREP_ASSET_DIR:-}"
target="${JGREP_TARGET:-}"
force=0
temporary_directory=""
staged_binary=""

usage() {
    cat <<'EOF'
Usage: install.sh [OPTIONS]

Install a verified localjev-grep release archive for this macOS or Linux host.
Without --version, the script resolves the latest published GitHub Release.

Options:
  --version <TAG>       Release tag (for example v0.1.1 or 0.1.1).
  --install-dir <DIR>   Destination directory (default: $XDG_BIN_HOME or
                        $HOME/.local/bin).
  --asset-dir <DIR>     Read a release archive and checksum from DIR without
                        using the network. Requires --version.
  --target <TRIPLE>     Override platform detection for a supported Unix
                        release target. Intended for controlled environments.
  --force               Replace an existing jgrep executable in the target
                        directory.
  -h, --help            Show this help.

Environment equivalents: JGREP_VERSION, JGREP_INSTALL_DIR, JGREP_ASSET_DIR,
JGREP_REPOSITORY, and JGREP_TARGET.
EOF
}

die() {
    printf 'jgrep installer: %s\n' "$*" >&2
    exit 1
}

cleanup() {
    if [[ -n "${staged_binary:-}" && -e "$staged_binary" ]]; then
        rm -f -- "$staged_binary"
    fi
    if [[ -n "${temporary_directory:-}" && -d "$temporary_directory" ]]; then
        rm -rf -- "$temporary_directory"
    fi
}

trap cleanup EXIT HUP INT TERM

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

normalize_tag() {
    local candidate="$1"
    if [[ "$candidate" != v* ]]; then
        candidate="v${candidate}"
    fi
    [[ "$candidate" =~ ^v[0-9A-Za-z][0-9A-Za-z._-]*$ ]] || die "invalid release tag: $1"
    printf '%s\n' "$candidate"
}

resolve_latest_tag() {
    local redirected
    redirected="$(curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
        --output /dev/null --write-out '%{url_effective}' \
        "https://github.com/${repository}/releases/latest")" \
        || die "could not resolve the latest GitHub Release for ${repository}"
    redirected="${redirected%/}"
    normalize_tag "${redirected##*/}"
}

detect_target() {
    local operating_system architecture translated
    operating_system="$(uname -s)"
    architecture="$(uname -m)"

    case "$operating_system" in
        Darwin)
            # A Rosetta shell reports x86_64. Prefer the native Apple Silicon
            # release when the host tells us translation is active.
            translated="$(sysctl -in sysctl.proc_translated 2>/dev/null || true)"
            if [[ "$translated" == "1" ]]; then
                printf '%s\n' "aarch64-apple-darwin"
                return
            fi
            case "$architecture" in
                arm64|aarch64) printf '%s\n' "aarch64-apple-darwin" ;;
                x86_64|amd64) printf '%s\n' "x86_64-apple-darwin" ;;
                *) die "unsupported macOS architecture: $architecture" ;;
            esac
            ;;
        Linux)
            case "$architecture" in
                x86_64|amd64) printf '%s\n' "x86_64-unknown-linux-gnu" ;;
                *) die "no published Linux release supports architecture: $architecture" ;;
            esac
            ;;
        *) die "unsupported operating system: $operating_system" ;;
    esac
}

validate_target() {
    case "$1" in
        aarch64-apple-darwin|x86_64-apple-darwin|x86_64-unknown-linux-gnu) ;;
        *) die "unsupported Unix release target: $1" ;;
    esac
}

sha256_file() {
    local file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{ print $1 }'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{ print $1 }'
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$file" | awk '{ print $NF }'
    else
        die "need sha256sum, shasum, or openssl to verify the release archive"
    fi
}

read_expected_checksum() {
    local manifest="$1"
    local archive_name="$2"
    local expected

    expected="$(awk -v archive="$archive_name" '
        NF != 2 || length($1) != 64 || $1 !~ /^[[:xdigit:]]+$/ { bad = 1; next }
        $2 == archive { count += 1; hash = $1 }
        END {
            if (bad || count != 1) exit 1
            print hash
        }
    ' "$manifest")" || die "invalid SHA-256 manifest: $manifest"
    printf '%s\n' "$expected"
}

download() {
    local source_url="$1"
    local destination="$2"
    curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
        --retry 3 --connect-timeout 15 --output "$destination" "$source_url" \
        || die "could not download $source_url"
}

while (($# > 0)); do
    case "$1" in
        --version)
            (($# >= 2)) || die "--version requires a tag"
            version="$2"
            shift 2
            ;;
        --version=*)
            version="${1#*=}"
            shift
            ;;
        --install-dir|--bin-dir)
            (($# >= 2)) || die "$1 requires a directory"
            install_dir="$2"
            shift 2
            ;;
        --install-dir=*|--bin-dir=*)
            install_dir="${1#*=}"
            shift
            ;;
        --asset-dir)
            (($# >= 2)) || die "--asset-dir requires a directory"
            asset_dir="$2"
            shift 2
            ;;
        --asset-dir=*)
            asset_dir="${1#*=}"
            shift
            ;;
        --target)
            (($# >= 2)) || die "--target requires a target triple"
            target="$2"
            shift 2
            ;;
        --target=*)
            target="${1#*=}"
            shift
            ;;
        --force)
            force=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        *) die "unknown option: $1" ;;
    esac
done

(($# == 0)) || die "this installer does not accept positional arguments"
[[ "$repository" =~ ^[0-9A-Za-z._-]+/[0-9A-Za-z._-]+$ ]] \
    || die "invalid GitHub repository: $repository"

if [[ -z "$install_dir" ]]; then
    [[ -n "${HOME:-}" ]] || die "HOME is not set; pass --install-dir"
    install_dir="$HOME/.local/bin"
fi

if [[ -z "$target" ]]; then
    target="$(detect_target)"
fi
validate_target "$target"

if [[ -n "$asset_dir" ]]; then
    [[ -n "$version" && "$version" != "latest" ]] \
        || die "--asset-dir requires an explicit --version"
    [[ -d "$asset_dir" ]] || die "asset directory does not exist: $asset_dir"
    asset_dir="$(cd "$asset_dir" && pwd -P)"
    tag="$(normalize_tag "$version")"
else
    require_command curl
    if [[ -z "$version" || "$version" == "latest" ]]; then
        tag="$(resolve_latest_tag)"
    else
        tag="$(normalize_tag "$version")"
    fi
fi

require_command tar
require_command awk

package_root="localjev-grep-${tag}-${target}"
archive_name="${package_root}.tar.gz"
checksum_name="${archive_name}.sha256"
binary_member="${package_root}/jgrep"

temporary_directory="$(mktemp -d "${TMPDIR:-/tmp}/localjev-grep-install.XXXXXXXX")" \
    || die "could not create a temporary directory"
archive_path="${temporary_directory}/${archive_name}"
checksum_path="${temporary_directory}/${checksum_name}"

if [[ -n "$asset_dir" ]]; then
    [[ -f "${asset_dir}/${archive_name}" ]] \
        || die "release archive is missing from asset directory: ${archive_name}"
    cp -- "${asset_dir}/${archive_name}" "$archive_path" 2>/dev/null \
        || cp "${asset_dir}/${archive_name}" "$archive_path"
    if [[ -f "${asset_dir}/${checksum_name}" ]]; then
        cp -- "${asset_dir}/${checksum_name}" "$checksum_path" 2>/dev/null \
            || cp "${asset_dir}/${checksum_name}" "$checksum_path"
    elif [[ -f "${asset_dir}/SHA256SUMS" ]]; then
        cp -- "${asset_dir}/SHA256SUMS" "$checksum_path" 2>/dev/null \
            || cp "${asset_dir}/SHA256SUMS" "$checksum_path"
    else
        die "asset directory has neither ${checksum_name} nor SHA256SUMS"
    fi
else
    release_base="https://github.com/${repository}/releases/download/${tag}"
    download "${release_base}/${archive_name}" "$archive_path"
    download "${release_base}/${checksum_name}" "$checksum_path"
fi

expected_checksum="$(read_expected_checksum "$checksum_path" "$archive_name")"
actual_checksum="$(sha256_file "$archive_path")"
if [[ "$(printf '%s' "$expected_checksum" | tr '[:upper:]' '[:lower:]')" != \
      "$(printf '%s' "$actual_checksum" | tr '[:upper:]' '[:lower:]')" ]]; then
    die "SHA-256 verification failed for ${archive_name}"
fi

# Extract only the expected executable member to stdout. This avoids allowing a
# release archive to choose arbitrary destination paths during installation.
members_path="${temporary_directory}/members.txt"
tar -tzf "$archive_path" > "$members_path" || die "could not list ${archive_name}"
member_count=0
while IFS= read -r member; do
    if [[ "$member" == "$binary_member" ]]; then
        member_count=$((member_count + 1))
    fi
done < "$members_path"
((member_count == 1)) || die "release archive does not contain exactly one ${binary_member}"

mkdir -p -- "$install_dir" || die "could not create installation directory: $install_dir"
destination="${install_dir}/jgrep"
if [[ -L "$destination" ]]; then
    die "installation destination is a symbolic link: ${destination}"
fi
if [[ -e "$destination" ]]; then
    [[ -f "$destination" ]] || die "installation destination is not a file: ${destination}"
    ((force == 1)) || die "${destination} already exists; rerun with --force to replace it"
fi

staged_binary="$(mktemp "${install_dir}/.jgrep.XXXXXXXX")" \
    || die "could not stage jgrep in ${install_dir}"
tar -xOzf "$archive_path" "$binary_member" > "$staged_binary" \
    || die "could not extract jgrep from ${archive_name}"
[[ -s "$staged_binary" ]] || die "release archive contains an empty jgrep executable"
chmod 755 "$staged_binary"
expected_version="jgrep ${tag#v}"
reported_version="$("$staged_binary" --version 2>&1)" \
    || die "release archive contains a jgrep executable that could not run"
[[ "$reported_version" == "$expected_version" ]] \
    || die "release archive version mismatch: expected ${expected_version}, got ${reported_version}"
mv -f -- "$staged_binary" "$destination" || die "could not install jgrep to ${destination}"
staged_binary=""

"$destination" --version >/dev/null || die "installed jgrep could not run"
printf 'Installed jgrep %s at %s\n' "$tag" "$destination"
case ":${PATH}:" in
    *":${install_dir}:"*) ;;
    *) printf "Add it to your shell PATH if needed: export PATH=\"%s:\$PATH\"\\n" "$install_dir" ;;
esac
