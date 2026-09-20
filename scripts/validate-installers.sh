#!/usr/bin/env bash
# Lightweight, network-free validation for installer and agent-template files.

set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
bash_installer="${repository_root}/scripts/install.sh"
powershell_installer="${repository_root}/scripts/install.ps1"
agent_template="${repository_root}/templates/AGENTS.jgrep.md"
integration_guide="${repository_root}/docs/installation-and-agents.md"

require_text() {
    local file="$1"
    local text="$2"
    if ! grep -Fq -- "$text" "$file"; then
        printf 'validation failed: %s does not contain %s\n' "$file" "$text" >&2
        exit 1
    fi
}

for file in "$bash_installer" "$powershell_installer" "$agent_template" "$integration_guide"; do
    [[ -f "$file" ]] || {
        printf 'validation failed: required artifact is missing: %s\n' "$file" >&2
        exit 1
    }
done

bash -n "$bash_installer"

if command -v shellcheck >/dev/null 2>&1; then
    shellcheck -s bash "$bash_installer" "$repository_root/scripts/validate-installers.sh"
else
    printf 'ShellCheck is not installed; Bash syntax validation passed.\n' >&2
fi

# Keep the two installers aligned with the release archive and checksum
# contract even on a host that cannot parse PowerShell.
require_text "$bash_installer" "--asset-dir"
require_text "$bash_installer" "SHA-256 verification failed"
require_text "$bash_installer" "localjev-grep-\${tag}-\${target}"
require_text "$bash_installer" "version mismatch"
require_text "$powershell_installer" "-AssetDirectory"
require_text "$powershell_installer" "Get-Sha256"
require_text "$powershell_installer" "Copy-ZipEntryToFile"
require_text "$powershell_installer" "x86_64-pc-windows-msvc"
require_text "$powershell_installer" "version mismatch"
require_text "$powershell_installer" "returned tag"
require_text "$powershell_installer" "reparse point"
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
    # shellcheck disable=SC2016 # The quoted body is PowerShell, not Bash.
    JGREP_POWERSHELL_INSTALLER="$powershell_installer" \
        pwsh -NoProfile -NonInteractive -Command '
            $tokens = $null
            $errors = $null
            [System.Management.Automation.Language.Parser]::ParseFile(
                $env:JGREP_POWERSHELL_INSTALLER,
                [ref] $tokens,
                [ref] $errors
            ) | Out-Null
            if ($errors.Count -ne 0) {
                $errors | ForEach-Object { Write-Error $_ }
                exit 1
            }
        '
else
    printf 'pwsh is not installed; PowerShell contract validation passed.\n' >&2
fi

printf 'Installer and agent-template validation passed.\n'
