# Dosya Yolu: REPO_ROOT/deploy/burak-manga/tests/test_cloudflare_router.py
# Amac: Byparr -> FlareSolverr otomatik gecis kontrati
# Modul - Python
# Version: 0.4.0
# Aciklama: Dis agsiz, deterministik API mock testleri
# Bagimli Oldugu Katman: Testing
import importlib.util
from pathlib import Path
import unittest

path = Path(__file__).resolve().parents[1] / "cloudflare-router" / "router.py"
spec = importlib.util.spec_from_file_location("cf_router", path)
router = importlib.util.module_from_spec(spec)
spec.loader.exec_module(router)


def ok():
    return {
        "status": "ok", "message": "",
        "solution": {
            "url": "https://example.invalid", "status": 200, "userAgent": "Chrome",
            "cookies": [{"name": "cf_clearance", "value": "secret"}],
        },
        "startTimestamp": 1, "endTimestamp": 2, "version": "3.5.2",
    }


class CloudflareRouterTest(unittest.TestCase):
    def test_primary_success(self):
        calls = []
        def backend(url, payload, headers):
            calls.append((url, payload, headers))
            return ok()
        status, result, name = router.route_payload(
            {"cmd": "request.get", "maxTimeout": 150000},
            {"X-Proxy-Server": "socks5://proxy:1080"}, backend,
        )
        self.assertEqual((status, name, len(calls)), (200, "Byparr", 1))
        self.assertEqual(result["solution"]["cookies"][0]["value"], "secret")
        self.assertEqual(calls[0][1]["maxTimeout"], 60000)
        self.assertEqual(calls[0][2]["X-Proxy-Server"], "socks5://proxy:1080")

    def test_unreachable_primary_tries_secondary_without_reusing_session(self):
        calls = []
        def backend(url, payload, headers):
            calls.append(payload)
            if len(calls) == 1:
                raise router.BackendFailure("network")
            return ok()
        status, _, name = router.route_payload({
            "cmd": "request.get", "session": "suwayomi",
            "session_ttl_minutes": 15,
        }, backend=backend)
        self.assertEqual((status, name), (200, "FlareSolverr"))
        self.assertEqual(calls[0]["session"], "suwayomi")
        self.assertNotIn("session", calls[1])
        self.assertNotIn("session_ttl_minutes", calls[1])

    def test_http_ok_but_no_cloudflare_clearance_triggers_fallback(self):
        calls = []
        def backend(url, payload, headers):
            calls.append(url)
            if len(calls) == 1:
                result = ok()
                result["solution"]["cookies"] = []
                return result
            return ok()
        status, _, name = router.route_payload({"cmd": "request.get"}, backend=backend)
        self.assertEqual((status, name, len(calls)), (200, "FlareSolverr", 2))

    def test_error_status_triggers_fallback(self):
        calls = []
        def backend(url, payload, headers):
            calls.append(url)
            return {"status": "error", "message": "timeout"} if len(calls) == 1 else ok()
        status, _, provider = router.route_payload({"cmd": "request.get"}, backend=backend)
        self.assertEqual((status, provider), (200, "FlareSolverr"))

    def test_both_failed_at_most_two_attempts_and_hide_sensitive_data(self):
        calls = []
        def backend(url, payload, headers):
            calls.append(url)
            raise router.BackendFailure("secret-user-site")
        status, body, name = router.route_payload({"cmd": "request.get"}, backend=backend)
        self.assertEqual((status, name, len(calls)), (502, None, 2))
        self.assertNotIn("secret", repr(body))

    def test_non_idempotent_post_is_never_replayed(self):
        calls = []
        def backend(url, payload, headers):
            calls.append(url)
            raise router.BackendFailure("network")
        status, _, _ = router.route_payload({"cmd": "request.post"}, backend=backend)
        self.assertEqual((status, len(calls)), (502, 1))

    def test_session_command_goes_to_primary(self):
        calls = []
        def backend(url, payload, headers):
            calls.append(url)
            return {"status": "ok", "session": "suwayomi"}
        status, _, provider = router.route_payload({"cmd": "sessions.create"}, backend=backend)
        self.assertEqual((status, provider, len(calls)), (200, "Byparr", 1))

    def test_invalid_body_fails_closed(self):
        for payload in (None, [], {}, {"cmd": None}):
            with self.subTest(payload=payload):
                status, _, _ = router.route_payload(payload)
                self.assertEqual(status, 400)


if __name__ == "__main__":
    unittest.main()
