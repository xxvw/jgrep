[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]*$')]
    [string]$Version,

    [Parameter(Mandatory)]
    [string]$OutputDirectory,

    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9A-Za-z._-]+/[0-9A-Za-z._-]+$')]
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
        throw "required installer-bundle file is missing: $Source"
    }
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

function Copy-RequiredDirectory {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination
    )

    if (-not (Test-Path -LiteralPath $Source -PathType Container)) {
        throw "required installer-bundle directory is missing: $Source"
    }
    Copy-Item -LiteralPath $Source -Destination $Destination -Recurse -Force
}

function Write-Checksum {
    param([Parameter(Mandatory)][string]$Archive)

    $hash = (Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
    $checksum = "$Archive.sha256"
    $name = Split-Path -Leaf $Archive
    # Release manifests are consumed on every supported platform. Keep the
    # separator and line ending stable even when this script runs on Windows.
    Set-Content -LiteralPath $checksum -Value "$hash  $name`n" -Encoding ascii -NoNewline
}

function New-ZipArchive {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination
    )

    # Compress-Archive can skip hidden paths. The bundle contains .agents, so
    # create the ZIP directly to keep its contents identical to the tarball.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory(
        $Source,
        $Destination,
        [System.IO.Compression.CompressionLevel]::Optimal,
        $true
    )
}

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$out = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $out | Out-Null

$bundleRoot = "localjev-grep-installers-v$Version"
$stage = Join-Path $out $bundleRoot
if (Test-Path -LiteralPath $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
New-Item -ItemType Directory -Path $stage | Out-Null

try {
    @("README.md", "LICENSE", "NOTICE", "THIRD_PARTY_NOTICES.md") | ForEach-Object {
        Copy-RequiredFile -Source (Join-Path $repositoryRoot $_) -Destination $stage
    }
    Get-ChildItem -LiteralPath $repositoryRoot -File -Filter "README*.md" | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination $stage -Force
    }
    @("AGENTS.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", "CODE_OF_CONDUCT.md", "RELEASING.md") | ForEach-Object {
        Copy-RequiredFile -Source (Join-Path $repositoryRoot $_) -Destination $stage
    }
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot "LICENSES") -Destination (Join-Path $stage "LICENSES")
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot "docs") -Destination (Join-Path $stage "docs")
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot "eval") -Destination (Join-Path $stage "eval")
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot "templates") -Destination (Join-Path $stage "templates")
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot "installers") -Destination (Join-Path $stage "installers")
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot "plugins") -Destination (Join-Path $stage "plugins")
    Copy-RequiredDirectory -Source (Join-Path $repositoryRoot ".agents") -Destination (Join-Path $stage ".agents")

    $stageScripts = Join-Path $stage "scripts"
    New-Item -ItemType Directory -Path $stageScripts | Out-Null
    @("install.sh", "install.ps1", "validate-installers.sh", "generate-localized-installers.py") | ForEach-Object {
        Copy-RequiredFile -Source (Join-Path $repositoryRoot "scripts/$_") -Destination $stageScripts
    }

    $sourceNotice = @(
        "localjev-grep installer source and build instructions"
        ""
        "Corresponding source for these GPL-3.0-or-later installer files is available at:"
        "https://github.com/$Repository/tree/$Ref"
        ""
        "The tagged source includes checksum verification, the localized wrappers,"
        "and the documented cargo xtask ci validation entry point."
    ) -join "`n"
    Set-Content -LiteralPath (Join-Path $stage "SOURCE.txt") -Value $sourceNotice -Encoding utf8

    # Copy-Item does not provide a portable executable-mode contract. Mark the
    # Unix entry points explicitly before creating the tarball.
    $unixScripts = @(
        (Join-Path $stageScripts "install.sh"),
        (Join-Path $stageScripts "validate-installers.sh")
    ) + @(Get-ChildItem -LiteralPath (Join-Path $stage "installers") -Recurse -Filter "*.sh" | ForEach-Object { $_.FullName })
    foreach ($script in $unixScripts) {
        & chmod 755 $script
        if ($LASTEXITCODE -ne 0) {
            throw "could not mark installer-bundle script executable: $script"
        }
    }

    $tarArchive = Join-Path $out "$bundleRoot.tar.gz"
    $zipArchive = Join-Path $out "$bundleRoot.zip"
    @($tarArchive, "$tarArchive.sha256", $zipArchive, "$zipArchive.sha256") | ForEach-Object {
        if (Test-Path -LiteralPath $_) {
            Remove-Item -LiteralPath $_ -Force
        }
    }

    & tar -czf $tarArchive -C $out $bundleRoot
    if ($LASTEXITCODE -ne 0) {
        throw "tar failed while creating $tarArchive"
    }
    New-ZipArchive -Source $stage -Destination $zipArchive
    Write-Checksum -Archive $tarArchive
    Write-Checksum -Archive $zipArchive

    Write-Output "Created $tarArchive, $tarArchive.sha256, $zipArchive, and $zipArchive.sha256"
}
finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
}
