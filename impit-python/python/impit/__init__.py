from typing import Literal
from typing_extensions import TypeAlias

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
    'Client',
    'BrowserType',
    'delete',
    'get',
    'head',
    'options',
    'patch',
    'post',
    'put',
    'trace',
]


BrowserType: TypeAlias = Literal['chrome', 'firefox']
