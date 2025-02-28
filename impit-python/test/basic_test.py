import urllib.parse
from impit import Client
import os
import urllib

import pytest
import json

# @pytest.fixture
def getHttpBinUrl(path: str, *, https: bool = True) -> str:
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

    return urllib.parse.urljoin(url.geturl(), path)

@pytest.mark.parametrize(
    ("browser"),
    [
        "chrome",
        "firefox",
        None,
    ],
)
class TestBasicRequests:
    @pytest.mark.parametrize(
        ("protocol"),
        [
            "http://",
            "https://"
        ],
    )
    def test_basic_requests(self, protocol: str, browser: str) -> None:
        impit = Client(browser=browser)

        resp = impit.get(f"{protocol}example.org")
        assert resp.status_code == 200

    def test_headers_work(self, browser: str) -> None:
        impit = Client(browser=browser)

        response = impit.get(
            getHttpBinUrl('/headers'),
            headers = {
                'Impit-Test': 'foo',
                'Cookie': 'test=123; test2=456'
            }
        );

        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    def test_http3_works(self, browser: str) -> None:
        impit = Client(browser=browser, http3=True)

        response = impit.get(
            "https://curl.se",
            force_http3=True
        );

        assert response.status_code == 200
        assert "curl" in response.text
        assert response.http_version == "HTTP/3"

    @pytest.mark.parametrize(
        ("method"),
        [
            "GET",
            "POST",
            "PUT",
            "DELETE",
            "PATCH",
            "HEAD",
            "OPTIONS"
        ],
    )
    def test_methods_work(self, browser: str, method: str) -> None:
        impit = Client(browser=browser)

        m = getattr(impit, method.lower())

        m("https://example.org");


@pytest.mark.parametrize(
    ("browser"),
    [
        "chrome",
        "firefox",
        None,
    ],
)
class TestRequestBody:
    def test_passing_string_body(self, browser: str) -> None:
        impit = Client(browser=browser)

        response = impit.post(
            getHttpBinUrl('/post'),
            content = bytearray('{"Impit-Test":"foořžš"}', 'utf-8'),
            headers = { 'Content-Type': 'application/json' }
        );

        assert response.status_code == 200
        assert json.loads(response.text)['data'] == '{"Impit-Test":"foořžš"}'

    def test_passing_binary_body(self, browser: str) -> None:
        impit = Client(browser=browser)

        response = impit.post(
            getHttpBinUrl('/post'),
            content = [0x49, 0x6d, 0x70, 0x69, 0x74, 0x2d, 0x54, 0x65, 0x73, 0x74, 0x3a, 0x66, 0x6f, 0x6f, 0xc5, 0x99, 0xc5, 0xbe, 0xc5, 0xa1],
            headers = { 'Content-Type': 'application/json' }
        );

        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'Impit-Test:foořžš'

    @pytest.mark.parametrize(
    ("method"),
    [ "POST", "PUT", "PATCH" ],
)
    def test_methods_accept_request_body(self, browser: str, method: str) -> None:
        impit = Client(browser=browser)

        m = getattr(impit, method.lower())

        response = m(
            getHttpBinUrl(f'/{method.lower()}'),
            content = b'foo'
        );

        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'foo'
