# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_archive.py
# Amac: CBZ aktarimi ve kaynak metadata sozlesmesini sinar
# Modul - Python
# Version: 0.1.0
# Aciklama: Agsiz, gecici dosya sistemiyle unit testler
# Bagimli Oldugu Katman: Testing
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from import_cbz import import_cbz, safe_component
from validate_sources import validate_registry


class RegistryTests(unittest.TestCase):
    def test_real_registry_is_valid(self):
        registry = json.loads((ROOT / "config" / "sources.json").read_text(encoding="utf-8"))
        self.assertEqual(validate_registry(registry), 2)

    def test_reject_cross_host_urls(self):
        registry = {"schema_version": 1, "sources": [{"id": "demo", "name": "Demo",
                    "base_url": "https://example.com/", "example_url": "https://evil.test/",
                    "status": "access_unverified"}]}
        with self.assertRaises(ValueError):
            validate_registry(registry)

    def test_reject_duplicate_identifiers(self):
        item = {"id": "demo", "name": "Demo", "base_url": "https://example.com/",
                "example_url": "https://example.com/manga/", "status": "access_unverified"}
        with self.assertRaises(ValueError):
            validate_registry({"schema_version": 1, "sources": [item, item]})


class LocalImportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.inputs = self.base / "pages"
        self.inputs.mkdir()
        self.root = self.base / "local"

    def add_images(self):
        for name in ["10.jpg", "2.jpg", "1.jpg"]:
            (self.inputs / name).write_bytes(b"TEST IMAGE")

    def test_archive_page_sort_and_location(self):
        self.add_images()
        output, count = import_cbz(self.inputs, self.root, "Martial Peak", "Bolum 93")
        self.assertEqual(count, 3)
        self.assertEqual(output, self.root / "Martial Peak" / "Bolum 93.cbz")
        with zipfile.ZipFile(output) as z:
            self.assertEqual(z.namelist(), ["00001.jpg", "00002.jpg", "00003.jpg"])

    def test_dry_run_does_not_write(self):
        self.add_images()
        output, _ = import_cbz(self.inputs, self.root, "Manga", "Chapter 1", dry_run=True)
        self.assertFalse(output.exists())
        self.assertFalse(self.root.exists())

    def test_does_not_overwrite(self):
        self.add_images()
        import_cbz(self.inputs, self.root, "Manga", "Chapter 1")
        with self.assertRaises(FileExistsError):
            import_cbz(self.inputs, self.root, "Manga", "Chapter 1")

    def test_invalid_title(self):
        for candidate in ["../hello", "..", "C:\\x", "bad/name", "a|b", "bad."]:
            with self.subTest(candidate=candidate), self.assertRaises(ValueError):
                safe_component(candidate)

    def test_empty_input(self):
        with self.assertRaises(ValueError):
            import_cbz(self.inputs, self.root, "Manga", "Chapter 1")

    def test_symlinked_page_is_rejected(self):
        outside = self.base / "outside.jpg"
        outside.write_bytes(b"test")
        try:
            (self.inputs / "page1.jpg").symlink_to(outside)
        except (NotImplementedError, OSError):
            self.skipTest("symlinks unavailable")
        with self.assertRaises(ValueError):
            import_cbz(self.inputs, self.root, "Manga", "Chapter 1")


if __name__ == "__main__":
    unittest.main()
