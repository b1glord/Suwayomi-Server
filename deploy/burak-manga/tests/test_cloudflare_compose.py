# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_cloudflare_compose.py
# Amac: Suwayomi/Byparr Compose sozlesmesini offline dogrulamak
# Modul - Python
# Version: 0.3.0
# Aciklama: Docker engine gerektirmeyen Compose JSON static contract testi
# Bagimli Oldugu Katman: Testing
import json
import os
from pathlib import Path
import shutil
import subprocess
import unittest


COMPOSE = Path(__file__).resolve().parents[1] / "compose.yaml"


class CloudflareComposeTest(unittest.TestCase):
    def compose_config(self, official=False):
        environment = os.environ.copy()
        environment.pop("COMPOSE_PROFILES", None)
        environment["MANGA_CF_HELPER_URL"] = (
            "http://flaresolverr-official:8191" if official
            else "http://flaresolverr:8191"
        )
        args = ["docker", "compose", "-f", str(COMPOSE)]
        if official:
            args += ["--profile", "official-flaresolverr"]
        args += ["config", "--format", "json"]
        result = subprocess.run(args, cwd=COMPOSE.parent, env=environment,
                                capture_output=True, text=True, timeout=30, check=True)
        return json.loads(result.stdout)

    @unittest.skipUnless(shutil.which("docker"), "docker compose CLI is unavailable")
    def test_cloudflare_helper_is_private_and_server_is_persistent(self):
        config = self.compose_config()
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
        self.assertNotIn("flaresolverr-official", config["services"])

    @unittest.skipUnless(shutil.which("docker"), "docker compose CLI is unavailable")
    def test_official_helper_is_opt_in_private_and_pinned(self):
        config = self.compose_config(official=True)
        services = config["services"]
        helper = services["flaresolverr-official"]
        server = services["suwayomi"]
        self.assertEqual(helper["image"], "ghcr.io/flaresolverr/flaresolverr:v3.5.2")
        self.assertEqual(server["environment"]["FLARESOLVERR_URL"],
                         "http://flaresolverr-official:8191")
        self.assertFalse(helper.get("ports"))
        self.assertEqual(helper["profiles"], ["official-flaresolverr"])
        self.assertTrue(helper.get("tmpfs"))
        self.assertIn("flaresolverr", services)
        self.assertEqual(server["ports"][0]["host_ip"], "127.0.0.1")

    def test_source_has_no_dangerous_volume_operations(self):
        contents = COMPOSE.read_text(encoding="utf-8")
        self.assertNotIn("docker compose down -v", contents)
        self.assertIn("./storage/Tachidesk:", contents)


if __name__ == "__main__":
    unittest.main()
