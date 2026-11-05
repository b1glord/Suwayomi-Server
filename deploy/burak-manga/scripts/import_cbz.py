# Dosya Yolu: REPO_ROOT/deploy/burak-manga/scripts/import_cbz.py
# Amac: Izinli yerel sayfa gorsellerini Suwayomi CBZ arsivine donusturur
# Modul - Python
# Version: 0.1.0
# Aciklama: CWD bagimsiz, mevcut arsivi ezmeyen cevrimdisi CBZ aktarimi
# Bagimli Oldugu Katman: Tool
"""Import permitted local page images into Suwayomi's Local source as CBZ."""

import argparse
import re
import zipfile
from pathlib import Path

DEFAULT_LOCAL_ROOT = Path(__file__).resolve().parents[1] / "storage" / "Tachidesk" / "local"
IMAGE_SUFFIXES = {".jpg", ".jpeg", ".png", ".webp", ".gif", ".avif"}
CHUNKS = re.compile(r"(\d+)")


def safe_component(value):
    if not isinstance(value, str) or not value.strip() or value.strip() in {".", ".."}:
        raise ValueError("Series/chapter title cannot be empty or dot-only")
    if any(ch in value for ch in ('/', '\\', '\0', ':', '*', '?', '"', '<', '>', '|')):
        raise ValueError("Series/chapter title contains a path separator or forbidden character")
    if value.endswith(" ") or value.endswith(".") or value != value.strip():
        raise ValueError("Series/chapter title cannot have trailing spaces or periods")
    if len(value) > 140:
        raise ValueError("Series/chapter title is too long")
    return value


def natural_key(path):
    return [int(part) if part.isdigit() else part.casefold() for part in CHUNKS.split(path.name)]


def prepare_import(input_dir, local_root, series, chapter):
    series = safe_component(series)
    chapter = safe_component(chapter)
    input_dir = Path(input_dir)
    if input_dir.is_symlink():
        raise ValueError("Input directory must not be a symlink")
    if not input_dir.is_dir():
        raise ValueError("Input directory does not exist")
    pages = sorted((p for p in input_dir.iterdir() if p.suffix.lower() in IMAGE_SUFFIXES), key=natural_key)
    if not pages:
        raise ValueError("No supported images in the input directory")
    if any(not p.is_file() or p.is_symlink() for p in pages):
        raise ValueError("Symlinked or invalid page files are not supported")
    root = Path(local_root)
    if root.is_symlink() or (root.exists() and not root.is_dir()):
        raise ValueError("Local root cannot be a symlink or a file")
    series_dir = root / series
    if series_dir.is_symlink():
        raise ValueError("Series directory must not be a symlink")
    destination = series_dir / f"{chapter}.cbz"
    if destination.exists() or destination.is_symlink():
        raise FileExistsError(f"Archive already exists: {destination}")
    return pages, destination


def import_cbz(input_dir, local_root, series, chapter, dry_run=False):
    pages, destination = prepare_import(input_dir, local_root, series, chapter)
    if dry_run:
        return destination, len(pages)
    destination.parent.mkdir(parents=True, exist_ok=True)
    temp_file = destination.with_suffix(".cbz.partial")
    if temp_file.exists() or temp_file.is_symlink():
        raise FileExistsError(f"Temporary archive already exists: {temp_file}")
    try:
        with zipfile.ZipFile(temp_file, "x", compression=zipfile.ZIP_DEFLATED) as archive:
            for idx, page in enumerate(pages, start=1):
                archive.write(page, arcname=f"{idx:05d}{page.suffix.lower()}")
        if destination.exists() or destination.is_symlink():
            raise FileExistsError(f"Archive already exists: {destination}")
        temp_file.rename(destination)
    finally:
        if temp_file.exists():
            temp_file.unlink()
    return destination, len(pages)


def main():
    parser = argparse.ArgumentParser(description="Create a CBZ in Suwayomi Local source.")
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--series", required=True)
    parser.add_argument("--chapter", required=True)
    parser.add_argument("--local-root", type=Path, default=DEFAULT_LOCAL_ROOT)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    destination, count = import_cbz(args.input_dir, args.local_root, args.series, args.chapter, args.dry_run)
    prefix = "DRY RUN" if args.dry_run else "CREATED"
    print(f"{prefix}: {destination} ({count} pages)")


if __name__ == "__main__":
    main()
