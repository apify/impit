from __future__ import annotations

from typing import Literal


Browser = Literal['chrome', 'firefox']

HTTPError: type
"Represents an HTTP-related error."
RequestError: type
"Represents an error during the request process."
TransportError: type
"Represents a transport-layer error."
TimeoutException: type
"Represents a timeout error."
ConnectTimeout: type
"Represents a connection timeout error."
ReadTimeout: type
"Represents a read timeout error."
WriteTimeout: type
"Represents a write timeout error."
PoolTimeout: type
"Represents a connection pool timeout error."
NetworkError: type
"Represents a network-related error."
ConnectError: type
"Represents a connection error."
ReadError: type
"Represents a read error."
WriteError: type
"Represents a write error."
CloseError: type
"Represents an error when closing a connection."
ProtocolError: type
"Represents a protocol-related error."
LocalProtocolError: type
"Represents a local protocol error."
RemoteProtocolError: type
"Represents a remote protocol error."
ProxyError: type
"Represents a proxy-related error."
UnsupportedProtocol: type
"Represents an unsupported protocol error."
DecodingError: type
"Represents an error during response decoding."
TooManyRedirects: type
"Represents an error due to excessive redirects."
HTTPStatusError: type
"Represents an error related to HTTP status codes."
InvalidURL: type
"Represents an error due to an invalid URL."
CookieConflict: type
"Represents a cookie conflict error."
StreamError: type
"Represents a stream-related error."
StreamConsumed: type
"Represents an error when a stream is already consumed."
ResponseNotRead: type
"Represents an error when a response is not read."
RequestNotRead: type
"Represents an error when a request is not read."
StreamClosed: type
"Represents an error when a stream is closed."

class Response:
    """Response object returned by impit requests."""

    status_code: int
    """HTTP status code (e.g., 200, 404)"""

    reason_phrase: str
    """HTTP reason phrase (e.g., 'OK', 'Not Found')"""

    http_version: str
    """HTTP version (e.g., 'HTTP/1.1', 'HTTP/2')"""

    headers: dict[str, str]
    """Response headers as a dictionary"""

    text: str
    """Response body as text. Decoded from `content` using `encoding`."""

    encoding: str
    """Response content encoding"""

    is_redirect: bool
    """Whether the response is a redirect"""

    url: str
    """Final URL"""

    content: bytes
    """Response body as bytes"""


class Client:
    """Synchronous HTTP client with browser impersonation capabilities."""

    def __init__(
        self,
        browser: Browser | None = None,
        http3: bool | None = None,
        proxy: str | None = None,
        timeout: float | None = None,
        verify: bool | None = None,
        default_encoding: str | None = None,
        follow_redirects: bool | None = None,
        max_redirects: int | None = None,
    ) -> None:
        """Initialize a synchronous HTTP client.

        Args:
            browser: Browser to impersonate ("chrome" or "firefox")
            http3: Enable HTTP/3 support
            proxy: Proxy URL to use
            timeout: Default request timeout in seconds
            verify: Verify SSL certificates (set to False to ignore TLS errors)
            default_encoding: Default encoding for response.text field (e.g., "utf-8", "cp1252"). Overrides `content-type`
                header and bytestream prescan.
            follow_redirects: Whether to follow redirects (default: False)
            max_redirects: Maximum number of redirects to follow (default: 20)
        """

    def get(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a GET request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def post(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a POST request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol

        """

    def put(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a PUT request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def patch(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a PATCH request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def delete(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a DELETE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def head(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a HEAD request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def options(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an OPTIONS request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def trace(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make a TRACE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def request(
        self,
        method: str,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an HTTP request with the specified method.

        Args:
            method: HTTP method (e.g., "get", "post")
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """


class AsyncClient:
    """Asynchronous HTTP client with browser impersonation capabilities."""

    def __init__(
        self,
        browser: Browser | None = None,
        http3: bool | None = None,
        proxy: str | None = None,
        timeout: float | None = None,
        verify: bool | None = None,
        default_encoding: str | None = None,
        follow_redirects: bool | None = None,
        max_redirects: int | None = None,
    ) -> None:
        """Initialize an asynchronous HTTP client.

        Args:
            browser: Browser to impersonate ("chrome" or "firefox")
            http3: Enable HTTP/3 support
            proxy: Proxy URL to use
            timeout: Default request timeout in seconds
            verify: Verify SSL certificates (set to False to ignore TLS errors)
            default_encoding: Default encoding for response.text field (e.g., "utf-8", "cp1252"). Overrides `content-type`
                header and bytestream prescan.
            follow_redirects: Whether to follow redirects (default: False)
            max_redirects: Maximum number of redirects to follow (default: 20)
        """

    async def get(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous GET request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def post(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous POST request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def put(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous PUT request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def patch(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous PATCH request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def delete(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous DELETE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def head(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous HEAD request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def options(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous OPTIONS request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def trace(
        self,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous TRACE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def request(
        self,
        method: str,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> Response:
        """Make an asynchronous HTTP request with the specified method.

        Args:
            method: HTTP method (e.g., "get", "post")
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """


def get(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a GET request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def post(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a POST request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def put(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a PUT request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def patch(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a PATCH request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def delete(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a DELETE request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def head(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a HEAD request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def options(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make an OPTIONS request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds (overrides default timeout)
        force_http3: Force HTTP/3 protocol
    """


def trace(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> Response:
    """Make a TRACE request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds (overrides default timeout)
        force_http3: Force HTTP/3 protocol
    """
