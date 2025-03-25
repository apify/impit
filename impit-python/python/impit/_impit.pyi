from __future__ import annotations

from typing import Literal

from typing_extensions import TypeAlias


Headers: TypeAlias = dict[str, str]
Content: TypeAlias = bytes | bytearray | list[int]
BrowserLiteral: TypeAlias = Literal['chrome', 'firefox']


class BrowserType:
    """Enum-like class defining supported browser types."""

    CHROME: str
    FIREFOX: str


class ImpitPyResponse:
    """Response object returned by impit requests."""

    status_code: int
    """HTTP status code (e.g., 200, 404)"""

    reason_phrase: str
    """HTTP reason phrase (e.g., 'OK', 'Not Found')"""

    http_version: str
    """HTTP version (e.g., 'HTTP/1.1', 'HTTP/2')"""

    headers: Headers
    """Response headers as a dictionary"""

    text: str
    """Response body as text"""

    encoding: str
    """Response content encoding"""

    is_redirect: bool
    """Whether the response is a redirect"""


class Client:
    """Synchronous HTTP client with browser impersonation capabilities."""

    def __init__(
        self,
        browser: BrowserLiteral | None = None,
        http3: bool | None = None,
        proxy: str | None = None,
        timeout: float | None = None,
        verify: bool | None = None,
    ) -> None:
        """Initialize a synchronous HTTP client.

        Args:
            browser: Browser to impersonate ("chrome" or "firefox")
            http3: Enable HTTP/3 support
            proxy: Proxy URL to use
            timeout: Default request timeout in seconds
            verify: Verify SSL certificates (set to False to ignore TLS errors)
        """

    def get(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a GET request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def post(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a POST request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol

        """

    def put(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a PUT request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def patch(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a PATCH request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def delete(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a DELETE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def head(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a HEAD request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def options(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an OPTIONS request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def trace(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make a TRACE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    def request(
        self,
        method: str,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an HTTP request with the specified method.

        Args:
            method: HTTP method (e.g., "get", "post")
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """


class AsyncClient:
    """Asynchronous HTTP client with browser impersonation capabilities."""

    def __init__(
        self,
        browser: BrowserLiteral | None = None,
        http3: bool | None = None,
        proxy: str | None = None,
        timeout: float | None = None,
        verify: bool | None = None,
    ) -> None:
        """Initialize an asynchronous HTTP client.

        Args:
            browser: Browser to impersonate ("chrome" or "firefox")
            http3: Enable HTTP/3 support
            proxy: Proxy URL to use
            timeout: Default request timeout in seconds
            verify: Verify SSL certificates (set to False to ignore TLS errors)
        """

    async def get(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous GET request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def post(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous POST request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def put(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous PUT request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def patch(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous PATCH request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def delete(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous DELETE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def head(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous HEAD request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def options(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous OPTIONS request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def trace(
        self,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous TRACE request.

        Args:
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """

    async def request(
        self,
        method: str,
        url: str,
        content: Content | None = None,
        data: dict[str, str] | None = None,
        headers: Headers | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> ImpitPyResponse:
        """Make an asynchronous HTTP request with the specified method.

        Args:
            method: HTTP method (e.g., "get", "post")
            url: URL to request
            content: Raw content to send
            data: Form data to send (will be URL-encoded)
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """


def get(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    """Make a GET request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send (will be URL-encoded)
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def post(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    """Make a POST request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send (will be URL-encoded)
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def put(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    """Make a PUT request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send (will be URL-encoded)
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def patch(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    """Make a PATCH request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send (will be URL-encoded)
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def delete(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    """Make a DELETE request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send (will be URL-encoded)
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def head(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    """Make a HEAD request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send (will be URL-encoded)
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol

    Returns:
        Response object
    """


def options(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    ...


def trace(
    url: str,
    content: Content | None = None,
    data: dict[str, str] | None = None,
    headers: Headers | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
) -> ImpitPyResponse:
    ...
