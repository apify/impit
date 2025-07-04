from typing import Literal

from .cookies import Cookies
from .impit import (
    AsyncClient,
    Client,
    CloseError,
    ConnectError,
    ConnectTimeout,
    CookieConflict,
    DecodingError,
    HTTPError,
    HTTPStatusError,
    InvalidURL,
    LocalProtocolError,
    NetworkError,
    PoolTimeout,
    ProtocolError,
    ProxyError,
    ReadError,
    ReadTimeout,
    RemoteProtocolError,
    RequestError,
    RequestNotRead,
    Response,
    ResponseNotRead,
    StreamClosed,
    StreamConsumed,
    StreamError,
    TimeoutException,
    TooManyRedirects,
    TransportError,
    UnsupportedProtocol,
    WriteError,
    WriteTimeout,
    delete,
    get,
    head,
    options,
    patch,
    post,
    put,
    trace,
)

__all__ = [
    'AsyncClient',
    'Browser',
    'Client',
    'CloseError',
    'ConnectError',
    'ConnectTimeout',
    'CookieConflict',
    'Cookies',
    'DecodingError',
    'HTTPError',
    'HTTPStatusError',
    'InvalidURL',
    'LocalProtocolError',
    'NetworkError',
    'PoolTimeout',
    'ProtocolError',
    'ProxyError',
    'ReadError',
    'ReadTimeout',
    'RemoteProtocolError',
    'RequestError',
    'RequestNotRead',
    'Response',
    'ResponseNotRead',
    'StreamClosed',
    'StreamConsumed',
    'StreamError',
    'TimeoutException',
    'TooManyRedirects',
    'TransportError',
    'UnsupportedProtocol',
    'WriteError',
    'WriteTimeout',
    'delete',
    'get',
    'head',
    'options',
    'patch',
    'post',
    'put',
    'trace',
]


Browser = Literal['chrome', 'firefox']
