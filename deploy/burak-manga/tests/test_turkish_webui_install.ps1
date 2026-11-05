# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_turkish_webui_install.ps1
# Amac: Turkce WebUI ZIP kontrolunu Docker gerektirmeden test eder
# Modul - PowerShell
# Version: 0.1.0
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
    Write-Host 'Turkish WebUI offline checks passed.'
} finally {
    Remove-Item -LiteralPath $root -Force -Recurse
}
