import os
import urllib.parse

def get_httpbin_url(path: str, *, https: bool = True) -> str:
    url = None
    if os.environ.get('APIFY_HTTPBIN_TOKEN') is not None:
        url = urllib.parse.urlparse(f'https://httpbin.apify.actor')
        query = urllib.parse.parse_qs(url.query)
        query['token'] = os.environ['APIFY_HTTPBIN_TOKEN']
        url = url._replace(query=urllib.parse.urlencode(query, doseq=True))
    else:
        url = urllib.parse.urlparse(f'https://httpbin.org')
    scheme = 'https' if https else 'http'
    url = url._replace(scheme=scheme)

    return url._replace(path=path).geturl()
