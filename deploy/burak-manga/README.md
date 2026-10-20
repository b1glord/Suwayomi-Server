<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/README.md
Amac: Yerel manga arsivini kurma ve test etme kilavuzu
Modul - Markdown
Version: 0.1.0
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

Run from the repository root on a machine with Docker Desktop:

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
container testing. Use permitted content only; no DRM/CAPTCHA bypass.

For the architecture decision, see docs/decisions/0001-suwayomi.md in
this directory.
