# Dosya Yolu: REPO_ROOT/deploy/burak-manga/scripts/validate_sources.py
# Amac: Manga kaynak adaylarini ag erisimi olmadan dogrular
# Modul - Python
# Version: 0.1.0
# Aciklama: Kaynak URL, benzersiz kimlik ve metadata kontrati
# Bagimli Oldugu Katman: Tool
"""Offline validator for the source candidate registry (not an extension)."""

import argparse
import json
import re
from pathlib import Path
from urllib.parse import urlsplit

DEFAULT_FILE = Path(__file__).resolve().parents[1] / "config" / "sources.json"
VALID_STATUSES = {"extension_unverified", "access_unverified", "extension_code_found", "verified", "disabled"}
ID_PATTERN = re.compile(r"^[a-z][a-z0-9_-]{1,47}$")


def validate_registry(payload):
    if not isinstance(payload, dict) or payload.get("schema_version") != 1:
        raise ValueError("schema_version must be 1")
    sources = payload.get("sources")
    if not isinstance(sources, list):
        raise ValueError("sources must be an array")
    ids = set()
    for item in sources:
        if not isinstance(item, dict):
            raise ValueError("source must be an object")
        ident = item.get("id")
        if not isinstance(ident, str) or not ID_PATTERN.fullmatch(ident):
            raise ValueError("invalid source id")
        if ident in ids:
            raise ValueError("duplicate source id")
        ids.add(ident)
        if not isinstance(item.get("name"), str) or not item["name"].strip():
            raise ValueError(f"{ident}: name required")
        base_url = item.get("base_url")
        example_url = item.get("example_url")
        if not isinstance(base_url, str) or not isinstance(example_url, str):
            raise ValueError(f"{ident}: URLs must be strings")
        base = urlsplit(base_url)
        example = urlsplit(example_url)
        if (base.scheme != "https" or not base.hostname or base.username or base.password
                or base.port or base.query or base.fragment or base.path not in {"", "/"}):
            raise ValueError(f"{ident}: invalid base URL")
        if (example.scheme != "https" or not example.hostname or example.hostname != base.hostname
                or example.username or example.password or example.port or example.fragment):
            raise ValueError(f"{ident}: example URL must belong to the base host")
        if item.get("status") not in VALID_STATUSES:
            raise ValueError(f"{ident}: unknown status")
        if item.get("status") == "extension_code_found":
            extension_base_url = item.get("extension_base_url")
            extension_code = item.get("extension_code")
            if not isinstance(extension_base_url, str) or urlsplit(extension_base_url).scheme != "https":
                raise ValueError(f"{ident}: valid extension_base_url required")
            if not isinstance(extension_code, str) or not extension_code.startswith("https://github.com/"):
                raise ValueError(f"{ident}: GitHub source reference required")
    return len(ids)


def main():
    parser = argparse.ArgumentParser(description="Validate source metadata (no requests).")
    parser.add_argument("--file", type=Path, default=DEFAULT_FILE)
    args = parser.parse_args()
    data = json.loads(args.file.read_text(encoding="utf-8"))
    count = validate_registry(data)
    print(f"OK: {count} source candidate(s); no network requests were made.")


if __name__ == "__main__":
    main()
