from __future__ import annotations

import urllib.parse


def get_httpbin_url(path: str, *, query: dict[str, list[str] | str] | None = None, https: bool = True) -> str:
    url = urllib.parse.urlparse('https://httpbin.apify.actor')
    if query:
        url = url._replace(query=urllib.parse.urlencode(query, doseq=True))

    scheme = 'https' if https else 'http'
    url = url._replace(scheme=scheme)
    result_url = url._replace(path=path).geturl()

    return result_url.removesuffix('/')
