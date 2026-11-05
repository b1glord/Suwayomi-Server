# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_cloudflare_compose.py
# Amac: Iki backend ve router icin tek Compose kontratini kontrol eder
# Modul - Python
# Version: 0.4.0
# Aciklama: API URL, servis bagimliligi, host port ve veri koruma testleri
# Bagimli Oldugu Katman: Testing
import json
import os
from pathlib import Path
import shutil
import subprocess
import unittest

COMPOSE = Path(__file__).resolve().parents[1] / "compose.yaml"


class CloudflareComposeTest(unittest.TestCase):
    @unittest.skipUnless(shutil.which("docker"), "docker compose CLI is unavailable")
    def test_both_backends_are_always_in_default_compose(self):
        env = os.environ.copy()
        env.pop("COMPOSE_PROFILES", None)
        env["MANGA_CF_HELPER_URL"] = "http://flaresolverr-official:8191"
        output = subprocess.run(
            ["docker", "compose", "-f", str(COMPOSE), "config", "--format", "json"],
            cwd=COMPOSE.parent, env=env,
            capture_output=True, text=True, check=True, timeout=30,
        ).stdout
        services = json.loads(output)["services"]
        self.assertEqual(set(services), {
            "suwayomi", "cf-router", "flaresolverr", "flaresolverr-official",
        })
        server = services["suwayomi"]
        gateway = services["cf-router"]
        self.assertEqual(server["environment"]["FLARESOLVERR_ENABLED"], "true")
        self.assertEqual(server["environment"]["FLARESOLVERR_URL"], "http://cf-router:8191")
        self.assertEqual(server["environment"]["FLARESOLVERR_TIMEOUT"], "150")
        self.assertEqual(gateway["environment"]["CF_PRIMARY"], "http://flaresolverr:8191")
        self.assertEqual(gateway["environment"]["CF_SECONDARY"], "http://flaresolverr-official:8191")
        self.assertIn("cf-router", server["depends_on"])
        self.assertIn("flaresolverr", gateway["depends_on"])
        self.assertIn("flaresolverr-official", gateway["depends_on"])
        self.assertTrue(gateway["read_only"])
        self.assertEqual(services["flaresolverr"]["image"], "ghcr.io/thephaseless/byparr:v3.0.4")
        self.assertEqual(services["flaresolverr-official"]["image"], "ghcr.io/flaresolverr/flaresolverr:v3.5.2")
        for name in ("cf-router", "flaresolverr", "flaresolverr-official"):
            self.assertFalse(services[name].get("ports"))
        self.assertEqual(server["ports"][0]["host_ip"], "127.0.0.1")
        self.assertTrue(any(
            v["type"] == "bind" and v["target"] == "/home/suwayomi/.local/share/Tachidesk"
            for v in server["volumes"]
        ))

    def test_compose_does_not_use_old_profile_or_dynamic_helper_url(self):
        source = COMPOSE.read_text(encoding="utf-8")
        self.assertNotIn("profiles:", source)
        self.assertNotIn("MANGA_CF_HELPER_URL", source)
        self.assertIn("./storage/Tachidesk:/home/suwayomi/.local/share/Tachidesk", source)


if __name__ == "__main__":
    unittest.main()
