<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/README.md
Amac: Yerel manga arsivini kurma ve test etme kilavuzu
Modul - Markdown
Version: 0.2.0
Aciklama: Docker Compose, CBZ import, kaynak listesi ve sinirlar
Bagimli Oldugu Katman: Documentation
-->
# Burak Manga Archive - local integration v0.1.0

This is a standalone local integration within the Suwayomi-Server fork.
Its Compose service uses the official upstream stable image; code changes
in this fork do NOT change that image. The host port is bound to localhost.

## Windows: E:\Docker\Suwayomi-Server install

For the complete E: drive setup, data location and safe Windows commands, see
[WINDOWS-E-DRIVE.md](WINDOWS-E-DRIVE.md). No fixed drive is hardcoded in
the runtime Compose configuration or startup script.

Run from the repository root on a machine with Docker Desktop.
The first start also pulls a separate Byparr Cloudflare helper container.
Its API is reachable from Suwayomi over the internal Compose network only.
No extra host port is exposed.

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

## Cloudflare helper integration (Byparr)

Compose now starts the upstream-supported FlareSolverr-compatible Byparr
service alongside Suwayomi. It uses a pinned published version
`ghcr.io/thephaseless/byparr:v3.0.4` and URL `http://flaresolverr:8191`.
Only Suwayomi's WebUI port 4567 is mapped to loopback; the helper has
**no published host port**.

The Docker service is named `flaresolverr` for API compatibility,
although the image runs Byparr. Suwayomi uses the official environment
keys `FLARESOLVERR_ENABLED`, `FLARESOLVERR_URL` and
`FLARESOLVERR_TIMEOUT`. Environment variables override persisted server
settings when Suwayomi starts; edit Compose for permanent adjustments.

To apply on an existing Windows installation **without deleting downloads**:

```powershell
cd E:\Docker\Suwayomi-Server
git pull --ff-only origin develop/burak-manga
docker compose -f deploy/burak-manga/compose.yaml config --quiet
docker compose -f deploy/burak-manga/compose.yaml pull flaresolverr
docker compose -f deploy/burak-manga/compose.yaml up -d
docker compose -f deploy/burak-manga/compose.yaml ps
docker compose -f deploy/burak-manga/compose.yaml logs --tail 80 flaresolverr
```

If `git status --short` reports local changes, do not reset the working
tree. Review and preserve them before pull. **Do not use `down -v`**.
The existing `storage/Tachidesk` bind mount is unchanged.
First start can take time while the browser image is downloaded.

Test only one chapter on Merlin Scans or OkuToon before retrying the
large failed queue. A running helper does **not** guarantee that a given
website's challenge will be solved; Cloudflare can impose additional
checks or limitations.

If the helper has issues, inspect the service logs. To disable it, set
`FLARESOLVERR_ENABLED: "false"` in Compose and recreate only Suwayomi
with `docker compose -f deploy/burak-manga/compose.yaml up -d suwayomi`.
Do not manually delete manga data. Note that the helper container may
remain running until stopped with `docker compose stop flaresolverr`.

The upstream documentation is here:
https://github.com/Suwayomi/Suwayomi-Server/blob/master/docs/Configuring-Suwayomi%E2%80%90Server.md
