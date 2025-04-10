from __future__ import annotations

import os
import urllib.parse


def get_httpbin_url(path: str, *, query: dict[str, str] | None = None, https: bool = True) -> str:
    query = query or {}
    url = None
    if os.environ.get('APIFY_HTTPBIN_TOKEN'):
        url = urllib.parse.urlparse('https://httpbin.apify.actor')
        query['token'] = os.environ['APIFY_HTTPBIN_TOKEN']
        url = url._replace(query=urllib.parse.urlencode(query, doseq=True))
    else:
        url = urllib.parse.urlparse('https://httpbin.org')
    scheme = 'https' if https else 'http'
    url = url._replace(scheme=scheme)

    return url._replace(path=path).geturl()
