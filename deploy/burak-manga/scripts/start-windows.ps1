# Dosya Yolu: REPO_ROOT/deploy/burak-manga/scripts/start-windows.ps1
# Amac: Windows Docker Desktop uzerinde yerel manga arsivini guvenle baslatir
# Modul - PowerShell
# Version: 0.1.0
# Aciklama: Proje konumuna gore Compose ve veri yolunu cozer, var olan veriyi korur
# Bagimli Oldugu Katman: Tool
[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'Low')]
param(
    [switch] $CheckOnly,
    [switch] $PullImage
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$DeploymentRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$ComposeFile = Join-Path $DeploymentRoot 'compose.yaml'
$StorageParent = Join-Path $DeploymentRoot 'storage'
$StorageRoot = Join-Path $StorageParent 'Tachidesk'

if (-not (Test-Path -LiteralPath $ComposeFile -PathType Leaf)) {
    throw "COMPOSE_NOT_FOUND: $ComposeFile"
}

# Avoid accidentally following a storage symlink/junction outside the selected checkout.
foreach ($Candidate in @($StorageParent, $StorageRoot)) {
    if (Test-Path -LiteralPath $Candidate) {
        $Item = Get-Item -LiteralPath $Candidate -Force
        if (-not $Item.PSIsContainer) {
            throw "STORAGE_PATH_IS_NOT_A_DIRECTORY: $Candidate"
        }
        if (($Item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "STORAGE_REPARSE_POINT_NOT_ALLOWED: $Candidate"
        }
    }
}

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    throw 'DOCKER_NOT_FOUND: Install or start Docker Desktop first.'
}

& docker compose version
if ($LASTEXITCODE -ne 0) {
    throw 'DOCKER_COMPOSE_UNAVAILABLE'
}
& docker info --format '{{.ServerVersion}}'
if ($LASTEXITCODE -ne 0) {
    throw 'DOCKER_DAEMON_UNAVAILABLE: Start Docker Desktop and ensure its Linux engine is running.'
}
& docker compose -f $ComposeFile config --quiet
if ($LASTEXITCODE -ne 0) {
    throw 'INVALID_COMPOSE_CONFIG'
}

Write-Host "Compose file: $ComposeFile"
Write-Host "Local archive directory: $StorageRoot"
Write-Host 'Web UI (localhost only): http://localhost:4567'

if ($CheckOnly) {
    Write-Host 'CHECK_ONLY: Docker is ready, Compose is valid, nothing started.'
    return
}

if ($PSCmdlet.ShouldProcess($StorageRoot, 'Create persistent storage directory (if absent)')) {
    New-Item -ItemType Directory -Path $StorageRoot -Force | Out-Null
}

if ($PullImage -and $PSCmdlet.ShouldProcess('suwayomi:stable', 'Pull latest stable Docker image')) {
    & docker compose -f $ComposeFile pull
    if ($LASTEXITCODE -ne 0) {
        throw 'IMAGE_PULL_FAILED'
    }
}

if ($PSCmdlet.ShouldProcess('suwayomi', 'Start local manga server with Docker Compose')) {
    & docker compose -f $ComposeFile up -d
    if ($LASTEXITCODE -ne 0) {
        throw 'DOCKER_COMPOSE_START_FAILED'
    }
    & docker compose -f $ComposeFile ps
    if ($LASTEXITCODE -ne 0) {
        throw 'DOCKER_COMPOSE_STATUS_FAILED'
    }
}
