# Installation and coding-agent integration

`jgrep` publishes native, versioned release archives for macOS Apple
Silicon, macOS Intel, Windows x64, and Linux x64. The installers select the
matching archive, verify its SHA-256 manifest before using its executable,
require its `jgrep --version` output to match the selected tag, and install
only that executable into a per-user directory. They do not download the Qwen
model; semantic mode retrieves the pinned model only when it is first needed.

Release archives are checksummed but `v0.1.1` is neither signed nor
notarized. A checksum downloaded from the same release protects against
transfer corruption and accidental asset mismatch; it is not an independent
publisher-identity proof. The commands below download a version-pinned
installer bundle, verify it before extraction, then run a local file. They do
not use `git clone`, `curl | sh`, or `Invoke-Expression`.

## One-paste, no-clone installation

The bundle has the common installer and a localized wrapper for each supported
README language. The example below uses English (`en`); replace `en` in the
last path with `ja`, `zh-CN`, `ko`, `es`, `de`, `ru`, `fr`, `pt-BR`, `it`,
`ar`, or `hi` to select its localized start message. The core's safety checks
and supported options are identical in every language.

### macOS and Linux

Paste this single compound command into Bash or zsh:

```sh
(
  set -e
  version=v0.1.1
  archive="localjev-grep-installers-${version}.tar.gz"
  workdir="$(mktemp -d)"
  trap 'rm -rf "$workdir"' EXIT
  base="https://github.com/xxvw/jgrep/releases/download/${version}"
  curl --fail --silent --show-error --location --proto '=https' \
    --proto-redir '=https' -o "$workdir/$archive" "$base/$archive"
  curl --fail --silent --show-error --location --proto '=https' \
    --proto-redir '=https' -o "$workdir/$archive.sha256" "$base/$archive.sha256"
  (cd "$workdir" && if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "$archive.sha256"
  else
    sha256sum -c "$archive.sha256"
  fi)
  tar -xzf "$workdir/$archive" -C "$workdir"
  bash "$workdir/localjev-grep-installers-${version}/installers/en/install.sh" \
    --version "$version"
)
```

### Windows PowerShell

Paste this single PowerShell command block:

```powershell
& {
  $ErrorActionPreference = 'Stop'
  $version = 'v0.1.1'
  $archive = "localjev-grep-installers-$version.zip"
  $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
  New-Item -ItemType Directory -Path $workdir | Out-Null
  try {
    $base = "https://github.com/xxvw/jgrep/releases/download/$version"
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile (Join-Path $workdir $archive)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile (Join-Path $workdir "$archive.sha256")
    $manifest = (Get-Content -LiteralPath (Join-Path $workdir "$archive.sha256") -Raw).Trim()
    $manifestPattern = '^[A-Fa-f0-9]{64}  ' + [regex]::Escape($archive) + '$'
    if ($manifest -notmatch $manifestPattern) { throw 'installer bundle checksum manifest is invalid' }
    $expected = $manifest.Substring(0, 64).ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath (Join-Path $workdir $archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'installer bundle checksum mismatch' }
    Expand-Archive -LiteralPath (Join-Path $workdir $archive) -DestinationPath $workdir -Force
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\en\install.ps1") -Version $version
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

The wrapper forwards every option to the verified core installer. To choose a
custom directory, add `--install-dir ~/bin` to the final `bash` invocation
inside the macOS/Linux block, or add `-InstallDir C:\bin` to the final
PowerShell invocation. Each platform installer then downloads and verifies
only the matching native binary archive.

## Reviewed checkout and offline assets

### macOS and Linux from a reviewed checkout

From a reviewed checkout, run:

```sh
git clone https://github.com/xxvw/jgrep.git
cd jgrep
./scripts/install.sh
```

With no version option, the script resolves the latest published GitHub
Release. Pin a release for reproducible installations:

```sh
./scripts/install.sh --version v0.1.1
```

The script selects one of these native release targets:

| Host | Target | Archive |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| Linux x64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |

It installs `jgrep` into `$XDG_BIN_HOME` when that variable is set, otherwise
`$HOME/.local/bin`. The installer does not edit shell startup files. If that
directory is not already on `PATH`, add it in your shell configuration:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Pass `--install-dir <directory>` (or the `JGREP_INSTALL_DIR` environment
variable) to choose another directory. The installer refuses to replace an
existing executable unless `--force` is supplied. `--bin-dir` is an alias for
`--install-dir`.

For a local release build, air-gapped validation, or an internal mirror that
preserves release filenames and checksum files, use `--asset-dir`. This makes
no HTTP request and requires an explicit version:

```sh
./scripts/install.sh --asset-dir ./dist --version v0.1.1
```

The asset directory must contain the target-qualified `.tar.gz` archive and
either its `.tar.gz.sha256` file or `SHA256SUMS`. `JGREP_VERSION`,
`JGREP_ASSET_DIR`, `JGREP_TARGET`, and `JGREP_REPOSITORY` provide corresponding
environment overrides. `--target` is intended for controlled build or test
environments; it does not make an incompatible binary executable on another
CPU or operating system.

The Linux x64 archive requires glibc 2.35 or newer. Build from source on an
older Linux system or an unsupported architecture. macOS releases are not
notarized; follow your organization’s normal review and platform policy rather
than disabling platform security controls in an installer.

## Windows PowerShell from a reviewed checkout

From a reviewed checkout in PowerShell:

```powershell
git clone https://github.com/xxvw/jgrep.git
Set-Location jgrep
.\scripts\install.ps1
```

The default resolves the latest published release and installs `jgrep.exe` to
`$env:LOCALAPPDATA\Programs\jgrep\bin`. Pin an installation or install from
already-produced assets with:

```powershell
.\scripts\install.ps1 -Version v0.1.1
.\scripts\install.ps1 -AssetDirectory .\dist -Version v0.1.1
```

The PowerShell installer supports the `x86_64-pc-windows-msvc` release only;
it rejects unsupported Windows architectures. Use `-InstallDir` or
`JGREP_INSTALL_DIR` to change the destination. It refuses to replace an
existing `jgrep.exe` unless `-Force` is supplied. Add the selected directory
to the current and future user `PATH` with the opt-in `-AddToPath` switch:

```powershell
.\scripts\install.ps1 -Version v0.1.1 -AddToPath
```

The Windows release uses the standard Microsoft Visual C++ runtime. Install
the current Visual C++ Redistributable when that runtime is not already
present. The script does not download or alter system runtimes.

For CI-style offline verification, `-AssetDirectory` requires the versioned
`.zip` archive and its `.zip.sha256` file (or a `SHA256SUMS` file) from the
same release. It uses the same checksum and safe single-entry extraction path
as an online install. `JGREP_VERSION`, `JGREP_ASSET_DIR`, `JGREP_REPOSITORY`,
and `JGREP_TARGET` can also be set as environment variables.

## Verify the installation and warm the model

Both installers run `jgrep --version` after placing the executable. Check it
again from a new shell after updating `PATH`:

```sh
jgrep --version
jgrep --download-model
jgrep --offline 'network connection failure' app.log
```

`--download-model` obtains the pinned Qwen GGUF file without searching input.
For a machine that must not use the network, transfer a validated model and
use `--model <path> --offline`; see [the model guide](model.md). Fixed-string
and regex searches (`-F` and `-E`) do not need a model.

## Tell coding agents to use compact search

The reusable agent-instruction template is
[`templates/AGENTS.jgrep.md`](../templates/AGENTS.jgrep.md). Copy its guidance
into the repository-level instruction file understood by the coding agent
(for example `AGENTS.md` for Codex-compatible agents or `CLAUDE.md` for Claude
Code), merging it with the repository’s existing instructions rather than
replacing them.

## Install the Codex plugin

`jgrep-agent` is a skills-only Codex plugin that packages the same compact
search guidance. Install `jgrep` first, then register this repository's
marketplace and plugin without cloning the repository:

```sh
codex plugin marketplace add xxvw/jgrep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@jgrep
```

Start a new Codex session after installation. The plugin and its localized
READMEs are in [`plugins/jgrep-agent/`](../plugins/jgrep-agent/); it contains
no model, server, or executable of its own.

The template teaches agents to use:

```sh
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'where authorization errors are handled' src/
```

`--ai` prints bounded `INPUT_LABEL:LINE` locators rather than source text. It
reduces tool-output tokens by letting an agent fetch only cited line ranges.
Agents must parse the final `:decimal-line-number` suffix, because a Windows
path can contain an earlier colon. They should treat a cap notice on stderr as
an incomplete result and narrow the next query. The usual grep statuses still
apply: `0` means locations were found, `1` is a normal no-location result, and
`2` identifies an invocation error. `-:LINE` results are best used only
with reproducible pipelines.

Use semantic `--ai` searches for concepts and behavior. Combine `--ai` with
`-F` or `-E` for known symbols or patterns when model-free lexical matching is
more precise. Normal formatting, count, file-list, quiet, score, and context
flags are deliberately incompatible with the compact agent format; the
template lists the exact restrictions.

## Script validation

Run the repository-provided static validation after changing installer or
template artifacts:

```sh
./scripts/validate-installers.sh
```

It always checks Bash syntax and the shared artifact contract, uses ShellCheck
when available, and parses the PowerShell script when `pwsh` is available.
The offline `--asset-dir` mode lets release jobs or maintainers exercise the
full download-free installation path against a freshly produced archive before
publication.
