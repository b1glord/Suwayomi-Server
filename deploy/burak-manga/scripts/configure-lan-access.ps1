# Dosya Yolu: REPO_ROOT/deploy/burak-manga/scripts/configure-lan-access.ps1
# Amac: Suwayomi telefon erisimi icin yerel IPv4 yayini ac/kapat
# Modul - PowerShell
# Version: 0.9.1
# Aciklama: Auth korumasi, yalnizca suwayomi recreate, yedekli geri alma
# Bagimli Oldugu Katman: Tool / Config

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet('Status','Enable','Disable')]
    [string]$Mode,
    [string]$IpAddress
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$base = Join-Path $root 'compose.yaml'
$overlay = Join-Path $root 'compose.lan.yaml'
$data = Join-Path $root 'storage/Tachidesk'
$config = Join-Path $data 'server.conf'

function Assert-Plain([string]$Path) {
    if (Test-Path -LiteralPath $Path) {
        $item = Get-Item -LiteralPath $Path -Force
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw 'LAN_REPARSE_POINT_NOT_ALLOWED'
        }
    }
}
function Read-Setting([string]$Text,[string]$Name) {
    $expr = '(?m)^\s*["'']?' + [regex]::Escape($Name) + '["'']?\s*[:=]\s*(?<v>[^#\r\n]+)'
    $found = [regex]::Matches($Text,$expr)
    if ($found.Count -ne 1) { throw "LAN_CONFIG_SETTING_MISSING_OR_AMBIGUOUS: $Name" }
    return $found[0].Groups['v'].Value.Trim().Trim('"',"'")
}
function Assert-Auth {
    if (-not (Test-Path -LiteralPath $config -PathType Leaf)) { throw 'LAN_SERVER_CONFIG_MISSING' }
    Assert-Plain $config
    $text = Get-Content -LiteralPath $config -Raw -Encoding UTF8
    $auth = (Read-Setting $text 'server.authMode').ToUpperInvariant()
    if ($auth -notin @('UI_LOGIN','SIMPLE_LOGIN','BASIC_AUTH')) { throw 'LAN_AUTH_REQUIRED' }
    if ([string]::IsNullOrWhiteSpace((Read-Setting $text 'server.authUsername')) -or
        [string]::IsNullOrWhiteSpace((Read-Setting $text 'server.authPassword'))) {
        throw 'LAN_USERNAME_AND_PASSWORD_REQUIRED'
    }
    $listen = [regex]::Match($text,'(?m)^\s*server\.ip\s*[:=]\s*["'']?([0-9.]+)')
    if ($listen.Success -and $listen.Groups[1].Value -ne '0.0.0.0') {
        throw 'LAN_LISTEN_IP_MUST_BE_0_0_0_0'
    }
}
function Assert-LocalIPv4([string]$Ip) {
    $parsed = [Net.IPAddress]::None
    if (-not [Net.IPAddress]::TryParse($Ip,[ref]$parsed) -or
        $parsed.AddressFamily -ne [Net.Sockets.AddressFamily]::InterNetwork) {
        throw 'LAN_INVALID_IPV4'
    }
    $o = $parsed.GetAddressBytes()
    $private = $o[0] -eq 10 -or
        ($o[0] -eq 172 -and $o[1] -ge 16 -and $o[1] -le 31) -or
        ($o[0] -eq 192 -and $o[1] -eq 168)
    if (-not $private -or $o[3] -in @(0,255)) { throw 'LAN_PRIVATE_IPV4_REQUIRED' }
    $matches = @(Get-NetIPAddress -AddressFamily IPv4 -IPAddress $parsed.ToString() -ErrorAction SilentlyContinue)
    if ($matches.Count -eq 0) { throw 'LAN_IP_NOT_ASSIGNED_TO_WINDOWS_HOST' }
    return $parsed.ToString()
}
function Managed-IP {
    if (-not (Test-Path -LiteralPath $overlay -PathType Leaf)) { return $null }
    Assert-Plain $overlay
    $text = (Get-Content -LiteralPath $overlay -Raw -Encoding UTF8).Replace([Environment]::NewLine,[string][char]10).Trim()
    if ($text -notmatch '(?s)^# BURAK_MANGA_LAN_V1\nservices:\n  suwayomi:\n    ports:\n      - "((?:\d{1,3}\.){3}\d{1,3}):4567:4567"$') {
        throw 'LAN_OVERLAY_NOT_MANAGED_BY_LAUNCHER'
    }
    return $Matches[1]
}
function Compose([string[]]$Options) {
    & docker compose -f $base @Options
    if ($LASTEXITCODE -ne 0) { throw 'LAN_DOCKER_COMPOSE_FAILED' }
}
if (-not (Test-Path -LiteralPath $base -PathType Leaf) -or
    -not (Test-Path -LiteralPath $data -PathType Container)) { throw 'LAN_DEPLOYMENT_MISSING' }
foreach ($item in @($root,$base,(Join-Path $root 'storage'),$data)) { Assert-Plain $item }
if (-not (Get-Content -LiteralPath $base -Raw).Contains('"127.0.0.1:4567:4567"')) {
    throw 'LAN_UNEXPECTED_BASE_PORT'
}
$old = Managed-IP
if ($Mode -eq 'Status') {
    if ($old) { Write-Host ('Telefon baglantisi: http://' + $old + ':4567') }
    else { Write-Host 'Telefon erisimi kapali; PC http://127.0.0.1:4567' }
    return
}
if ($Mode -eq 'Enable') {
    Assert-Auth
    $ip = Assert-LocalIPv4 $IpAddress
    if ($old) {
        if ($old -eq $ip) { Write-Host ('Telefon erisimi zaten etkin: http://' + $ip + ':4567'); return }
        throw 'LAN_ALREADY_ENABLED_DISABLE_FIRST'
    }
    $temp = Join-Path $data ('lan-stage-' + [guid]::NewGuid().ToString('N') + '.yaml')
    try {
        $lines = @('# BURAK_MANGA_LAN_V1','services:','  suwayomi:','    ports:',('      - "'+$ip+':4567:4567"'))
        [IO.File]::WriteAllText($temp,($lines -join [string][char]10) + [string][char]10,[Text.UTF8Encoding]::new($false))
        Compose -Options @('-f',$temp,'config','--quiet')
        Move-Item -LiteralPath $temp -Destination $overlay
        try {
            Compose -Options @('-f',$overlay,'up','-d','--no-deps','suwayomi')
        } catch {
            $failed = Join-Path $data ('lan-failed-'+[guid]::NewGuid().ToString('N')+'.yaml')
            Move-Item -LiteralPath $overlay -Destination $failed
            try { Compose -Options @('up','-d','--no-deps','suwayomi') }
            catch { throw 'LAN_ENABLE_AND_ROLLBACK_FAILED' }
            throw 'LAN_ENABLE_FAILED_RESTORED_LOCALHOST'
        }
        Write-Host ('Telefon baglantisi acildi: http://' + $ip + ':4567')
        Write-Host 'Firewall veya Wi-Fi izolasyonu engellerse ayrica kontrol edilmeli.'
    } finally {
        if (Test-Path -LiteralPath $temp -PathType Leaf) { Remove-Item -LiteralPath $temp -Force }
    }
}
if ($Mode -eq 'Disable') {
    if (-not $old) { Write-Host 'Telefon erisimi zaten kapali'; return }
    $backup = Join-Path $data ('lan-disabled-'+[guid]::NewGuid().ToString('N')+'.yaml')
    Move-Item -LiteralPath $overlay -Destination $backup
    try {
        Compose -Options @('up','-d','--no-deps','suwayomi')
    } catch {
        Move-Item -LiteralPath $backup -Destination $overlay
        try { Compose -Options @('-f',$overlay,'up','-d','--no-deps','suwayomi') }
        catch { throw 'LAN_DISABLE_AND_ROLLBACK_FAILED' }
        throw 'LAN_DISABLE_FAILED_RESTORED'
    }
    Write-Host 'Telefon erisimi kapatildi; localhost korundu.'
}
