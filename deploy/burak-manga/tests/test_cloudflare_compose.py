# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_cloudflare_compose.py
# Amac: Suwayomi/Byparr Compose sozlesmesini offline dogrulamak
# Modul - Python
# Version: 0.2.0
# Aciklama: Docker engine gerektirmeyen Compose JSON static contract testi
# Bagimli Oldugu Katman: Testing
import json
from pathlib import Path
import shutil
import subprocess
import unittest


COMPOSE = Path(__file__).resolve().parents[1] / "compose.yaml"


class CloudflareComposeTest(unittest.TestCase):
    @unittest.skipUnless(shutil.which("docker"), "docker compose CLI is unavailable")
    def test_cloudflare_helper_is_private_and_server_is_persistent(self):
        proc = subprocess.run(
            ["docker", "compose", "-f", str(COMPOSE), "config", "--format", "json"],
            capture_output=True, text=True, timeout=30, check=True,
        )
        config = json.loads(proc.stdout)
        server = config["services"]["suwayomi"]
        helper = config["services"]["flaresolverr"]
        environment = server["environment"]
        self.assertEqual(environment["FLARESOLVERR_ENABLED"], "true")
        self.assertEqual(environment["FLARESOLVERR_URL"], "http://flaresolverr:8191")
        self.assertEqual(environment["FLARESOLVERR_TIMEOUT"], "60")
        self.assertIn("flaresolverr", server["depends_on"])
        self.assertEqual(helper["image"], "ghcr.io/thephaseless/byparr:v3.0.4")
        self.assertFalse(helper.get("ports"), "Helper must have no published host ports")
        self.assertTrue(any(
            mount["target"] == "/home/suwayomi/.local/share/Tachidesk"
            and mount["type"] == "bind"
            for mount in server["volumes"]
        ), "Preserve persistent manga bind mount")
        self.assertEqual(server["ports"][0]["host_ip"], "127.0.0.1")

    def test_source_has_no_dangerous_volume_operations(self):
        contents = COMPOSE.read_text(encoding="utf-8")
        self.assertNotIn("docker compose down -v", contents)
        self.assertIn("./storage/Tachidesk:", contents)


if __name__ == "__main__":
    unittest.main()
