#!/usr/bin/env bash
# Lightweight, network-free validation for installer and agent-template files.

set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
bash_installer="${repository_root}/scripts/install.sh"
powershell_installer="${repository_root}/scripts/install.ps1"
localized_generator="${repository_root}/scripts/generate-localized-installers.py"
localized_installer_root="${repository_root}/installers"
agent_template="${repository_root}/templates/AGENTS.jgrep.md"
integration_guide="${repository_root}/docs/installation-and-agents.md"
readonly -a localized_locales=(ar de en es fr hi it ja ko pt-BR ru zh-CN)

bash_wrapper_files=()
powershell_wrapper_files=()

require_text() {
    local file="$1"
    local text="$2"
    if ! grep -Fq -- "$text" "$file"; then
        printf 'validation failed: %s does not contain %s\n' "$file" "$text" >&2
        exit 1
    fi
}

for file in "$bash_installer" "$powershell_installer" "$localized_generator" "$agent_template" "$integration_guide"; do
    [[ -f "$file" ]] || {
        printf 'validation failed: required artifact is missing: %s\n' "$file" >&2
        exit 1
    }
done

for locale in "${localized_locales[@]}"; do
    bash_wrapper="${localized_installer_root}/${locale}/install.sh"
    powershell_wrapper="${localized_installer_root}/${locale}/install.ps1"
    for file in "$bash_wrapper" "$powershell_wrapper"; do
        [[ -f "$file" && ! -L "$file" ]] || {
            printf 'validation failed: localized installer is missing or a symbolic link: %s\n' "$file" >&2
            exit 1
        }
    done
    bash_wrapper_files+=("$bash_wrapper")
    powershell_wrapper_files+=("$powershell_wrapper")
done

if command -v python3 >/dev/null 2>&1; then
    python3 "$localized_generator" --check
elif command -v python >/dev/null 2>&1; then
    python "$localized_generator" --check
else
    printf 'validation failed: Python 3 is required to check generated localized installers\n' >&2
    exit 1
fi

for bash_file in "$bash_installer" "$repository_root/scripts/validate-installers.sh" "${bash_wrapper_files[@]}"; do
    bash -n "$bash_file"
done

if command -v shellcheck >/dev/null 2>&1; then
    shellcheck -s bash "$bash_installer" "$repository_root/scripts/validate-installers.sh"
    # Generated translations legitimately contain typographic apostrophes.
    shellcheck -s bash -e SC1112 "${bash_wrapper_files[@]}"
else
    printf 'ShellCheck is not installed; Bash syntax validation passed for core and localized installers.\n' >&2
fi

# Keep the two installers aligned with the release archive and checksum
# contract even on a host that cannot parse PowerShell.
require_text "$bash_installer" "--asset-dir"
require_text "$bash_installer" "SHA-256 verification failed"
require_text "$bash_installer" "jgrep-\${tag}-\${target}"
require_text "$bash_installer" "version mismatch"
require_text "$powershell_installer" "-AssetDirectory"
require_text "$powershell_installer" "Get-Sha256"
require_text "$powershell_installer" "Copy-ZipEntryToFile"
require_text "$powershell_installer" "x86_64-pc-windows-msvc"
require_text "$powershell_installer" "version mismatch"
require_text "$powershell_installer" "returned tag"
require_text "$powershell_installer" "reparse point"
for wrapper in "${bash_wrapper_files[@]}"; do
    require_text "$wrapper" "scripts/install.sh"
    # shellcheck disable=SC2016 # Match the literal generated wrapper source.
    require_text "$wrapper" 'exec bash "${core_installer}" "$@"'
done
for wrapper in "${powershell_wrapper_files[@]}"; do
    require_text "$wrapper" "scripts/install.ps1"
    require_text "$wrapper" "[System.IO.Directory]::GetParent"
    # shellcheck disable=SC2016 # Match the literal generated wrapper source.
    require_text "$wrapper" '& $coreInstaller @PSBoundParameters'
done
require_text "$agent_template" "jgrep --ai"
require_text "$agent_template" "--ai-max-results"
require_text "$agent_template" "decimal line number"
require_text "$agent_template" "--color=always"
require_text "$agent_template" "-:LINE"
require_text "$integration_guide" "scripts/install.sh"
require_text "$integration_guide" "install.ps1"
require_text "$integration_guide" "templates/AGENTS.jgrep.md"
require_text "$integration_guide" "-:LINE"

if command -v pwsh >/dev/null 2>&1; then
    powershell_command="pwsh"
elif command -v powershell.exe >/dev/null 2>&1; then
    powershell_command="powershell.exe"
else
    powershell_command=""
fi

if [[ -n "$powershell_command" ]]; then
    # shellcheck disable=SC2016 # The quoted body is PowerShell, not Bash.
    JGREP_POWERSHELL_INSTALLER_ROOT="$repository_root" \
        "$powershell_command" -NoProfile -NonInteractive -Command '
            $root = $env:JGREP_POWERSHELL_INSTALLER_ROOT
            $files = @(
                (Join-Path $root "scripts/install.ps1")
            ) + @(
                Get-ChildItem -LiteralPath (Join-Path $root "installers") -Recurse -File -Filter "install.ps1"
            )
            if ($files.Count -ne 13) {
                Write-Error "expected the core PowerShell installer and 12 localized wrappers; found $($files.Count)"
                exit 1
            }
            foreach ($file in $files) {
                $tokens = $null
                $errors = $null
                [System.Management.Automation.Language.Parser]::ParseFile(
                    [string] $file,
                    [ref] $tokens,
                    [ref] $errors
                ) | Out-Null
                if ($errors.Count -ne 0) {
                    $errors | ForEach-Object { Write-Error "${file}: $_" }
                    exit 1
                }
            }
        '
else
    printf 'PowerShell is not installed; PowerShell syntax validation was skipped.\n' >&2
fi

printf 'Installer and agent-template validation passed.\n'
