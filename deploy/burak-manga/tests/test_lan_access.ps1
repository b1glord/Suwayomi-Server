# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_lan_access.ps1
# Amac: Telefon erisimi icin auth, IP ve geri alma sozlesmesini test eder
# Modul - PowerShell
# Version: 0.9.0
# Aciklama: Yerel temp alani ve sahte Docker; canli ag degistirilmez
# Bagimli Oldugu Katman: Testing

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot '../scripts/configure-lan-access.ps1'
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('bm-lan-turkce-' + [guid]::NewGuid().ToString('N'))
$scriptDir = Join-Path $testRoot 'scripts'
$data = Join-Path $testRoot 'storage/Tachidesk'
$overlay = Join-Path $testRoot 'compose.lan.yaml'
$conf = Join-Path $data 'server.conf'

function global:Get-NetIPAddress {
    param([string]$AddressFamily, [string]$IPAddress, [string]$ErrorAction)
    if ($AddressFamily -eq 'IPv4' -and $IPAddress -eq '192.168.1.10') {
        return [pscustomobject]@{ IPAddress = '192.168.1.10' }
    }
}
$global:lan_calls = @()
$global:lan_fail_once = $false
function global:docker {
    param([Parameter(ValueFromRemainingArguments=$true)][string[]]$Values)
    $line = $Values -join ' '
    $global:lan_calls += $line
    $global:LASTEXITCODE = 0
    if ($global:lan_fail_once -and $line -match 'up -d --no-deps suwayomi$') {
        $global:lan_fail_once = $false
        $global:LASTEXITCODE = 1
    }
}
function Assert-Rejected([scriptblock]$Command, [string]$Expected) {
    try {
        & $Command
        throw "EXPECTED_REJECTION_NOT_OCCURRED: $Expected"
    } catch {
        if ($_.Exception.Message -notmatch [regex]::Escape($Expected)) { throw }
    }
}

New-Item -ItemType Directory -Path $scriptDir -Force | Out-Null
New-Item -ItemType Directory -Path $data -Force | Out-Null
try {
    Copy-Item -LiteralPath $source -Destination (Join-Path $scriptDir 'configure-lan-access.ps1')
    $installer = Join-Path $scriptDir 'configure-lan-access.ps1'
    $base = Join-Path $testRoot 'compose.yaml'
    Set-Content -LiteralPath $base -Value @('services:','  suwayomi:','    ports:','      - "127.0.0.1:4567:4567"') -Encoding UTF8
    Set-Content -LiteralPath $conf -Value @(
        'server.ip = "0.0.0.0"',
        'server.authMode = "NONE"',
        'server.authUsername = ""',
        'server.authPassword = ""'
    ) -Encoding UTF8

    & $installer -Mode Status
    Assert-Rejected { & $installer -Mode Enable -IpAddress '192.168.1.10' } 'LAN_AUTH_REQUIRED'
    if (Test-Path -LiteralPath $overlay) { throw 'UNAUTHENTICATED_OVERLAY_CREATED' }
    if ($global:lan_calls.Count -ne 0) { throw 'UNAUTHENTICATED_DOCKER_CALLED' }

    Set-Content -LiteralPath $conf -Value @(
        'server.ip = "0.0.0.0"',
        'server.authMode = "UI_LOGIN"',
        'server.authUsername = "local-user"',
        'server.authPassword = "test-only-value"'
    ) -Encoding UTF8
    Assert-Rejected { & $installer -Mode Enable -IpAddress '8.8.8.8' } 'LAN_PRIVATE_IPV4_REQUIRED'
    Assert-Rejected { & $installer -Mode Enable -IpAddress '192.168.1.11' } 'LAN_IP_NOT_ASSIGNED_TO_WINDOWS_HOST'
    if ($global:lan_calls.Count -ne 0) { throw 'INVALID_IP_DOCKER_CALLED' }

    & $installer -Mode Enable -IpAddress '192.168.1.10'
    $text = Get-Content -LiteralPath $overlay -Raw -Encoding UTF8
    if ($text -notmatch '192\.168\.1\.10:4567:4567') { throw 'VALID_LAN_BIND_MISSING' }
    if ($text -match 'test-only-value|local-user') { throw 'CREDENTIALS_LEAKED_TO_COMPOSE' }
    if ($global:lan_calls.Count -ne 2) { throw 'EXPECTED_CONFIG_AND_UP_CALLS' }
    & $installer -Mode Status
    & $installer -Mode Disable
    if (Test-Path -LiteralPath $overlay) { throw 'LAN_DISABLE_DID_NOT_REMOVE_OVERRIDE' }
    if (-not (Test-Path -LiteralPath $base)) { throw 'LAN_DISABLE_REMOVED_BASE' }
    if (-not (Test-Path -LiteralPath $conf)) { throw 'LAN_DISABLE_REMOVED_CONFIG' }
    if (@(Get-ChildItem -LiteralPath $data -Filter 'lan-disabled-*.yaml').Count -ne 1) {
        throw 'LAN_DISABLED_BACKUP_MISSING'
    }

    # On a failed recreate the previous localhost-only state must be restored.
    $global:lan_fail_once = $true
    Assert-Rejected { & $installer -Mode Enable -IpAddress '192.168.1.10' } 'LAN_ENABLE_FAILED_RESTORED_LOCALHOST'
    if (Test-Path -LiteralPath $overlay) { throw 'LAN_ENABLE_FAILURE_LEFT_PORT_OPEN' }
    if (@(Get-ChildItem -LiteralPath $data -Filter 'lan-failed-*.yaml').Count -ne 1) {
        throw 'LAN_FAILURE_BACKUP_MISSING'
    }

    # Existing non-managed YAML must never be replaced.
    Set-Content -LiteralPath $overlay -Value 'services: {suwayomi: {ports: ["0.0.0.0:4567:4567"]}}'
    Assert-Rejected { & $installer -Mode Disable } 'LAN_OVERLAY_NOT_MANAGED_BY_LAUNCHER'
    if ((Get-Content -LiteralPath $overlay -Raw).Trim() -notmatch '0\.0\.0\.0') {
        throw 'UNTRUSTED_COMPOSE_OVERWRITTEN'
    }
    Write-Host 'LAN helper mock tests passed.'
} finally {
    Remove-Item -LiteralPath $testRoot -Recurse -Force
    Remove-Item Function:\Get-NetIPAddress -ErrorAction SilentlyContinue
    Remove-Item Function:\docker -ErrorAction SilentlyContinue
    $global:lan_calls = @()
    $global:lan_fail_once = $false
}
