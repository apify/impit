from typing import Literal

from .impit import (
    AsyncClient,
    Client,
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
