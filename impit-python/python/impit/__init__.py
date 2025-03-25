from ._impit import (
    AsyncClient,
    BrowserType,
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
    'BROWSER',
    'AsyncClient',
    'BrowserType',
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


BROWSER = BrowserType()
