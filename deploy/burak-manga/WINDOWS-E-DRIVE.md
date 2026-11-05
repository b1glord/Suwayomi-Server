<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/WINDOWS-E-DRIVE.md
Amac: Suwayomi fork'unu E diski altina tasinabilir bicimde kurma kilavuzu
Modul - Markdown
Version: 0.1.0
Aciklama: E surucusunde guvenli klonlama ve Docker Desktop baslatma
Bagimli Oldugu Katman: Documentation
-->
# Windows Docker Desktop: E:\Docker kurulumu

Bu ornekte repo E:\Docker\Suwayomi-Server altinda tutulur. Docker Compose
konumuna gore goreli volume kullandigi icin manga veri klasoru de
E:\Docker\Suwayomi-Server\deploy\burak-manga\storage\Tachidesk
olacaktir. Bu yol sabit kodlanmaz; kurulum betigi kendi script
konumundan cozer.

## Ilk kurulum (PowerShell)

Docker Desktop Linux container motoru ve Git onceden kurulu/calisir olmali.
E: sifreli veya harici diskse once bagli/acik oldugunu kontrol edin.

~~~powershell
$Root = 'E:\Docker'
if (-not (Test-Path -LiteralPath 'E:\' -PathType Container)) {
    throw 'E_DRIVE_NOT_READY: Unlock or mount drive E: first.'
}
New-Item -ItemType Directory -Path $Root -Force | Out-Null
$Repo = Join-Path $Root 'Suwayomi-Server'
if (-not (Test-Path -LiteralPath $Repo)) {
    git clone --branch develop/burak-manga --single-branch https://github.com/b1glord/Suwayomi-Server.git $Repo
    if ($LASTEXITCODE -ne 0) { throw 'GIT_CLONE_FAILED' }
} elseif (-not (Test-Path -LiteralPath (Join-Path $Repo '.git'))) {
    throw 'TARGET_DIRECTORY_ALREADY_EXISTS_AND_IS_NOT_GIT'
}
$Branch = (& git -C $Repo branch --show-current).Trim()
if ($LASTEXITCODE -ne 0 -or $Branch -ne 'develop/burak-manga') {
    throw 'WRONG_BRANCH: Existing checkout left unchanged.'
}
$Starter = Join-Path $Repo 'deploy/burak-manga/scripts/start-windows.ps1'
if (-not (Test-Path -LiteralPath $Starter)) {
    throw 'STARTER_NOT_FOUND: Inspect the existing checkout before updating.'
}
& $Starter
~~~

Ardindan sadece yerel bilgisayardan http://localhost:4567 adresini acin.

## Komutlar

~~~powershell
& $Starter -CheckOnly  # Docker/Compose kontrolu, servisi baslatmaz
& $Starter -WhatIf     # Degisiklik yapmadan niyeti yazdirir
& $Starter            # Var olan imajla baslatir, yoksa indirir
& $Starter -PullImage # Yeni stable imaji kontrollu olarak indirip baslatir

docker compose -f (Join-Path $Repo 'deploy/burak-manga/compose.yaml') ps
docker compose -f (Join-Path $Repo 'deploy/burak-manga/compose.yaml') logs --tail 50
~~~

Eski klonlar otomatik silinmez, resetlenmez, branch degistirilmez veya pull
yapilmaz. Mevcut repoyu guncellemeden once git status ve mevcut branch'i
kontrol edin. Temiz branch'te git pull --ff-only origin develop/burak-manga
komutuyla guncelleyin.

**Veri kaybi uyarisi:** Manga verileri storage/Tachidesk altinda kalici
bind mount ile tutulur; Docker'i kapatip yedek almadan klasoru silmeyin.
Yedek almadan docker compose down -v komutunu kullanmayin.

Docker Desktop'in kendi imaj/VM disk dosyalari E: uzerinde olmak zorunda degildir.
Bu kurulumun garantisi, manga uygulamasinin bind-mounted **kalici veri** klasorunun
E: uzerinde olmasidir. Docker Desktop'in image storage konumu farkli bir ayardir.

Bu sistem izinli yerel arsivleme icindir. Ucuncu taraf manga sitelerine erisim,
uyumlu kaynak uzantisi ve indirme izni ayrica dogrulanmalidir.
