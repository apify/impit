from typing import Literal

from .impit import (
    AsyncClient,
    Client,
    Response,
    HttpError,
    HttpStatusError,
    RequestError,
    TransportError,
    UnsupportedProtocol,
    TooManyRedirects,
    InvalidUrl,
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
    'HttpError',
    'HttpStatusError',
    'RequestError',
    'TransportError',
    'UnsupportedProtocol',
    'TooManyRedirects',
    'InvalidUrl',
    'Browser',
    'Client',
    'Response',
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
