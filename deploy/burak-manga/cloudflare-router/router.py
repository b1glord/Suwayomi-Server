# Dosya Yolu: REPO_ROOT/deploy/burak-manga/cloudflare-router/router.py
# Amac: Suwayomi API isteklerini Byparr, sonra FlareSolverr ile cozer
# Modul - Python
# Version: 0.4.0
# Aciklama: Tek URL, en fazla iki deneme, hassas veri loglamaz
# Bagimli Oldugu Katman: Adapter
"""FlareSolverr-compatible internal router; only GET challenges are replayed.

Session state belongs to individual backends. On fallback, use an ephemeral
session on FlareSolverr rather than reusing the primary's session name.
"""
import json
import logging
import os
import socket
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib import error, request

BIND_HOST = "0.0.0.0"
PORT = 8191
PRIMARY = os.getenv("CF_PRIMARY", "http://flaresolverr:8191").rstrip("/")
SECONDARY = os.getenv("CF_SECONDARY", "http://flaresolverr-official:8191").rstrip("/")
BACKEND_TIMEOUT_SECONDS = 68
BACKEND_SOLVE_TIMEOUT_MS = 60_000
MAX_REQUEST_BYTES = 128 * 1024
MAX_RESPONSE_BYTES = 24 * 1024 * 1024
INFLIGHT = threading.BoundedSemaphore(value=2)
LOGGER = logging.getLogger("cf-router")


class BackendFailure(Exception):
    """Hide upstream exceptions to avoid leaking source URLs and cookies."""


def fetch_backend(url, payload, passthrough_headers):
    data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if "X-Proxy-Server" in passthrough_headers:
        headers["X-Proxy-Server"] = passthrough_headers["X-Proxy-Server"]
    req = request.Request(url + "/v1", data=data, headers=headers, method="POST")
    try:
        with request.urlopen(req, timeout=BACKEND_TIMEOUT_SECONDS) as response:
            if response.status != 200:
                raise BackendFailure("bad_http_status")
            raw = response.read(MAX_RESPONSE_BYTES + 1)
    except (error.HTTPError, error.URLError, socket.timeout, TimeoutError, OSError) as exc:
        raise BackendFailure("transport") from exc
    if len(raw) > MAX_RESPONSE_BYTES:
        raise BackendFailure("response_too_large")
    try:
        parsed = json.loads(raw)
    except (ValueError, UnicodeDecodeError) as exc:
        raise BackendFailure("invalid_json") from exc
    if not isinstance(parsed, dict):
        raise BackendFailure("invalid_shape")
    return parsed


def validate_result(response, command):
    if response.get("status") != "ok":
        return False
    if command != "request.get":
        return True
    solution = response.get("solution")
    if not isinstance(solution, dict):
        return False
    if not isinstance(solution.get("userAgent"), str) or not solution["userAgent"]:
        return False
    if not isinstance(solution.get("cookies"), list):
        return False
    if not isinstance(solution.get("status"), int) or not 200 <= solution["status"] < 300:
        return False
    return any(
        isinstance(cookie, dict) and cookie.get("name") == "cf_clearance"
        and isinstance(cookie.get("value"), str) and bool(cookie["value"])
        for cookie in solution["cookies"]
    )


def route_payload(payload, passthrough_headers=None, backend=fetch_backend):
    """Return (HTTP status, JSON-compatible response, provider)."""
    if not isinstance(payload, dict):
        return 400, {"status": "error", "message": "Invalid JSON request"}, None
    command = payload.get("cmd")
    if not isinstance(command, str) or not command:
        return 400, {"status": "error", "message": "Missing cmd"}, None
    passthrough_headers = passthrough_headers or {}
    safe_payload = dict(payload)
    if command in ("request.get", "request.post"):
        try:
            requested_timeout = int(payload.get("maxTimeout", BACKEND_SOLVE_TIMEOUT_MS))
        except (ValueError, TypeError):
            requested_timeout = BACKEND_SOLVE_TIMEOUT_MS
        safe_payload["maxTimeout"] = min(
            max(requested_timeout, 1_000), BACKEND_SOLVE_TIMEOUT_MS
        )
    backends = [("Byparr", PRIMARY)]
    if command == "request.get":
        backends.append(("FlareSolverr", SECONDARY))
    for label, endpoint in backends:
        current = dict(safe_payload)
        if label == "FlareSolverr":
            current.pop("session", None)
            current.pop("session_ttl_minutes", None)
        try:
            response = backend(endpoint, current, passthrough_headers)
        except BackendFailure:
            LOGGER.warning("provider=%s outcome=failed", label)
            continue
        if validate_result(response, command):
            LOGGER.info("provider=%s outcome=ok", label)
            return 200, response, label
        LOGGER.warning("provider=%s outcome=unresolved", label)
    return 502, {
        "status": "error",
        "message": "Cloudflare helpers did not resolve the challenge",
    }, None


class Handler(BaseHTTPRequestHandler):
    server_version = "CloudflareRouter/0.4"
    sys_version = ""

    def log_message(self, *_args):
        # No source URLs, headers, cookies or request body in logs.
        pass

    def send_json(self, status, data):
        encoded = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(encoded)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(encoded)

    def do_GET(self):
        if self.path == "/health":
            self.send_json(200, {"status": "ok", "service": "cf-router"})
        else:
            self.send_json(404, {"status": "error", "message": "Not found"})

    def do_POST(self):
        if self.path != "/v1":
            self.send_json(404, {"status": "error", "message": "Not found"})
            return
        try:
            size = int(self.headers.get("Content-Length", ""))
        except ValueError:
            self.send_json(411, {"status": "error", "message": "Content-Length required"})
            return
        if size <= 0 or size > MAX_REQUEST_BYTES:
            self.send_json(413, {"status": "error", "message": "Request size outside limit"})
            return
        if not INFLIGHT.acquire(blocking=False):
            self.send_json(503, {"status": "error", "message": "Router is busy"})
            return
        try:
            try:
                payload = json.loads(self.rfile.read(size))
            except (ValueError, UnicodeDecodeError):
                self.send_json(400, {"status": "error", "message": "Invalid JSON"})
                return
            headers = {}
            if self.headers.get("X-Proxy-Server"):
                headers["X-Proxy-Server"] = self.headers["X-Proxy-Server"]
            status, result, _provider = route_payload(payload, headers)
            self.send_json(status, result)
        finally:
            INFLIGHT.release()


def main():
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(message)s",
    )
    server = ThreadingHTTPServer((BIND_HOST, PORT), Handler)
    LOGGER.info("router=ready primary=Byparr secondary=FlareSolverr")
    server.serve_forever(poll_interval=0.5)


if __name__ == "__main__":
    main()
