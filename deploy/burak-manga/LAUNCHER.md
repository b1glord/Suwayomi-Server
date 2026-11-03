<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/LAUNCHER.md
Amac: Rust Windows launcher kurulumu, kullanim ve test sinirlari
Modul - Markdown
Version: 0.3.0
Aciklama: Tasinabilir test EXE, Compose guvenligi ve katalog islemleri
Bagimli Oldugu Katman: Documentation
-->

# Burak Manga Launcher v0.3.0 (Rust)

Windows 10/11 x64 icin yerel Win32 arayuzu. PowerShell, WPF, WebView
ve Rust SDK kurulumu gerektirmez. Docker Desktop Linux engine gereklidir.

## Indirme ve baslatma

GitHub Actions sayfasinda Burak Manga - Rust launcher isimli is akisini acin.
Basarili bir calismanin Artifacts bolumundeki
burak-manga-rust-windows-x64 dosyasini indirin.

ZIP icindeki BurakMangaLauncher.exe dosyasini
Suwayomi-Server/deploy/burak-manga klasorundeki compose.yaml dosyasinin
yanina kopyalayin. EXE'ye cift tiklayin; alternatif olarak Launcher.cmd.
ZIP icindeki README ve SHA256SUMS dosyalarini da kontrol edebilirsiniz.

EXE'nin kurulumu sabit disk yoluna bagli degildir. Yanlis klasore
konuldugunda compose.yaml bulunmadigi icin gorunur hata mesaji verir.

## Islevler

- Baslat: Docker Engine ve Compose konfigrasyonunu kontrol eder, Suwayomi'yi baslatir.
- Durdur: Sadece ilgili Suwayomi servisini durdurur.
- Yeniden Baslat: Sadece ilgili Suwayomi servisini yeniden baslatir.
- Durum ve gunluk: Container durumunu ve son 100 gunluk satirini gosterir.
- WebUI: Yerel http://127.0.0.1:4567 adresini acar.
- Katalog: Rüya Manga ve Merlin Scans imzali katalog URL'sini kopyalar ya da acar.

Katalog: https://raw.githubusercontent.com/b1glord/Suwayomi-Server-Extensions/repo/index.pb

Katalog Suwayomi WebUI uzanti depo ayarlarina bir kez elle eklenir.
Baslatici APK/uzanti kurulumu yapmaz, manga indirmez.

## Guvenlik

Storage/Tachidesk silinmez. down -v, image prune, secret islemi
ve GitHub release olusturma yoktur. Docker komutlari shell'e
karistirilmadan sabit argument dizileriyle yurutulur. Islem
suresi sinirlidir ve GUI'yi bloke etmemek icin worker thread kullanilir.
EXE kapatilsa da sunucu acik kalabilir.

## Dogrulama

Windows Actions testi Cargo Clippy, Rust unit test ve release build,
portable layout smoke, ZIP girdileri ve SHA256 kontrol dosyasini kapsar.
Windows GUI tiklama, yerel Docker ve uzanti calisma testleri yapilmadiysa
basarili olarak raporlanmaz.

Kaynak: deploy/burak-manga/launcher-rust.
Gelistirme: cargo fmt, cargo clippy, cargo test, cargo build --release.
CLI kontrol: BurakMangaLauncher.exe --check-layout (Docker baslatmaz).

Eski WPF launcher.ps1 dosyasi bu giris noktasindan artik calistirilmaz.
