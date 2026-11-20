"""Local HTTP servers used by the tests."""

from __future__ import annotations

import contextlib
import http.server
import re
import socket
import threading
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterator


@contextlib.contextmanager
def echoing_server(body_started: threading.Event | None = None) -> Iterator[int]:
    """Serve a single request, echoing it back in the response body, and yield the port to call.

    `body_started` is set as soon as the first body byte arrives, before the request is complete.
    """
    server = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 0)
    server.bind(('::', 0))
    server.listen(1)

    def received_whole_body(request: bytes) -> bool:
        head, separator, body = request.partition(b'\r\n\r\n')
        if not separator:
            return False
        content_length = re.search(rb'content-length: *(\d+)', head, re.IGNORECASE)
        if content_length is None:
            return request.endswith(b'0\r\n\r\n')
        return len(body) >= int(content_length[1])

    def echo() -> None:
        conn, _ = server.accept()
        conn.settimeout(5)
        request = b''
        with contextlib.suppress(TimeoutError):
            while not received_whole_body(request):
                request += conn.recv(1024)
                if body_started is not None and request.partition(b'\r\n\r\n')[2]:
                    body_started.set()
        conn.send(f'HTTP/1.1 200 OK\r\nContent-Length: {len(request)}\r\n\r\n'.encode() + request)
        conn.close()

    thread = threading.Thread(target=echo, daemon=True)
    thread.start()
    try:
        yield server.getsockname()[1]
    finally:
        thread.join(5)
        server.close()


@contextlib.contextmanager
def set_cookie_server(cookies: list[str]) -> Iterator[int]:
    """Answer every request with the given `Set-Cookie` headers over plain HTTP, and yield the port to call."""

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            self.send_response(200)
            for cookie in cookies:
                self.send_header('Set-Cookie', cookie)
            self.send_header('Content-Length', '0')
            self.end_headers()

        def log_message(self, *_: object) -> None:
            pass

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server.server_address[1]
    finally:
        server.shutdown()
        server.server_close()
