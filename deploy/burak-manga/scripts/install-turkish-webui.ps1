# Dosya Yolu: REPO_ROOT/deploy/burak-manga/scripts/install-turkish-webui.ps1
# Amac: Turkce WebUI test paketini yedekli ve geri alinabilir bicimde kurar
# Modul - PowerShell
# Version: 0.2.0
# Aciklama: Sadece webUI klasorunu degistirir; storage/DB ve Docker servislerini degistirmez
# Bagimli Oldugu Katman: Tool

[CmdletBinding()]
param(
    [ValidateSet('Check', 'PreflightInstall', 'PreflightRestore', 'Install', 'Restore')]
    [string]$Mode = 'Check',
    [string]$ZipPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Assert-PlainDirectory([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        throw "DIRECTORY_NOT_FOUND: $Path"
    }
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "REPARSE_DIRECTORY_NOT_ALLOWED: $Path"
    }
}

function Test-PreviewPackage([string]$Package) {
    $full = [IO.Path]::GetFullPath($Package)
    if (-not (Test-Path -LiteralPath $full -PathType Leaf)) {
        throw "PREVIEW_ZIP_NOT_FOUND: $full"
    }
    if ([IO.Path]::GetFileName($full) -ne 'Suwayomi-WebUI-TR-preview.zip') {
        throw 'UNEXPECTED_PACKAGE_NAME'
    }
    $checksumPath = Join-Path ([IO.Path]::GetDirectoryName($full)) 'SHA256SUMS.txt'
    if (-not (Test-Path -LiteralPath $checksumPath -PathType Leaf)) {
        throw 'SHA256SUMS_MISSING'
    }
    $checksumLines = @(Get-Content -LiteralPath $checksumPath -Encoding UTF8)
    $checksum = $checksumLines | Where-Object {
        $_ -match '^[a-fA-F0-9]{64}  Suwayomi-WebUI-TR-preview[.]zip$'
    } | Select-Object -First 1
    if (-not $checksum) {
        throw 'SHA256SUMS_INVALID'
    }
    $expected = $checksum.Substring(0, 64).ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expected -ne $actual) {
        throw 'PREVIEW_ZIP_CHECKSUM_MISMATCH'
    }

    $zip = [IO.Compression.ZipFile]::OpenRead($full)
    try {
        $names = [Collections.Generic.HashSet[string]]::new(
            [StringComparer]::OrdinalIgnoreCase
        )
        $fileCount = 0
        [long]$totalSize = 0
        foreach ($entry in $zip.Entries) {
            $name = $entry.FullName
            $normalized = $name.Replace('\', '/')
            if ([string]::IsNullOrWhiteSpace($name) -or
                $name -ne $normalized -or
                $name.StartsWith('/') -or
                $name.Contains(':') -or
                ($normalized.Split('/') | Where-Object { $_ -eq '..' -or $_ -eq '.' })) {
                throw "UNSAFE_ZIP_ENTRY: $name"
            }
            # UNIX symlink entries must not be followed on extraction.
            $unixType = ($entry.ExternalAttributes -shr 16) -band 0xF000
            if ($unixType -eq 0xA000) {
                throw "ZIP_SYMLINK_NOT_ALLOWED: $name"
            }
            if (-not $names.Add($normalized)) {
                throw "DUPLICATE_ZIP_ENTRY: $name"
            }
            if ($name.EndsWith('/')) { continue }
            $fileCount++
            $totalSize += $entry.Length
            if ($fileCount -gt 10000 -or $totalSize -gt 150MB) {
                throw 'ZIP_LIMIT_EXCEEDED'
            }
        }
        if (-not $names.Contains('index.html') -or
            -not $names.Contains('revision') -or
            -not (@($names | Where-Object { $_ -like 'assets/*' }).Count -gt 0) -or
            $fileCount -lt 10) {
            throw 'INCOMPLETE_PREVIEW_WEBUI'
        }
        $revisionEntry = $zip.GetEntry('revision')
        $reader = [IO.StreamReader]::new($revisionEntry.Open())
        try {
            $revision = $reader.ReadToEnd().Trim()
        } finally {
            $reader.Dispose()
        }
        if ($revision -notmatch '^r[0-9]+$') {
            throw 'INVALID_PREVIEW_REVISION'
        }
        Write-Host "Preview ZIP dogrulandi. Revision: $revision; dosya sayisi: $fileCount; SHA256: $actual"
    } finally {
        $zip.Dispose()
    }
    return $full
}

function Assert-StoppedSuwayomi([string]$Compose) {
    $lines = @(& docker compose -f $Compose ps --status running --services 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "DOCKER_COMPOSE_UNAVAILABLE: $($lines -join ' ')"
    }
    if (@($lines | Where-Object { $_.Trim() -eq 'suwayomi' }).Count -gt 0) {
        throw 'SUWAYOMI_IS_RUNNING: Indirmeleri duraklatin ve sadece suwayomi servisini durdurun.'
    }
}

$deployment = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$compose = Join-Path $deployment 'compose.yaml'
$data = Join-Path $deployment 'storage/Tachidesk'
$webui = Join-Path $data 'webUI'
$backups = Join-Path $data 'webui-tr-backups'

if ($Mode -in @('Check', 'PreflightInstall', 'Install')) {
    if ([string]::IsNullOrWhiteSpace($ZipPath)) {
        throw 'PREVIEW_ZIP_PATH_REQUIRED'
    }
    $verifiedZip = Test-PreviewPackage -Package $ZipPath
    if ($Mode -eq 'Check') {
        Write-Host 'Kontrol basarili. Hicbir dosya veya Docker servisi degistirilmedi.'
        return
    }
}

if (-not (Test-Path -LiteralPath $compose -PathType Leaf)) {
    throw 'COMPOSE_MISSING'
}
Assert-PlainDirectory $data

# Preflight is read-only; the Rust launcher calls this before stopping Suwayomi.
if ($Mode -eq 'PreflightInstall') {
    $config = Join-Path $data 'server.conf'
    if (-not (Test-Path -LiteralPath $config -PathType Leaf)) {
        throw 'SERVER_CONFIG_NOT_FOUND: Settings > WebUI > Flavor = Custom'
    }
    $configText = Get-Content -LiteralPath $config -Raw -Encoding UTF8
    if ($configText -notmatch '(?im)^\s*["'']?webUIFlavor["'']?\s*[:=]\s*["'']?Custom\b') {
        throw 'CUSTOM_WEBUI_NOT_SELECTED: Settings > WebUI > Flavor = Custom'
    }
    Assert-PlainDirectory $webui
    if (-not (Test-Path -LiteralPath (Join-Path $webui 'index.html') -PathType Leaf)) {
        throw 'INSTALLED_WEBUI_INVALID'
    }
    Write-Host 'Turkce WebUI kurulum on kontrolu basarili. Dosyalar degismedi.'
    return
}

if ($Mode -eq 'PreflightRestore') {
    Assert-PlainDirectory $backups
    $available = @(Get-ChildItem -LiteralPath $backups -Directory |
        Where-Object { $_.Name -like 'before-tr-*' })
    if ($available.Count -eq 0) { throw 'NO_WEBUI_BACKUP' }
    $latest = $available | Sort-Object Name -Descending | Select-Object -First 1
    Assert-PlainDirectory $latest.FullName
    if (-not (Test-Path -LiteralPath (Join-Path $latest.FullName 'index.html') -PathType Leaf)) {
        throw 'WEBUI_BACKUP_INVALID'
    }
    if (Test-Path -LiteralPath $webui) { Assert-PlainDirectory $webui }
    Write-Host 'WebUI geri alma on kontrolu basarili. Dosyalar degismedi.'
    return
}

Assert-StoppedSuwayomi $compose

if ($Mode -eq 'Install') {
    $config = Join-Path $data 'server.conf'
    if (-not (Test-Path -LiteralPath $config -PathType Leaf)) {
        throw 'SERVER_CONFIG_NOT_FOUND: WebUI ayarlarinda once Custom secin.'
    }
    $configText = Get-Content -LiteralPath $config -Raw -Encoding UTF8
    if ($configText -notmatch '(?im)^\s*["'']?webUIFlavor["'']?\s*[:=]\s*["'']?Custom\b') {
        throw 'CUSTOM_WEBUI_NOT_SELECTED: WebUI ayarlarinda Flavor=Custom secip kaydedin.'
    }
    Assert-PlainDirectory $webui
    if (-not (Test-Path -LiteralPath (Join-Path $webui 'index.html') -PathType Leaf)) {
        throw 'INSTALLED_WEBUI_INVALID'
    }
    if (-not (Test-Path -LiteralPath $backups)) {
        New-Item -ItemType Directory -Path $backups | Out-Null
    }
    Assert-PlainDirectory $backups
    $stamp = (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
    $backup = Join-Path $backups "before-tr-$stamp"
    $stage = Join-Path $data "webui-tr-staging-$stamp"
    [IO.Compression.ZipFile]::ExtractToDirectory($verifiedZip, $stage)
    try {
        $stageRevision = (Get-Content -LiteralPath (Join-Path $stage 'revision') -Raw).Trim()
        if ($stageRevision -notmatch '^r[0-9]+$') {
            throw 'STAGED_REVISION_INVALID'
        }
        Move-Item -LiteralPath $webui -Destination $backup
        try {
            Move-Item -LiteralPath $stage -Destination $webui
        } catch {
            Move-Item -LiteralPath $backup -Destination $webui
            throw
        }
        Write-Host "Turkce WebUI kuruldu. Eski WebUI yedegi: $backup"
        Write-Host 'Yalnizca Suwayomi servisini yeniden baslatabilirsiniz: docker compose -f deploy/burak-manga/compose.yaml up -d --no-deps suwayomi'
    } finally {
        if (Test-Path -LiteralPath $stage) {
            Remove-Item -LiteralPath $stage -Force -Recurse
        }
    }
} elseif ($Mode -eq 'Restore') {
    if (-not (Test-Path -LiteralPath $backups -PathType Container)) {
        throw 'NO_WEBUI_BACKUP'
    }
    Assert-PlainDirectory $backups
    $latest = Get-ChildItem -LiteralPath $backups -Directory |
        Where-Object { $_.Name -like 'before-tr-*' } |
        Sort-Object -Property Name -Descending |
        Select-Object -First 1
    if (-not $latest) {
        throw 'NO_WEBUI_BACKUP'
    }
    $stamp = (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
    $replaced = Join-Path $backups "tr-replaced-$stamp"
    if (Test-Path -LiteralPath $webui) {
        Assert-PlainDirectory $webui
        Move-Item -LiteralPath $webui -Destination $replaced
    }
    try {
        Move-Item -LiteralPath $latest.FullName -Destination $webui
    } catch {
        if (Test-Path -LiteralPath $replaced) {
            Move-Item -LiteralPath $replaced -Destination $webui
        }
        throw
    }
    Write-Host "Onceki WebUI geri alindi: $webui"
    Write-Host "Orijinal WebUI icin Settings > WebUI > Flavor = WebUI secilebilir."
    Write-Host 'Arayuzu acmak icin yalnizca suwayomi servisini baslatin.'
}
