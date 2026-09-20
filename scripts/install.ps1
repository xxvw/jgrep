<#
Installs a verified localjev-grep Windows x64 release for the current user.

Run this script from a reviewed checkout or a reviewed, versioned source URL.
It deliberately does not support `Invoke-WebRequest ... | Invoke-Expression`.
#>

[CmdletBinding()]
param(
    [string]$Version,

    [string]$InstallDir,

    [string]$AssetDirectory,

    [string]$Repository,

    [string]$Target,

    [switch]$Force,

    [switch]$AddToPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$DefaultRepository = "xxvw/localjev-grep"
$GitHubHeaders = @{
    Accept = "application/vnd.github+json"
    "User-Agent" = "localjev-grep-installer"
}

function Fail {
    param([Parameter(Mandatory)][string]$Message)

    throw "jgrep installer: $Message"
}

function Get-ExistingPathItem {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Description
    )

    try {
        $item = Get-Item -LiteralPath $Path -Force -ErrorAction Stop
        return $item
    }
    catch [System.Management.Automation.ItemNotFoundException] {
        return $null
    }
    catch {
        Fail "could not inspect ${Description}: $Path ($($_.Exception.Message))"
    }
}

function Assert-NoReparsePointInPath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Description
    )

    try {
        $fullPath = [System.IO.Path]::GetFullPath($Path)
    }
    catch {
        Fail "invalid $Description path: $Path"
    }

    $root = [System.IO.Path]::GetPathRoot($fullPath)
    if ([string]::IsNullOrWhiteSpace($root)) {
        Fail "invalid $Description path: $Path"
    }

    $relativePath = $fullPath.Substring($root.Length)
    $currentPath = $root
    foreach ($component in ($relativePath -split '[\\/]')) {
        if ([string]::IsNullOrWhiteSpace($component)) {
            continue
        }
        $currentPath = Join-Path $currentPath $component
        $item = Get-ExistingPathItem -Path $currentPath -Description $Description
        if ($null -eq $item) {
            break
        }
        if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            Fail "$Description contains a symbolic link or reparse point: $currentPath"
        }
    }

    return $fullPath
}

function Normalize-Tag {
    param([Parameter(Mandatory)][string]$Value)

    $candidate = $Value.Trim()
    if (-not $candidate.StartsWith("v", [System.StringComparison]::OrdinalIgnoreCase)) {
        $candidate = "v$candidate"
    }
    else {
        $candidate = "v" + $candidate.Substring(1)
    }
    if ($candidate -notmatch "^v[0-9A-Za-z][0-9A-Za-z._-]*$") {
        Fail "invalid release tag: $Value"
    }
    return $candidate
}

function Test-RepositoryName {
    param([Parameter(Mandatory)][string]$Value)

    if ($Value -notmatch "^[0-9A-Za-z._-]+/[0-9A-Za-z._-]+$") {
        Fail "invalid GitHub repository: $Value"
    }
}

function Get-DetectedTarget {
    $architecture = $env:PROCESSOR_ARCHITEW6432
    if ([string]::IsNullOrWhiteSpace($architecture)) {
        $architecture = $env:PROCESSOR_ARCHITECTURE
    }
    if ($architecture -notmatch "^(AMD64|x86_64)$") {
        Fail "no published Windows release supports architecture: $architecture"
    }
    return "x86_64-pc-windows-msvc"
}

function Test-Target {
    param([Parameter(Mandatory)][string]$Value)

    if ($Value -cne "x86_64-pc-windows-msvc") {
        Fail "unsupported Windows release target: $Value"
    }
}

function Invoke-GitHubApi {
    param([Parameter(Mandatory)][string]$Uri)

    try {
        return Invoke-RestMethod -Uri $Uri -Headers $GitHubHeaders -ErrorAction Stop
    }
    catch {
        Fail "could not query GitHub Release metadata: $($_.Exception.Message)"
    }
}

function Get-GitHubRelease {
    param(
        [Parameter(Mandatory)][string]$Repo,
        [string]$RequestedVersion
    )

    $base = "https://api.github.com/repos/$Repo/releases"
    if ([string]::IsNullOrWhiteSpace($RequestedVersion) -or $RequestedVersion -ieq "latest") {
        return Invoke-GitHubApi "$base/latest"
    }

    $requestedTag = Normalize-Tag $RequestedVersion
    return Invoke-GitHubApi "$base/tags/$requestedTag"
}

function Save-Download {
    param(
        [Parameter(Mandatory)][string]$Uri,
        [Parameter(Mandatory)][string]$Destination
    )

    if (-not $Uri.StartsWith("https://", [System.StringComparison]::OrdinalIgnoreCase)) {
        Fail "refusing a non-HTTPS download URL"
    }

    $parameters = @{
        Uri = $Uri
        OutFile = $Destination
        Headers = $GitHubHeaders
        ErrorAction = "Stop"
    }
    if ($PSVersionTable.PSEdition -eq "Desktop") {
        $parameters.UseBasicParsing = $true
    }

    for ($attempt = 1; $attempt -le 3; $attempt += 1) {
        try {
            Invoke-WebRequest @parameters | Out-Null
            return
        }
        catch {
            if ($attempt -eq 3) {
                Fail "could not download ${Uri}: $($_.Exception.Message)"
            }
            Start-Sleep -Seconds $attempt
        }
    }
}

function Read-ExpectedChecksum {
    param(
        [Parameter(Mandatory)][string]$ManifestPath,
        [Parameter(Mandatory)][string]$ArchiveName
    )

    $lines = @(
        ([System.IO.File]::ReadAllText($ManifestPath) -split '\r?\n') |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    )
    if ($lines.Count -eq 0) {
        Fail "invalid SHA-256 manifest: $ManifestPath"
    }

    $matches = @()
    foreach ($line in $lines) {
        $match = [System.Text.RegularExpressions.Regex]::Match(
            $line,
            "^(?<hash>[A-Fa-f0-9]{64})  (?<name>\S+)$"
        )
        if (-not $match.Success) {
            Fail "invalid SHA-256 manifest: $ManifestPath"
        }
        if ($match.Groups["name"].Value -ceq $ArchiveName) {
            $matches += $match
        }
    }
    if ($matches.Count -ne 1) {
        Fail "invalid SHA-256 manifest: $ManifestPath"
    }
    return $matches[0].Groups["hash"].Value
}

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [System.IO.File]::Open(
        $Path,
        [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::Read,
        [System.IO.FileShare]::Read
    )
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $algorithm.ComputeHash($stream)
        return ([System.BitConverter]::ToString($hash)).Replace("-", "").ToLowerInvariant()
    }
    finally {
        $algorithm.Dispose()
        $stream.Dispose()
    }
}

function Copy-ZipEntryToFile {
    param(
        [Parameter(Mandatory)][string]$ArchivePath,
        [Parameter(Mandatory)][string]$EntryName,
        [Parameter(Mandatory)][string]$Destination
    )

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [System.IO.Compression.ZipFile]::OpenRead($ArchivePath)
    try {
        $entries = @($archive.Entries | Where-Object { $_.FullName -ceq $EntryName })
        if ($entries.Count -ne 1 -or [string]::IsNullOrEmpty($entries[0].Name)) {
            Fail "release archive does not contain exactly one $EntryName"
        }

        $input = $entries[0].Open()
        try {
            $output = [System.IO.File]::Open(
                $Destination,
                [System.IO.FileMode]::CreateNew,
                [System.IO.FileAccess]::Write,
                [System.IO.FileShare]::None
            )
            try {
                $input.CopyTo($output)
            }
            finally {
                $output.Dispose()
            }
        }
        finally {
            $input.Dispose()
        }
    }
    finally {
        $archive.Dispose()
    }
}

function Add-InstallDirectoryToUserPath {
    param([Parameter(Mandatory)][string]$Directory)

    $current = [Environment]::GetEnvironmentVariable("Path", "User")
    $normalized = $Directory.TrimEnd("\\")
    $present = $false
    if (-not [string]::IsNullOrWhiteSpace($current)) {
        foreach ($entry in ($current -split ";")) {
            if (-not [string]::IsNullOrWhiteSpace($entry) -and
                $entry.Trim().TrimEnd("\\") -ieq $normalized) {
                $present = $true
                break
            }
        }
    }

    if (-not $present) {
        $newPath = if ([string]::IsNullOrWhiteSpace($current)) {
            $Directory
        }
        else {
            "$Directory;$current"
        }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = if ([string]::IsNullOrWhiteSpace($env:Path)) {
            $Directory
        }
        else {
            "$Directory;$env:Path"
        }
        Write-Host "Added $Directory to the current and future user PATH."
    }
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = $env:JGREP_VERSION
}
if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = $env:JGREP_INSTALL_DIR
}
if ([string]::IsNullOrWhiteSpace($AssetDirectory)) {
    $AssetDirectory = $env:JGREP_ASSET_DIR
}
if ([string]::IsNullOrWhiteSpace($Repository)) {
    $Repository = $env:JGREP_REPOSITORY
}
if ([string]::IsNullOrWhiteSpace($Target)) {
    $Target = $env:JGREP_TARGET
}

if ([string]::IsNullOrWhiteSpace($Repository)) {
    $Repository = $DefaultRepository
}
Test-RepositoryName $Repository

if ([string]::IsNullOrWhiteSpace($Target)) {
    $Target = Get-DetectedTarget
}
Test-Target $Target

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    if ([string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
        Fail "LOCALAPPDATA is not set; pass -InstallDir"
    }
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\jgrep\bin"
}

$archiveFormat = "zip"
if (-not [string]::IsNullOrWhiteSpace($AssetDirectory)) {
    if ([string]::IsNullOrWhiteSpace($Version) -or $Version -ieq "latest") {
        Fail "-AssetDirectory requires an explicit -Version"
    }
    if (-not (Test-Path -LiteralPath $AssetDirectory -PathType Container)) {
        Fail "asset directory does not exist: $AssetDirectory"
    }
    $AssetDirectory = (Resolve-Path -LiteralPath $AssetDirectory).Path
    $tag = Normalize-Tag $Version
}
else {
    # Windows PowerShell 5.1 otherwise defaults to older TLS negotiation on
    # some hosts. Adding TLS 1.2 keeps GitHub API and asset downloads usable.
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $release = Get-GitHubRelease -Repo $Repository -RequestedVersion $Version
    $releasedTag = Normalize-Tag ([string]$release.tag_name)
    if ([string]::IsNullOrWhiteSpace($Version) -or $Version -ieq "latest") {
        $tag = $releasedTag
    }
    else {
        $requestedTag = Normalize-Tag $Version
        if ($releasedTag -cne $requestedTag) {
            Fail "GitHub Release returned tag $releasedTag instead of requested tag $requestedTag"
        }
        $tag = $requestedTag
    }
}

$packageRoot = "localjev-grep-$tag-$Target"
$archiveName = "$packageRoot.$archiveFormat"
$checksumName = "$archiveName.sha256"
$entryName = "$packageRoot/jgrep.exe"
$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) (
    "localjev-grep-install-" + [guid]::NewGuid().ToString("N")
)

New-Item -ItemType Directory -Path $temporaryRoot | Out-Null
try {
    $archivePath = Join-Path $temporaryRoot $archiveName
    $checksumPath = Join-Path $temporaryRoot $checksumName

    if (-not [string]::IsNullOrWhiteSpace($AssetDirectory)) {
        $localArchive = Join-Path $AssetDirectory $archiveName
        if (-not (Test-Path -LiteralPath $localArchive -PathType Leaf)) {
            Fail "release archive is missing from asset directory: $archiveName"
        }
        Copy-Item -LiteralPath $localArchive -Destination $archivePath

        $localChecksum = Join-Path $AssetDirectory $checksumName
        $localManifest = Join-Path $AssetDirectory "SHA256SUMS"
        if (Test-Path -LiteralPath $localChecksum -PathType Leaf) {
            Copy-Item -LiteralPath $localChecksum -Destination $checksumPath
        }
        elseif (Test-Path -LiteralPath $localManifest -PathType Leaf) {
            Copy-Item -LiteralPath $localManifest -Destination $checksumPath
        }
        else {
            Fail "asset directory has neither $checksumName nor SHA256SUMS"
        }
    }
    else {
        $assets = @($release.assets)
        $archiveAsset = @($assets | Where-Object { $_.name -ceq $archiveName })
        $checksumAsset = @($assets | Where-Object { $_.name -ceq $checksumName })
        if ($archiveAsset.Count -ne 1 -or $checksumAsset.Count -ne 1) {
            Fail "GitHub Release $tag is missing the expected Windows x64 archive or checksum"
        }
        Save-Download -Uri ([string]$archiveAsset[0].browser_download_url) -Destination $archivePath
        Save-Download -Uri ([string]$checksumAsset[0].browser_download_url) -Destination $checksumPath
    }

    $expectedChecksum = Read-ExpectedChecksum -ManifestPath $checksumPath -ArchiveName $archiveName
    $actualChecksum = Get-Sha256 -Path $archivePath
    if (-not [string]::Equals(
            $expectedChecksum,
            $actualChecksum,
            [System.StringComparison]::OrdinalIgnoreCase
        )) {
        Fail "SHA-256 verification failed for $archiveName"
    }

    $InstallDir = Assert-NoReparsePointInPath -Path $InstallDir -Description "installation directory"
    $installDirectoryItem = Get-ExistingPathItem -Path $InstallDir -Description "installation directory"
    if ($null -ne $installDirectoryItem) {
        if (($installDirectoryItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            Fail "installation directory is a symbolic link or reparse point: $InstallDir"
        }
        if (-not $installDirectoryItem.PSIsContainer) {
            Fail "installation directory is not a directory: $InstallDir"
        }
    }
    else {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    $InstallDir = Assert-NoReparsePointInPath -Path $InstallDir -Description "installation directory"
    $destination = Join-Path $InstallDir "jgrep.exe"
    Assert-NoReparsePointInPath -Path $destination -Description "installation destination" | Out-Null
    $destinationItem = Get-ExistingPathItem -Path $destination -Description "installation destination"
    if ($null -ne $destinationItem) {
        if (($destinationItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            Fail "installation destination is a symbolic link or reparse point: $destination"
        }
        if (-not (Test-Path -LiteralPath $destination -PathType Leaf)) {
            Fail "installation destination is not a file: $destination"
        }
        if (-not $Force) {
            Fail "$destination already exists; rerun with -Force to replace it"
        }
    }

    $stagedBinary = Join-Path $InstallDir (".jgrep-" + [guid]::NewGuid().ToString("N") + ".exe")
    try {
        Copy-ZipEntryToFile -ArchivePath $archivePath -EntryName $entryName -Destination $stagedBinary
        if ((Get-Item -LiteralPath $stagedBinary).Length -le 0) {
            Fail "release archive contains an empty jgrep executable"
        }
        $expectedVersion = "jgrep $($tag.Substring(1))"
        $reportedVersion = @(& $stagedBinary --version)
        if ($LASTEXITCODE -ne 0) {
            Fail "release archive contains a jgrep executable that could not run; the current Visual C++ Redistributable may be missing"
        }
        if (($reportedVersion -join "`n") -cne $expectedVersion) {
            Fail "release archive version mismatch: expected $expectedVersion, got $($reportedVersion -join "`n")"
        }

        if ($null -ne $destinationItem) {
            [System.IO.File]::Replace($stagedBinary, $destination, $null, $true)
        }
        else {
            [System.IO.File]::Move($stagedBinary, $destination)
        }
    }
    finally {
        if (Test-Path -LiteralPath $stagedBinary) {
            Remove-Item -LiteralPath $stagedBinary -Force
        }
    }

    & $destination --version
    if ($LASTEXITCODE -ne 0) {
        Fail "installed jgrep could not run; the current Visual C++ Redistributable may be missing"
    }
    Write-Host "Installed jgrep $tag at $destination"
    if ($AddToPath) {
        Add-InstallDirectoryToUserPath -Directory $InstallDir
    }
    elseif ((@($env:Path -split ";") -notcontains $InstallDir)) {
        Write-Host "Add $InstallDir to your user PATH, or rerun this script with -AddToPath."
    }
}
finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}
