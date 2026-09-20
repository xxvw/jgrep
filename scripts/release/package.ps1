[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$BinaryPath,

    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]*$')]
    [string]$Version,

    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]*$')]
    [string]$Target,

    [Parameter(Mandatory)]
    [ValidateSet('tar.gz', 'zip')]
    [string]$Format,

    [Parameter(Mandatory)]
    [string]$OutputDirectory,

    [Parameter(Mandatory)]
    [string]$ThirdPartyLicensesPath,

    [Parameter(Mandatory)]
    [string]$Repository,

    [Parameter(Mandatory)]
    [string]$Ref
)

$ErrorActionPreference = "Stop"

function Copy-RequiredFile {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination
    )

    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        throw "required release file is missing: $Source"
    }
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$binary = (Resolve-Path -LiteralPath $BinaryPath).Path
if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
    throw "release binary is missing: $BinaryPath"
}

$licensesInventory = (Resolve-Path -LiteralPath $ThirdPartyLicensesPath).Path

$out = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $out | Out-Null
$packageRoot = "localjev-grep-v$Version-$Target"
$stage = Join-Path $out $packageRoot
if (Test-Path -LiteralPath $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
New-Item -ItemType Directory -Path $stage | Out-Null

Copy-Item -LiteralPath $binary -Destination (Join-Path $stage (Split-Path -Leaf $binary)) -Force
Copy-RequiredFile -Source (Join-Path $repositoryRoot "README.md") -Destination $stage
Copy-RequiredFile -Source (Join-Path $repositoryRoot "LICENSE") -Destination $stage
Copy-RequiredFile -Source (Join-Path $repositoryRoot "NOTICE") -Destination $stage
Copy-RequiredFile -Source (Join-Path $repositoryRoot "THIRD_PARTY_NOTICES.md") -Destination $stage
Copy-RequiredFile -Source $licensesInventory -Destination $stage
$projectLicenseDirectory = Join-Path $repositoryRoot "LICENSES"
if (-not (Test-Path -LiteralPath $projectLicenseDirectory -PathType Container)) {
    throw "required project license directory is missing: $projectLicenseDirectory"
}
Copy-Item -LiteralPath $projectLicenseDirectory -Destination (Join-Path $stage "LICENSES") -Recurse -Force

# Ship the translated readmes and maintainer-facing release/build guidance when
# present. The required English README above makes a malformed archive fail.
Get-ChildItem -LiteralPath $repositoryRoot -File -Filter "README*.md" | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $stage -Force
}
@("CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", "CODE_OF_CONDUCT.md", "RELEASING.md") | ForEach-Object {
    $candidate = Join-Path $repositoryRoot $_
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        Copy-Item -LiteralPath $candidate -Destination $stage -Force
    }
}
$docs = Join-Path $repositoryRoot "docs"
if (Test-Path -LiteralPath $docs -PathType Container) {
    Copy-Item -LiteralPath $docs -Destination (Join-Path $stage "docs") -Recurse -Force
}

# Keep the verified installers and the reusable agent instructions with every
# native archive, so an extracted release can be installed or integrated
# without first finding a separate source checkout.
Copy-RequiredFile -Source (Join-Path $repositoryRoot "AGENTS.md") -Destination $stage
$stageScripts = Join-Path $stage "scripts"
New-Item -ItemType Directory -Path $stageScripts | Out-Null
@("install.sh", "install.ps1", "validate-installers.sh") | ForEach-Object {
    Copy-RequiredFile -Source (Join-Path $repositoryRoot "scripts/$_") -Destination $stageScripts
}
$stageTemplates = Join-Path $stage "templates"
New-Item -ItemType Directory -Path $stageTemplates | Out-Null
Copy-RequiredFile -Source (Join-Path $repositoryRoot "templates/AGENTS.jgrep.md") -Destination $stageTemplates

# Copy-Item's Unix mode preservation is not part of the archive contract.
# Make the executable and Bash entry points executable explicitly in tarball
# targets.
if ($Format -eq "tar.gz") {
    $executablePaths = @(
        (Join-Path $stage (Split-Path -Leaf $binary)),
        (Join-Path $stageScripts "install.sh"),
        (Join-Path $stageScripts "validate-installers.sh")
    )
    foreach ($executablePath in $executablePaths) {
        & chmod 755 $executablePath
        if ($LASTEXITCODE -ne 0) {
            throw "could not mark release file executable: $executablePath"
        }
    }
}

$sourceNotice = @(
    "localjev-grep source and build instructions"
    ""
    "Corresponding source for this GPL-3.0-or-later binary is available at:"
    "https://github.com/$Repository/tree/$Ref"
    ""
    "The tagged source includes Cargo.lock, this release packaging script, and the"
    "documented `cargo xtask ci` validation entry point."
) -join "`n"
Set-Content -LiteralPath (Join-Path $stage "SOURCE.txt") -Value $sourceNotice -Encoding utf8

$archiveName = "$packageRoot.$Format"
$archive = Join-Path $out $archiveName
if (Test-Path -LiteralPath $archive) {
    Remove-Item -LiteralPath $archive -Force
}

if ($Format -eq "zip") {
    Compress-Archive -LiteralPath $stage -DestinationPath $archive -Force
} else {
    & tar -czf $archive -C $out $packageRoot
    if ($LASTEXITCODE -ne 0) {
        throw "tar failed while creating $archive"
    }
}

$hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
$checksum = "$archive.sha256"
# Use an explicit LF: the release job concatenates manifests on Linux, while
# PowerShell's default line ending on Windows would make an invalid filename.
Set-Content -LiteralPath $checksum -Value "$hash  $archiveName`n" -Encoding ascii -NoNewline

Remove-Item -LiteralPath $stage -Recurse -Force
Write-Output "Created $archive and $checksum"
