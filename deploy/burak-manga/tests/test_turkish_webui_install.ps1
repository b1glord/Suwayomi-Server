# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_turkish_webui_install.ps1
# Amac: Turkce WebUI ZIP kontrolunu Docker gerektirmeden test eder
# Modul - PowerShell
# Version: 0.2.0
# Aciklama: Gecerli paket, bozuk hash ve zip traversal reddi
# Bagimli Oldugu Katman: Testing

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$script = Join-Path $PSScriptRoot '../scripts/install-turkish-webui.ps1'
$root = Join-Path ([IO.Path]::GetTempPath()) ('bm-tr-webui-test-' + [guid]::NewGuid().ToString('N'))
New-Item -Path $root -ItemType Directory | Out-Null
try {
    $zipPath = Join-Path $root 'Suwayomi-WebUI-TR-preview.zip'
    function New-Package([bool]$Unsafe) {
        if (Test-Path -LiteralPath $zipPath) { Remove-Item -LiteralPath $zipPath -Force }
        $f = [IO.File]::Open($zipPath, [IO.FileMode]::CreateNew)
        $z = [IO.Compression.ZipArchive]::new($f, [IO.Compression.ZipArchiveMode]::Create)
        try {
            $paths = @('index.html', 'revision') + @(1..10 | ForEach-Object { "assets/chunk-$_.js" })
            if ($Unsafe) { $paths += '../escape.txt' }
            foreach ($path in $paths) {
                $entry = $z.CreateEntry($path)
                $writer = [IO.StreamWriter]::new($entry.Open())
                try {
                    $value = if ($path -eq 'revision') { 'r3518' } else { 'asset' }
                    $writer.Write($value)
                } finally {
                    $writer.Dispose()
                }
            }
        } finally {
            $z.Dispose()
            $f.Dispose()
        }
        $hash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
        Set-Content -LiteralPath (Join-Path $root 'SHA256SUMS.txt') -Value "$hash  Suwayomi-WebUI-TR-preview.zip" -Encoding UTF8
    }

    New-Package $false
    & $script -Mode Check -ZipPath $zipPath
    $checksum = Join-Path $root 'SHA256SUMS.txt'
    Set-Content -LiteralPath $checksum -Value ("0" * 64 + '  Suwayomi-WebUI-TR-preview.zip') -Encoding UTF8
    try {
        & $script -Mode Check -ZipPath $zipPath
        throw 'EXPECTED_BAD_HASH_REJECTION'
    } catch {
        if ($_.Exception.Message -notmatch 'PREVIEW_ZIP_CHECKSUM_MISMATCH') { throw }
    }

    New-Package $true
    try {
        & $script -Mode Check -ZipPath $zipPath
        throw 'EXPECTED_ZIP_TRAVERSAL_REJECTION'
    } catch {
        if ($_.Exception.Message -notmatch 'UNSAFE_ZIP_ENTRY') { throw }
    }

    # Portable mock deployment: no real Docker/DB, only a disposable temp tree.
    New-Package $false
    $mockRoot = Join-Path $root 'test deployment'
    $scriptDir = Join-Path $mockRoot 'scripts'
    $mockData = Join-Path $mockRoot 'storage/Tachidesk'
    $mockWebUI = Join-Path $mockData 'webUI'
    $mockBackup = Join-Path $mockData 'webui-tr-backups/before-tr-20261105-000000'
    New-Item -Path $scriptDir -ItemType Directory -Force | Out-Null
    New-Item -Path $mockWebUI -ItemType Directory -Force | Out-Null
    New-Item -Path $mockBackup -ItemType Directory -Force | Out-Null
    Copy-Item -LiteralPath $script -Destination (Join-Path $scriptDir 'install-turkish-webui.ps1')
    Set-Content -LiteralPath (Join-Path $mockRoot 'compose.yaml') -Value 'services: {}'
    Set-Content -LiteralPath (Join-Path $mockData 'server.conf') -Value 'webUIFlavor = "Custom"'
    Set-Content -LiteralPath (Join-Path $mockWebUI 'index.html') -Value 'original-interface'
    Set-Content -LiteralPath (Join-Path $mockBackup 'index.html') -Value 'previous-original-interface'
    $mockInstaller = Join-Path $scriptDir 'install-turkish-webui.ps1'

    # Preflight never needs Docker and never changes WebUI.
    & $mockInstaller -Mode PreflightInstall -ZipPath $zipPath
    & $mockInstaller -Mode PreflightRestore
    if ((Get-Content -LiteralPath (Join-Path $mockWebUI 'index.html') -Raw).Trim() -ne 'original-interface') {
        throw 'PREFLIGHT_MODIFIED_WEBUI'
    }

    # Install + Restore paths are exercised with an inert Docker command.
    function global:docker { $global:LASTEXITCODE = 0 }
    try {
        & $mockInstaller -Mode Install -ZipPath $zipPath
        $written = (Get-Content -LiteralPath (Join-Path $mockWebUI 'index.html') -Raw).Trim()
        if ($written -ne 'asset') { throw 'WEBUI_INSTALL_DID_NOT_REPLACE_INDEX' }
        & $mockInstaller -Mode PreflightRestore
        & $mockInstaller -Mode Restore
        $restored = (Get-Content -LiteralPath (Join-Path $mockWebUI 'index.html') -Raw).Trim()
        if ($restored -ne 'original-interface') { throw 'WEBUI_RESTORE_DID_NOT_RECOVER_ORIGINAL' }
        if (-not (Test-Path -LiteralPath (Join-Path $mockData 'webui-tr-backups'))) {
            throw 'WEBUI_BACKUP_DIRECTORY_REMOVED'
        }
    } finally {
        Remove-Item -Path Function:\docker -ErrorAction SilentlyContinue
    }

    Write-Host 'Turkish WebUI offline checks passed.'
} finally {
    Remove-Item -LiteralPath $root -Force -Recurse
}
