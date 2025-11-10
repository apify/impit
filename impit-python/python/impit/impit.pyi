from __future__ import annotations
from http.cookiejar import CookieJar
from .cookies import Cookies

from typing import Literal, Any
from collections.abc import Iterator, AsyncIterator
from contextlib import AbstractAsyncContextManager, AbstractContextManager


Browser = Literal['chrome', 'firefox']


class HTTPError(Exception):
    """Represents an HTTP-related error."""


class RequestError(HTTPError):
    """Represents an error during the request process."""


class TransportError(RequestError):
    """Represents a transport-layer error."""


class TimeoutException(TransportError):
    """Represents a timeout error."""


class ConnectTimeout(TimeoutException):
    """Represents a connection timeout error."""


class ReadTimeout(TimeoutException):
    """Represents a read timeout error."""


class WriteTimeout(TimeoutException):
    """Represents a write timeout error."""


class PoolTimeout(TimeoutException):
    """Represents a connection pool timeout error."""


class NetworkError(TransportError):
    """Represents a network-related error."""


class ConnectError(NetworkError):
    """Represents a connection error."""


class ReadError(NetworkError):
    """Represents a read error."""


class WriteError(NetworkError):
    """Represents a write error."""


class CloseError(NetworkError):
    """Represents an error when closing a connection."""


class ProtocolError(TransportError):
    """Represents a protocol-related error."""


class LocalProtocolError(ProtocolError):
    """Represents a local protocol error."""


class RemoteProtocolError(ProtocolError):
    """Represents a remote protocol error."""


class ProxyError(TransportError):
    """Represents a proxy-related error."""


class UnsupportedProtocol(TransportError):
    """Represents an unsupported protocol error."""


class DecodingError(RequestError):
    """Represents an error during response decoding."""


class TooManyRedirects(RequestError):
    """Represents an error due to excessive redirects."""


class HTTPStatusError(HTTPError):
    """Represents an error related to HTTP status codes."""


class InvalidURL(Exception):
    """Represents an error due to an invalid URL."""


class CookieConflict(Exception):
    """Represents a cookie conflict error."""


class StreamError(Exception):
    """Represents a stream-related error."""


class StreamConsumed(StreamError):
    """Represents an error when a stream is already consumed."""


class ResponseNotRead(StreamError):
    """Represents an error when a response is not read."""


class RequestNotRead(StreamError):
    """Represents an error when a request is not read."""


class StreamClosed(StreamError):
    """Represents an error when a stream is closed."""

class Response:
    """Response object returned by impit clients (:class:`Client` or :class:`AsyncClient` instances).

    When constructed manually (e.g. for testing purposes), the following parameters can be provided:

    Args:
        status_code: HTTP status code of the response (e.g., 200, 404)
        content: Response body as bytes (or None for empty body)
        headers: Response headers as a dictionary (or None for empty headers)
        default_encoding: Default encoding for the response text. Used only if `content-type` header is not present or does not specify a charset.
        url: Final URL of the response.
    """

    status_code: int
    """HTTP status code of the response (e.g., 200, 404).

    .. tip::

        If the status code indicates an error (4xx or 5xx), you can raise an `HTTPStatusError` exception using the :meth:`raise_for_status` method.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.status_code) # 200
    """

    reason_phrase: str
    """HTTP reason phrase for the response (e.g., 'OK', 'Not Found'). This maps the numerical :attr:`status_code` to a human-readable string.

     .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.reason_phrase) # 'OK'
    """

    http_version: str
    """HTTP version (e.g., 'HTTP/1.1', 'HTTP/2') negotiated for the response during the TLS handshake.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.http_version) # 'HTTP/2'
    """

    headers: dict[str, str]
    """Response headers as a Python dictionary.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.headers) # {'content-type': 'text/html; charset=utf-8', ... }
    """

    text: str
    """Response body as text. Decoded from :attr:`content` using :attr:`encoding`.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.text) # '<!DOCTYPE html>...'
    """

    encoding: str
    """Response content encoding. Determined from `content-type` header or by bytestream prescan. Falls back to 'utf-8' if not found.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.encoding) # 'utf-8'

    This can be used to decode the `Response` body manually. By default, :attr:`text` uses this encoding to decode :attr:`content`.
    """

    is_redirect: bool
    """`True` if the response is a redirect (has a 3xx status code), `False` otherwise.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.is_redirect) # False
    """

    url: str
    """The final URL of the response. This may differ from the requested URL if redirects were followed (see the `follow_redirects` parameter in :class:`Client` and :class:`AsyncClient`).

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.url) # 'https://crawlee.dev'
    """

    content: bytes
    """Contains the response body as bytes. If the response was created with `stream=True`, this will be empty until the content is read using :meth:`read` or :meth:`iter_bytes`.

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.content) # b'<!DOCTYPE html>...'
    """

    is_closed: bool
    """
    True if the response has been closed using the :meth:`close` method, `False` otherwise.

    Closing a response releases any underlying resources (e.g., network connections).

    .. code-block:: python

        response = await client.get("https://crawlee.dev")
        print(response.is_closed) # False
        response.close()
        print(response.is_closed) # True
    """

    is_stream_consumed: bool
    """Whether the response stream has been consumed or closed.

    If this is `True`, calling :meth:`read` or :meth:`iter_bytes` will raise a :class:`StreamConsumed` or :class:`StreamClosed` error.

    The read response body is still available in the :attr:`content` attribute.

    .. code-block:: python

        response = await client.get("https://crawlee.dev", stream=True)
        print(response.is_stream_consumed) # False
        for chunk in response.iter_bytes():
            pass
        print(response.is_stream_consumed) # True
        # calling response.read() or response.iter_bytes() again will raise StreamConsumed error
        # read the content of the response using response.content
    """

    def __init__(
        self,
        status_code: int,
        *,
        content: bytes | None = None,
        headers: dict[str, str] | None = None,
        default_encoding: str | None = None,
        url: str | None = None,
    ) -> None:
        """Initialize a Response object.

        Args:
            status_code: HTTP status code
            content: Response body as bytes
            headers: Response headers as a dictionary
            default_encoding: Default encoding for the response text. Used only if `content-type` header is not present or does not specify a charset.
            url: Final URL of the response
        """

    def read(self) -> bytes:
        """Read the response content as bytes. Synchronous version of :meth:`aread`.

        Useful for consuming the entire response body in one go (not chunked).

        .. code-block:: python

            with Client() as client:
                with client.stream("GET", "https://example.com/largefile") as response:
                    content = response.read()
                    process(content)  # Process the entire content at once
        """
    def iter_bytes(self) -> Iterator[bytes]:
        """Iterate over the response content in chunks. Synchronous version of :meth:`aiter_bytes`.

        Useful for streaming large responses without loading the entire content into memory.
        .. code-block:: python

            with Client() as client:
                with client.stream("GET", "https://example.com/largefile") as response:
                    for chunk in response.iter_bytes():
                        process(chunk)  # Process each chunk as it is received
        """

    async def aread(self) -> bytes:
        """Asynchronously read the response content as bytes. Asynchronous version of :meth:`read`.

        Useful for consuming the entire response body in one go (not chunked).

        .. code-block:: python

            async with AsyncClient() as client:
                async with client.stream("GET", "https://example.com/largefile") as response:
                    content = await response.aread()
                    process(content)  # Process the entire content at once
        """

    def aiter_bytes(self) -> AsyncIterator[bytes]:
        """Asynchronously iterate over the response content in chunks. Asynchronous version of :meth:`iter_bytes`.

        Useful for streaming large responses without loading the entire content into memory.

        .. code-block:: python

            async with AsyncClient() as client:
                async with client.stream("GET", "https://example.com/largefile") as response:
                    async for chunk in response.aiter_bytes():
                        process(chunk)  # Process each chunk as it is received
        """

    def json(self) -> Any:
        """Parse the response content as JSON.

        .. note::
            This method will raise a `DecodingError` if the response content is not valid JSON.

        .. code-block:: python

            response = await client.get("https://api.example.com/data")
            data = response.json()
            print(data)  # Parsed JSON data as a Python object (dict, list, etc.)
        """

    def close(self) -> None:
        """Close the response and release resources.

        .. warning::
            You should not need to call this method directly.

            Use the `with` statement to ensure proper resource management when working with synchronous clients.

            .. code-block:: python

                with impit.stream('GET', get_httpbin_url('/')) as response:
                    assert response.status_code == 200

                assert response.is_closed is True
        """

    async def aclose(self) -> None:
        """Asynchronously close the response and release resources.

        .. note::
            This method is for internal use only.

            Use the `async with` statement to ensure proper resource management when working with asynchronous clients.

            .. code-block:: python

                async with impit.stream('GET', get_httpbin_url('/')) as response:
                    assert response.status_code == 200

                assert response.is_closed is True
        """

class Client:
    """Synchronous HTTP client with browser impersonation capabilities.

        .. note::
            You can reuse the :class:`Client` instance to make multiple requests.

            All requests made by the same client will share the same configuration, resources (e.g., cookie jar and connection pool), and other settings.

        Args:
            browser: Browser to impersonate (`"chrome"` or `"firefox"`).

                If this is `None` (default), no impersonation is performed.
            http3:

                If set to `True`, Impit will try to connect to the target servers using HTTP/3 protocol (if supported by the server).

                .. note::
                    The HTTP/3 support is experimental and may not work with all servers. The impersonation capabilities are limited when using HTTP/3.

            proxy:

                The proxy URL to use for all the requests made by this client.

                This can be an HTTP, HTTPS, or SOCKS proxy.
            timeout:
                Default request timeout in seconds.

                This value can be overridden for individual requests.
            verify:
                If set to `False`, impit will not verify SSL certificates.

                This can be used to ignore TLS errors (e.g., self-signed certificates).

                True by default.
            default_encoding:
                Default encoding for response.text field (e.g., "utf-8", "cp1252").

                Overrides `content-type` headers and bytestream prescan.
            follow_redirects:

                If set to `True` the client will automatically follow HTTP redirects (3xx responses).

                `False` by default.
            max_redirects:

                Maximum number of redirects to follow if the `follow_redirects` option is enabled.

                Default is 20.
            cookie_jar:

                Cookie jar to store cookies in.

                This is a `http.cookiejar.CookieJar` instance.

                By default, :class:`Client` doesn't store cookies between requests.
            cookies: httpx-compatible cookies object.

                These are the cookies to include in all requests (to the matching servers) made by this client.
            headers: Default HTTP headers to include in requests.

                These headers will be included in all requests made by this client.

                Default is an empty dictionary.
            local_address:

                Local address to bind the client to.

                Useful for testing purposes or when you want to bind the client to a specific network interface.
                Can be an IP address in the format `xxx.xxx.xxx.xxx` (for IPv4) or `ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff` (for IPv6).
        """

    def __enter__(self) -> Client:
        """Enter the runtime context related to this object."""

    def __exit__(self, exc_type: type[BaseException] | None, exc_value: BaseException | None, traceback: object | None) -> None:
        """Exit the runtime context related to this object."""


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
        cookie_jar: CookieJar | None = None,
        cookies: Cookies | None = None,
        headers: dict[str, str] | None = None,
        local_address: str | None = None,
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
            cookie_jar: Cookie jar to store cookies in.
            cookies: httpx-compatible cookies object.
            headers: Default HTTP headers to include in requests.
            local_address: Local address to bind the client to. Useful for testing purposes or when you want to bind the client to a specific network interface.
                Can be an IP address in the format "xxx.xxx.xxx.xxx" (for IPv4) or "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff" (for IPv6).
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
        stream: bool = False,
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
            stream: Whether to return a streaming response (default: False)
        """

    def stream(
        self,
        method: str,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> AbstractContextManager[Response]:
        """Make a streaming request with the specified method.

        This method returns a context manager that yields a streaming :class:`Response` object.

        See the following example for usage:

        .. code-block:: python

            with Client() as client:
                with client.stream("GET", "https://example.com/largefile") as response:
                    for chunk in response.iter_bytes():
                        process(chunk)  # Process each chunk as it is received

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
    """Asynchronous HTTP client with browser impersonation capabilities.

        .. note::
            You can reuse the :class:`Client` instance to make multiple requests.

            All requests made by the same client will share the same configuration, resources (e.g., cookie jar and connection pool), and other settings.

        Args:
            browser: Browser to impersonate (`"chrome"` or `"firefox"`).

                If this is `None` (default), no impersonation is performed.
            http3:

                If set to `True`, Impit will try to connect to the target servers using HTTP/3 protocol (if supported by the server).

                .. note::
                    The HTTP/3 support is experimental and may not work with all servers. The impersonation capabilities are limited when using HTTP/3.

            proxy:

                The proxy URL to use for all the requests made by this client.

                This can be an HTTP, HTTPS, or SOCKS proxy.
            timeout:
                Default request timeout in seconds.

                This value can be overridden for individual requests.
            verify:
                If set to `False`, impit will not verify SSL certificates.

                This can be used to ignore TLS errors (e.g., self-signed certificates).

                True by default.
            default_encoding:
                Default encoding for response.text field (e.g., "utf-8", "cp1252").

                Overrides `content-type` headers and bytestream prescan.
            follow_redirects:

                If set to `True` the client will automatically follow HTTP redirects (3xx responses).

                `False` by default.
            max_redirects:

                Maximum number of redirects to follow if the `follow_redirects` option is enabled.

                Default is 20.
            cookie_jar:

                Cookie jar to store cookies in.

                This is a `http.cookiejar.CookieJar` instance.

                By default, :class:`Client` doesn't store cookies between requests.
            cookies: httpx-compatible cookies object.

                These are the cookies to include in all requests (to the matching servers) made by this client.
            headers: Default HTTP headers to include in requests.

                These headers will be included in all requests made by this client.

                Default is an empty dictionary.
            local_address:

                Local address to bind the client to.

                Useful for testing purposes or when you want to bind the client to a specific network interface.
                Can be an IP address in the format `xxx.xxx.xxx.xxx` (for IPv4) or `ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff` (for IPv6).
        """

    async def __aenter__(self) -> AsyncClient:
        """Enter the runtime context related to this object."""

    async def __aexit__(self, exc_type: type[BaseException] | None, exc_value: BaseException | None, traceback: object | None) -> None:
        """Exit the runtime context related to this object."""

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
        cookie_jar: CookieJar | None = None,
        cookies: Cookies | None = None,
        headers: dict[str, str] | None = None,
        local_address: str | None = None,
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
            cookie_jar: Cookie jar to store cookies in.
            cookies: httpx-compatible cookies object.
            headers: Default HTTP headers to include in requests.
            local_address: Local address to bind the client to. Useful for testing purposes or when you want to bind the client to a specific network interface.
                Can be an IP address in the format "xxx.xxx.xxx.xxx" (for IPv4) or "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff" (for IPv6).
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
        stream: bool = False,
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
            stream: Whether to return a streaming response (default: False)
        """

    def stream(
        self,
        method: str,
        url: str,
        content: bytes | bytearray | list[int] | None = None,
        data: dict[str, str] | None = None,
        headers: dict[str, str] | None = None,
        timeout: float | None = None,
        force_http3: bool | None = None,
    ) -> AbstractAsyncContextManager[Response]:
        """Make an asynchronous streaming request with the specified method.

        This method returns a AsyncContextManager that yields a streaming :class:`Response` object.

        See the following example for usage:

        .. code-block:: python

            with Client() as client:
                with client.stream("GET", "https://example.com/largefile") as response:
                    for chunk in response.iter_bytes():
                        process(chunk)  # Process each chunk as it is received

        Args:
            method: HTTP method (e.g., "get", "post")
            url: URL to request
            content: Raw content to send
            data: Form data to send in request body
            headers: HTTP headers
            timeout: Request timeout in seconds (overrides default timeout)
            force_http3: Force HTTP/3 protocol
        """


def stream(
    method: str,
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> AbstractContextManager[Response]:
    """Make a streaming request without creating a client instance.

    Args:
        method: HTTP method (e.g., "get", "post")
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

    Returns:
        Response object
    """

def get(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a GET request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

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
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a POST request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

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
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a PUT request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

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
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a PATCH request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

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
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a DELETE request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

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
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a HEAD request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.

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
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make an OPTIONS request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds (overrides default timeout)
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.
    """


def trace(
    url: str,
    content: bytes | bytearray | list[int] | None = None,
    data: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    timeout: float | None = None,
    force_http3: bool | None = None,
    follow_redirects: bool | None = None,
    max_redirects: int | None = None,
    cookie_jar: CookieJar | None = None,
    cookies: Cookies | None = None,
    proxy: str | None = None,
) -> Response:
    """Make a TRACE request without creating a client instance.

    Args:
        url: URL to request
        content: Raw content to send
        data: Form data to send in request body
        headers: HTTP headers
        timeout: Request timeout in seconds (overrides default timeout)
        force_http3: Force HTTP/3 protocol
        follow_redirects: Whether to follow redirects (default: False)
        max_redirects: Maximum number of redirects to follow (default: 20)
        cookie_jar: Cookie jar to store cookies in.
        cookies: httpx-compatible cookies object.
        proxy: Proxy URL to use to make the request.
    """
