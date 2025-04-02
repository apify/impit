from typing import Literal

from .impit import (
    AsyncClient,
    Client,
    Response,
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
