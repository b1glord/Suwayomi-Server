<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/README.md
Amac: Yerel manga arsivini kurma ve test etme kilavuzu
Modul - Markdown
Version: 0.5.1
Aciklama: Docker Compose, CBZ import, kaynak listesi ve sinirlar
Bagimli Oldugu Katman: Documentation
-->
# Burak Manga Archive - local integration v0.5.1

This is a standalone local integration within the Suwayomi-Server fork.
Its Compose service uses the official upstream stable image; code changes
in this fork do NOT change that image. The host port is bound to localhost.

## Windows: E:\Docker\Suwayomi-Server install

For the complete E: drive setup, data location and safe Windows commands, see
[WINDOWS-E-DRIVE.md](WINDOWS-E-DRIVE.md). No fixed drive is hardcoded in
the runtime Compose configuration or startup script.

Run from the repository root on a machine with Docker Desktop.
The first start downloads Byparr, official FlareSolverr and the small Python
routing service. They run inside this same Compose stack. Only Suwayomi
port 4567 is published to localhost; helper APIs are internal.

```powershell
docker compose -f deploy/burak-manga/compose.yaml config --quiet
docker compose -f deploy/burak-manga/compose.yaml pull
docker compose -f deploy/burak-manga/compose.yaml up -d
docker compose -f deploy/burak-manga/compose.yaml ps
```

Open http://localhost:4567. Do not expose this unauthenticated local
service to LAN/Internet. Persistent files are in
deploy/burak-manga/storage/Tachidesk (excluded from Git). Stop the
container before backing up the folder. Never use down -v for cleanup
without an independent backup.

## Existing extension candidates

- Ruya Manga: Keiyoushi source code targets www.ruyamanga2.com.
- Merlin Scans: Keiyoushi source code targets merlintoon.com, NOT the
  requested merlinscans.org URL. Domain compatibility is unknown.

Public extension catalog from the Keiyoushi source README:
https://github.com/keiyoushi/extensions/raw/repo/index.pb

Add only trusted extension catalogs through Suwayomi's Extensions settings.
These source references are metadata, not a downloader; no source or
copyrighted content is bundled. Real site fetches have NOT been tested.

## Import legally held local page images

```powershell
python deploy/burak-manga/scripts/import_cbz.py --input-dir "C:\my-pages" --series "Martial Peak" --chapter "Bolum 93" --dry-run
python deploy/burak-manga/scripts/import_cbz.py --input-dir "C:\my-pages" --series "Martial Peak" --chapter "Bolum 93"
```

Replace the example input directory with an existing folder containing
images you are allowed to store. The script reads LOCAL images only, and
writes deploy/burak-manga/storage/Tachidesk/local/Martial Peak/Bolum 93.cbz.
Refresh the Local source chapter list in Suwayomi if necessary. Existing
chapters are not intentionally overwritten.

## Local validation (run before GitHub CI)

```powershell
python deploy/burak-manga/scripts/validate_sources.py
python -m unittest discover -s deploy/burak-manga/tests -v
docker compose -f deploy/burak-manga/compose.yaml config --quiet
```

Source candidates live in deploy/burak-manga/config/sources.json.
Only extension source code presence was established. Actual site access,
site permissions, extension runtime, and server GUI smoke checks remain
unverified. The Docker CLI is needed for Compose validation and
container testing. Use sources responsibly and respect the sites' terms and content rights.

For the architecture decision, see docs/decisions/0001-suwayomi.md in
this directory.

## Windows GUI launcher (Rust)

Windows icin native Rust EXE kurulumu ve test artifacti adimlari:
[LAUNCHER.md](LAUNCHER.md).
Mevcut Docker Compose ve storage verileri korunur; WPF/PowerShell arayuzu kullanilmaz.

## Automatic Cloudflare helper failover

Byparr includes an upstream Docker healthcheck with a 15-minute
interval. It may show `health: starting` for a long time even when the
API is already serving requests, especially if the first check ran
during GeoIP initialization. The deployment overrides this with a
30-second check against `/health`, a 3-minute startup grace period
and three retries. This improves state reporting but does not
guarantee source-site downloads. Other services are unchanged.

The single Docker Compose file always starts four services:

- suwayomi (official stable server; only localhost:4567 is published)
- cf-router (internal Python adapter on port 8191; no host port)
- flaresolverr (Byparr official image :latest; first-choice helper)
- flaresolverr-official (FlareSolverr v3.5.2; second-choice helper)

Suwayomi sees one FlareSolverr-compatible URL:
http://cf-router:8191. The router receives POST /v1 requests, asks
Byparr first and tries official FlareSolverr on Byparr transport errors,
timeouts, unsuccessful API status, or a GET challenge response without
the expected cf_clearance cookie. At most TWO attempts are made per
GET request. An unsuccessful pair returns an error to Suwayomi; there
is no indefinite retry loop.

For compatibility with Suwayomi, the server timeout is 150 seconds
and each backend is limited to 60 seconds of solver processing plus
network overhead. The second helper gets a temporary browser session,
because persistent session names are backend-local. POST requests and
session commands are not automatically replayed, to avoid duplicate
actions. All four services can run simultaneously, but only the helper
selected by the router handles each attempt. A healthy router does not
mean that a particular source site is accessible.

### Resolve missing Byparr image tag

Do not use ghcr.io/thephaseless/byparr:v3.0.4: that image tag was
never confirmed as published. Byparr's official Compose uses the
ghcr.io/thephaseless/byparr:latest tag. The tag can advance between
deployments; CI checks the referenced remote container manifests on
each Compose change. FlareSolverr remains pinned to v3.5.2.

When an unavailable image prevents a Compose startup, Docker may report
other images as "Interrupted" or "No such image". Those messages do not
prove the secondary images are unavailable: the original pull stopped
before they finished. Fix the first failing reference, then pull again.

### Update existing Windows Docker installation

From the project root, provided git status shows no uncommitted changes:

~~~powershell
Set-Location 'E:\Docker\Suwayomi-Server'
git status --short
git pull --ff-only origin develop/burak-manga
docker compose -f deploy/burak-manga/compose.yaml config --quiet
docker compose -f deploy/burak-manga/compose.yaml pull
docker compose -f deploy/burak-manga/compose.yaml up -d
docker compose -f deploy/burak-manga/compose.yaml ps
docker compose -f deploy/burak-manga/compose.yaml logs --tail 80 cf-router
~~~

The prior manually selected helper .env values
(COMPOSE_PROFILES=official-flaresolverr and MANGA_CF_HELPER_URL=...)
are now ignored by the Compose file and can be removed from local
.env after checking for other unrelated settings. Do not overwrite
or automatically delete the file.

The stored manga and download queue stay under the same bind mount:
deploy/burak-manga/storage/Tachidesk. No schema migration or volume
pruning is involved. Never use docker compose down -v or delete
the storage folder as part of this update.

To inspect actual routing results:

~~~powershell
docker compose -f deploy/burak-manga/compose.yaml logs --tail 120 cf-router
~~~

The router logs only backend outcomes, such as provider=Byparr outcome=ok
or provider=FlareSolverr outcome=ok. It deliberately does not record
source URLs, browser cookies, request bodies or upstream exceptions.
The health route /health only means the router process is running.

Test with ONE manga chapter before retrying a failed backlog. Neither
helper guarantees Cloudflare verification will succeed on every source.
The combined fallback can take up to around two minutes when both
helpers time out. A WebUI/docker end-to-end acceptance test is required
on the actual Windows host.

### Rollback

If the updated stack fails to start, restore the last previously working
compose.yaml from version control, leaving storage/Tachidesk unchanged,
then execute docker compose up -d. Do not reset uncommitted files or
delete existing manga. For a quick diagnostic without any download
retries, use docker compose ps and docker compose logs.

## Turkce WebUI test kurulumu (geri alinabilir)

Rust Launcher v0.8.0 artik `Turkce Paketi Al`, `Turkce WebUI Kur`
ve `Turkceyi Geri Al` dugmelerini sunuyor. Windows kullanicilari
asagidaki komutlari manuel yazmak yerine GUI'yi kullanabilir.
[LAUNCHER.md](LAUNCHER.md) icindeki Turkce WebUI dugmeleri bolumune
bakin. Asagidaki PowerShell komutlari Docker auth veya GUI sorunlarinda
manuel fallback'tir.


Suwayomi-WebUI upstream'in Turkce katalogu su anda normal dil listesinde
yer almiyor. Ayrica hazirlanmis Turkce preview paketinde dil kaydi etkin
ve bos 176 arayuz metnine ceviri taslagi ekleniyor. Bazi etiketler
halen Ingilizce kalabilir; tam ceviri iddiasi yoktur.

1. [Turkce WebUI CI test paketi](https://github.com/b1glord/Suwayomi-Server/actions/workflows/suwayomi_webui_tr_preview.yml)
   uzerinden en yeni basarili `suwayomi-webui-tr-preview` artifact'ini
   indirin. ZIP'i acin; icindeki `Suwayomi-WebUI-TR-preview.zip` ve
   `SHA256SUMS.txt` dosyalarini ayni klasore koyun.
2. Suwayomi WebUI > Settings > WebUI ayarlarinda **WebUI Flavor = Custom**
   secin ve kaydedin. Indirmeler aktifse once duraklatin. Bu test kurulumunu
   uygun bakim aninda yapin: yalnizca suwayomi servisi durdurulacak.
3. Repository kokunde asagidaki komutlarla indirdiginiz gercek ZIP yolunu
   belirtin. `Check` dosyalari degistirmeden paket dogrular.
4. `Install` sadece durmus Suwayomi'ye izin verir, WebUI ayarinin
   Custom oldugunu denetler, `storage/Tachidesk/webUI` klasorunu
   `webui-tr-backups` altinda saklar, Turkce arayuzu kurar.
   Sonra yalnizca Suwayomi servisini baslatin ve Settings > General
   (veya Appearance) > App language alanindan Turkceyi secin.

~~~powershell
Set-Location 'E:\Docker\Suwayomi-Server'
$Compose = 'deploy/burak-manga/compose.yaml'
$Zip = 'C:\Path\To\Suwayomi-WebUI-TR-preview.zip'
$Installer = 'deploy/burak-manga/scripts/install-turkish-webui.ps1'
& $Installer -Mode Check -ZipPath $Zip
docker compose -f $Compose stop suwayomi
& $Installer -Mode Install -ZipPath $Zip
docker compose -f $Compose up -d --no-deps suwayomi
~~~

**Rollback:** Indirmeleri duraklatin, `docker compose -f $Compose stop
suwayomi`, sonra `& $Installer -Mode Restore`, ardindan yalnizca
`suwayomi` servisini `up -d --no-deps` ile baslatin. Eski UI dosyalari
ve Turkce dosyalarin geri alinan kopyasi yedek klasorunde tutulur.
Gerekiyorsa Settings > WebUI > Flavor ayarini normal WebUI'ye cevirin.

**Onemli sinir:** Test preview, upstream WebUI'nin belirli commit'inden
uretilmistir; kullanicinin canli server/WebUI entegrasyonu yerel Docker'da
ayrica dogrulanmalidir. ZIP dosyasini guvenilir GitHub Actions sayfasindan
edinin. ZIP checksum dosyasi paketin bozulmadigini denetler, tek basina
yayinci kimligini dogrulamaz. Suwayomi indirme kuyrugunu yeniden baslatmadan
once tek bolum ile deneyin. Script DB, downloaded manga, Compose ve
Cloudflare servislerini degistirmez.
