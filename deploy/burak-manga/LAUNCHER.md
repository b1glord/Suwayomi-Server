<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/LAUNCHER.md
Amac: Windows manga baslaticisini kullanma ve test etme
Modul - Markdown
Version: 0.2.0
Aciklama: Docker GUI islemleri, imzali katalog, guvenlik sinirlari
Bagimli Oldugu Katman: Documentation
-->

# Burak Manga Launcher v0.2.0

Windows 10/11 icin tasinabilir bir PowerShell 5.1 + WPF arayuzudur.
Harici EXE kurulumuna ihtiyac yoktur. Mevcut Suwayomi Docker projesinde
develop/burak-manga dalini kullanin.

## Kullanim

Temiz calisma agacinda git pull --ff-only origin develop/burak-manga
komutundan sonra deploy/burak-manga/Launcher.cmd dosyasina cift tiklayin.
Baslatici hangi diskte veya klasorde bulunursa bulunsun kendi script
konumuna gore Compose ve kalici veri klasorunu bulur.

- Baslat: Mevcut start-windows.ps1 ile dogrulama ve Docker Compose up -d.
- Durdur: Yalniz suwayomi servisine docker compose stop.
- Yeniden baslat: Yalniz suwayomi servisine docker compose restart.
- Durum: Docker Compose ps --status running --services.
- Gunlukler: Son 100 Docker Compose gunluk satiri.
- WebUI: Yerel http://127.0.0.1:4567 adresini tarayicida acar.
- Katalog: Imzali uzanti katalog adresini kopyalar veya HTTP HEAD ile kontrol eder.

Katalog:
https://raw.githubusercontent.com/b1glord/Suwayomi-Server-Extensions/repo/index.pb

Rüya Manga ve Merlin Scans icin bu katalog adresini Suwayomi WebUI sunucu
ayarlarindaki uzanti depolarina bir kez ekleyin; uzanti kurulumunu WebUI'den
gerceklestirin. Launcher sessiz APK kurulumu veya otomatik indirme yapmaz.

## Sinirlar ve guvenlik

- Docker Desktop Linux motoru calisir durumda olmalidir.
- Yonetici yetkisi istemez. Git, branch veya server ayarlarini degistirmez.
- Storage/Tachidesk verisini silmez; down -v komutu bulunmaz.
- GitHub Secrets, imza anahtari ve parolalara erismez.
- GUI penceresini kapatmak calisan Docker container'ini durdurmaz.
- HEAD 200 sonucunu gercek uzanti kurulumu/bolum okuma basarisi saymayin.
- Docker gunlukleri hassas URL ya da bilgiler icerebilir; paylasmadan once inceleyin.

## Windows arayuz dogrulamasi

PowerShell 5.1 ile:

    powershell.exe -NoProfile -STA -File .\deploy\burak-manga\scripts\launcher.ps1 -ValidateLayout

Bu test UI nesnelerini kurar, Docker'a dokunmaz. Windows GitHub Actions
is akisi ayni testi ve PowerShell syntax kontrolunu yapar. Bu kontroller
canli Docker, uzanti kurulum ve bolum okuma testlerinin yerine gecmez.

Acma sorunu olursa PowerShell'den GUI betigini calistirarak hatayi gorun.
