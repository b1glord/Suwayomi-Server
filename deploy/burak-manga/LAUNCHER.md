<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/LAUNCHER.md
Amac: Rust Windows launcher kurulumu, kullanim ve test sinirlari
Modul - Markdown
Version: 0.6.0
Aciklama: Tasinabilir test EXE, Compose guvenligi ve katalog islemleri
Bagimli Oldugu Katman: Documentation
-->

# Burak Manga Launcher v0.6.0 (Rust)

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

- Baslat: Docker Engine ve Compose kontrolunden sonra Suwayomi ile uc yardimci servisini baslatir.
- Durdur: Suwayomi, cf-router, Byparr ve FlareSolverr servislerini birlikte durdurur.
- Yeniden Baslat: Ayni dort servisi birlikte yeniden baslatir.
- Durum ve gunluk: Container durumunu ve son 100 gunluk satirini gosterir.
- WebUI: Yerel http://127.0.0.1:4567 adresini acar.
- Katalog: Rüya Manga ve Merlin Scans imzali katalog URL'sini kopyalar ya da acar.
- Arsiv: storage/Tachidesk/downloads altindaki CBZ dosyalarini, CBZ bulunan manga klasoru sayisini ve tum indirme dosyalarinin kapladigi boyutu gosterir.
- Arsiv klasorunu ac: Mevcut indirme dizinini Windows Explorer'da acar, veri olusturmaz veya silmez.
- OkuToon: Martial Peak sayfasini tarayicida acar. Bu, Suwayomi'ye uzanti kurmaz.
- Indirme tanilama: Son 10 dakikanin Docker gunluklerinden Cloudflare hata satirlari,
  baglanti sizintisi uyarilari ve bulunan son kuyruk sayilarini gosterir.
- Indirmeleri Duraklat: Suwayomi'nin yerel GraphQL stopDownloader mutasyonunu
  kullanir. Kuyruk silme, tamamlama veya otomatik tekrar deneme yapmaz.
- Otomatik Koruma: Varsayilan KAPALI; kullanici acarsa son 10 dakikada en az
  uc Cloudflare hata satiri ve gunlukte son bilinen aktif indirme varsa bir
  kez stopDownloader cagirir. Kaynak ayirt etmez ve tekrar acilis yapmaz.
  Durdurma basarisizsa WebUI uzerinden manuel kontrol gerekir.

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

## OkuToon kurulumu (Suwayomi)

OkuToon modulu Keiyoushi extensions-source icinde mevcut, ancak bu projenin imzali
iki uzantili katalogunda yok. Suwayomi WebUI Extension stores ekranina ayrica
Keiyoushi katalogunu ekleyebilirsiniz:

    https://github.com/keiyoushi/extensions/raw/repo/index.pb

Browse > Extensions icinde OkuToon uzantisini bulun, kurun ve kaynaklarda
Martial Peak arayin. Yalniz indirme izniniz bulunan icerigi arsivleyin.
Yerel calisma, bolum erisimi ve indirme haklari ayri dogrulanmalidir.

Arsiv sayaci CBZ uzantili dosyalari bolum olarak sayar; dosya uzantisi tek basina indirme tamamlandi veya ZIP saglam demek degildir. Manga sayisi,
CBZ bulunan ust klasorlerin sayisidir; kaynak klasor yapisi degisirse
manga sayisi yaklasik olabilir. Boyut, indirme dizinindeki tum duz dosyalari
(kismi indirmeler dahil) kapsar. Klasor yoksa hata gorunur. Tarama junction
ve simgesel baglantilari takip etmez. Scan butonu GUI'yi bloke etmez.

## Tanimli sinirlar

Tanilama log satirlarini sayar; Cloudflare hata satiri sayisi benzersiz
bolum veya icerik sayisi degildir. Son 10 dakikada durum satiri yoksa aktif
indirme varsayilmaz. Otomatik Koruma global stopDownloader kullandigindan
etkinse o anda calisan diger kaynaklari da duraklatabilir. Bu nedenle
kullanici acmadan koruma devreye girmez. Docker disinda yetki/esas sunucu
yapilandirmasi gerektiren ortamlarda GraphQL islemi reddedilebilir; servis
veya guvenlik ayarlari otomatik olarak degistirilmez.

Docker gunlukleri buyuk oldugunda stdout ve stderr ayri worker'larda
akis halinde okunur, bellek kullanimi sinirlanir ve timeout korunur.
Yasam dongusu testi gercek docker suwayomi durumu ve WebUI API'sine
kullanici bilgisayarinda baglanarak ayrica dogrulanmalidir.

## Otomatik Cloudflare yedeklemesi (v0.6.0)

Rust Launcher, cf-router, Byparr ve FlareSolverr servislerini tek Compose
stack'i icinde yonetir. Baslat, Durdur, Yeniden Baslat, Durum ve Gunlukler
islemleri dort servis icindir. Manuel .env secimi veya Compose profile
gerekmemektedir. Baslatilan stack'te once Byparr, basarisiz GET
dogrulamasinda ikinci olarak FlareSolverr denenir. Iki servis de basarisiz
olursa hata doner; otomatik sonsuz tekrar yoktur.

Indirme Tanilama ekrani, son on dakikalik Docker gunluklerinde
provider=Byparr / provider=FlareSolverr basari sayilarini ayrica gosterir.
Bu rakamlar manga bolumu veya Cloudflare asildi garantisi degildir.
API guvenlik politikalari ve kaynak icerik kullanim kosullari korunur.

Baslatici herhangi bir manga dosyasini, kayitli indirme kuyrugunu veya
kalici storage klasorunu silmez. Gercek Cloudflare erisimi yerel Docker
ve kaynak site ile ayrica sinanmalidir.
