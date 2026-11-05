<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/LAUNCHER.md
Amac: Rust Windows launcher kurulumu, kullanim ve test sinirlari
Modul - Markdown
Version: 0.9.0
Aciklama: Tasinabilir test EXE, Compose guvenligi ve katalog islemleri
Bagimli Oldugu Katman: Documentation
-->

# Burak Manga Launcher v0.9.0 (Rust)

Windows 10/11 x64 icin yerel Win32 arayuzu. WPF, WebView ve Rust SDK
kurulumu gerektirmez. Turkce WebUI kurulum islemleri icin Windows ile
gelen PowerShell kullanilir; GUI Rust ile yazilmistir. Docker Desktop
Linux engine gereklidir.

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
- Git Pull: Secili fork'un develop/burak-manga dalini yalnizca fast-forward ile gunceller; once onay alir.
- WebUI: Yerel http://127.0.0.1:4567 adresini acar.
- Telefon Erisimi Ac: Windows ozel ag IPv4 adresini dogrular,
  auth etkinse ek LAN portunu acip yalniz suwayomi servisini yeniler.
- Telefon Erisimi Kapat: Yedek alarak LAN portunu kapatir.
- Turkce Paketi Al: En son basarili Turkce WebUI test paketini bulmak icin GitHub Actions sayfasini acar.
- Turkce WebUI Kur: Paket ZIP'ini Windows dosya secicisinden alir, hash/on kosul dogrular; yedekli kurar.
- Turkceyi Geri Al: En son orijinal WebUI yedegini geri yukler. Her iki islem de onay gerektirir.
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

## Git Pull dugmesi (v0.7.0)

Git Pull, Windows Git CLI'yi arka plan worker'inda kullanir; PowerShell
ve shell scripti gerektirmez. Dugmeye basildiginda kullanicidan onay alinir.
Klasor, sabit disk harfi veya CWD tahminiyle degil EXE'nin yanindaki
compose.yaml dosyasinin ust dizinlerinden cozulur.

Guncellemeden once Git'in gercek repo root'u, aktif branch,
`origin` deposunun GitHub adresi ve `git status --porcelain` cikisi
dogrulanir. Beklenen repository `b1glord/Suwayomi-Server`; branch
`develop/burak-manga`. Degisiklik varsa `git pull --ff-only origin
develop/burak-manga` **calistirilmaz**. Branch degistirme, `reset`,
`stash`, `checkout`, kuvvetli pull veya otomatik merge yoktur.

Git CLI erisilemez ya da yetkilendirme gerekirse guvenli sekilde hata
doner. Git Pull sadece kaynak kodu gunceller: Docker image'ini cekmez,
Compose konteynerlerini baslatmaz, Rust EXE'nin kendisini degistirmez.
Yeni kod derleme artifakti GitHub Actions uzerinden ayrica edinilmelidir.

Yerel launcher EXE'si `.gitignore` kapsaminda tutularak kaydedilmemis
manga verileri ve diger kullanici dosyalari korunur.

## Turkce WebUI butonlari (v0.8.0)

Rust Launcher Turkce WebUI'yi kontrol edilen bir Windows PowerShell alt
isleminde yonetir; arayuz Rust/Win32 kalir. EXE'nin yeni surumunu GitHub
Actions **Burak Manga - Rust launcher** test ZIP'inden alip compose.yaml
yanina kopyalayin. Git Pull ile
deploy/burak-manga/scripts/install-turkish-webui.ps1 dosyasinin
guncel oldugunu dogrulayin. Script bulunamazsa islem veri degistirmeden
hata verir.

1. WebUI > Settings > WebUI > Flavor alanini **Custom** secip kaydedin.
   Launcher bu ayari kendiliginden degistirmez.
2. Launcher'da **Turkce Paketi Al** ile
   Suwayomi - Turkish WebUI preview workflow sayfasini acin. En yeni
   basarili workflow'un Artifacts bolumundeki paketi indirin. Dis artifact
   ZIP'ini acin. **Suwayomi-WebUI-TR-preview.zip** ve
   **SHA256SUMS.txt** dosyalari ayni klasorde bulunmalidir.
3. **Turkce WebUI Kur** dugmesine basin, onay verin ve ic ZIP dosyasini
   Windows dosya secicisinden secin. On kontrolu basarisiz olursa
   calisan konteynerlere dokunulmaz.
4. Basariliysa sadece suwayomi durur, script mevcut webUI klasorunu
   webui-tr-backups altinda saklar, test paketini kurar, suwayomi
   yeniden baslatilir (onceden calisiyorsa). cf-router/Byparr/FlareSolverr
   yeniden baslatilmaz. Suwayomi baslamadiysa kapali birakilir.
5. WebUI sayfasinda App Language alaninda Turkceyi secin. Dil
   secilmisse ama menu degismiyorsa Ctrl+F5 ile yenileyin.

**Turkceyi Geri Al:** Son yedek okunabilirligini onceden denetler;
onaydan sonra ayni hizmet-durdur/geri-yukle/islet akisini izler.
Orijinal UI yedeklenmeden silinmez. Turkce WebUI dosyalari da
webui-tr-backups altina tasinir. Gerekirse WebUI > Flavor = WebUI
ayarina geri donulabilir.

Yalnizca pinli upstream'den derlenen test arayuzudur, production
release degildir. Turkce eksik metinler kalabilir. CI artifact'i sureli
saklanir; kimlik dogrulamasi gereken GitHub Actions artifact'ini
Launcher kullanici yerine token ile indirmez. ZIP'in SHA256 kontrolu
saglanir; yayinlayici kimligi icin GitHub run kaynagini kullanin.

Her iki islemden once Launcher stopDownloader GraphQL mutasyonunu
dogrular. Bu adim API yetkilendirmesi nedeniyle basarisizsa Suwayomi
durdurulmaz; WebUI'den manuel duraklatin, gerekirse README'deki
manuel kurulum yolunu kullanin. **Suwayomi yeniden baslarken indirme
kuyrugu otomatik devam edebilir**; kurulum sonrasinda Download
durumunu kontrol edin. Queue/DB/CBZ otomatik silinmez.

Gercek Windows Docker WebUI geri alma islemi kullanici ortaminda
ayrica test edilmelidir; CI birim/portable layout testleri bu testi
kapsamaz.

## Telefon baglantisi (v0.9.0)

**Varsayilan KAPALI.** Suwayomi'nin server.ip ayari 0.0.0.0
kalir; Windows LAN IP'sini server.ip alanina yazmayin.
Orijinal Compose localhost binding'i korunur.

1. Suwayomi > Settings > Authentication icinde UI Login (veya diger
   desteklenen auth kiplerinden biri) acin; kullanici adi ve parola
   belirleyin. Giris korumasi olmadan LAN erisimi acilmaz.
2. Windows PowerShell'de ipconfig ile aktif Wi-Fi/Ethernet IPv4
   adresini ogrenin.
3. Launcher > Telefon IP kutusuna adresi girip Telefon Erisimi Ac
   butonuna basin; uyariyi onaylayin.
4. Launcher IP'nin RFC1918 ozel adres oldugunu ve Windows hostuna
   atandigini kontrol eder, compose.yaml dosyasini degistirmez.
   Git'in yok saydigi compose.lan.yaml olusur. Docker Compose
   dogrulamasindan sonra sadece suwayomi servisi yeniden olusturulur.
5. Telefon ayni Wi-Fi agindayken http://WINDOWS-IP:4567 adresini acin.
   Bilgisayarda http://127.0.0.1:4567 kullanilmaya devam eder.
6. Windows Defender Firewall engellerse ancak gerekli oldugunda
   Private network ve LocalSubnet ile sinirli TCP 4567 kuralini
   elle olusturun; yonetici izni gerekebilir. Modem port yonlendirmesi
   yapmayin.

**Telefon Erisimi Kapat** LAN ek portunu kaldirir; onceki
LAN Compose dosyasi veri klasorunde yedeklenir. Silinmez.
Kurulum veya geri alma Docker'da basarisizsa onceki duruma
donulmeye calisilir. Yabanci/degistirilmis ek Compose dosyasi
algilanirsa islem reddedilir.

**Guvenlik:** HTTP sifreli degildir; parola ve oturum belirtecleri
agda gorunebilir. Guvenilmeyen Wi-Fi'de kullanmayin.
Internet/mobil veri erisimi icin VPN ve TLS ayri gerekir.
Launcher kendiliginden Windows Firewall, modem veya auth
ayarlari degistirmez.

**Onemli:** Yeniden olusturma sonrasinda Suwayomi indirme kuyrugu
otomatik devam edebilir. Mevcut manga dosyalari, veritabani ve
diger 3 servis degistirilmez. Launcher tum Compose komutlarina
yonetilen ek dosyayi dahil eder; elle docker compose -f compose.yaml
komutu kullanilirsa LAN portu kaybolabilir. Manuel durumda ek
-f compose.lan.yaml parametresini de belirtin.

**Test siniri:** PowerShell mock-Docker testleri, Rust arguman
ve duz metin override testleri, Compose static merge kontrolu CI'da
calistirilir. Gercek telefon/Windows Firewall ve canli Docker
baglantisi kullanici makinesinde ayrica dogrulanmalidir.
