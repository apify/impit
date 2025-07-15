from __future__ import annotations

import os
import urllib.parse


def get_httpbin_url(path: str, *, query: dict[str, list[str] | str] | None = None, https: bool = True) -> str:
    query = query or {}
    url = None
    if os.environ.get('APIFY_HTTPBIN_TOKEN'):
        url = urllib.parse.urlparse('https://httpbin.apify.actor')
        query['token'] = os.environ['APIFY_HTTPBIN_TOKEN']
        url = url._replace(query=urllib.parse.urlencode(query, doseq=True))
    else:
        url = urllib.parse.urlparse('https://httpbin.org')
        if query:
            url = url._replace(query=urllib.parse.urlencode(query, doseq=True))

    scheme = 'https' if https else 'http'
    url = url._replace(scheme=scheme)
    result_url = url._replace(path=path).geturl()

    return result_url.removesuffix('/')
