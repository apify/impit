from impit import AsyncClient
from .httpbin import get_httpbin_url

import pytest
import json

@pytest.mark.parametrize(
    ("browser"),
    [
        "chrome",
        "firefox",
        None,
    ],
)
class TestBasicRequests:
    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        ("protocol"),
        [
            "http://",
            "https://"
        ],
    )
    async def test_basic_requests(self, protocol: str, browser: str) -> None:
        impit = AsyncClient(browser=browser)

        resp = await impit.get(f"{protocol}example.org")
        assert resp.status_code == 200

    @pytest.mark.asyncio
    async def test_headers_work(self, browser: str) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.get(
            get_httpbin_url('/headers'),
            headers = {
                'Impit-Test': 'foo',
                'Cookie': 'test=123; test2=456'
            }
        );

        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    @pytest.mark.asyncio
    async def test_http3_works(self, browser: str) -> None:
        impit = AsyncClient(browser=browser, http3=True)

        response = await impit.get(
            "https://curl.se",
            force_http3=True
        );

        assert response.status_code == 200
        assert "curl" in response.text
        assert response.http_version == "HTTP/3"

    @pytest.mark.asyncio
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
    async def test_methods_work(self, browser: str, method: str) -> None:
        impit = AsyncClient(browser=browser)

        m = getattr(impit, method.lower())

        await m("https://example.org");


@pytest.mark.parametrize(
    ("browser"),
    [
        "chrome",
        "firefox",
        None,
    ],
)
class TestRequestBody:
    @pytest.mark.asyncio
    async def test_passing_string_body(self, browser: str) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.post(
            get_httpbin_url('/post'),
            content = bytearray('{"Impit-Test":"foořžš"}', 'utf-8'),
            headers = { 'Content-Type': 'application/json' }
        );

        assert response.status_code == 200
        assert json.loads(response.text)['data'] == '{"Impit-Test":"foořžš"}'

    @pytest.mark.asyncio
    async def test_passing_binary_body(self, browser: str) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.post(
            get_httpbin_url('/post'),
            content = [0x49, 0x6d, 0x70, 0x69, 0x74, 0x2d, 0x54, 0x65, 0x73, 0x74, 0x3a, 0x66, 0x6f, 0x6f, 0xc5, 0x99, 0xc5, 0xbe, 0xc5, 0xa1],
            headers = { 'Content-Type': 'application/json' }
        );

        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'Impit-Test:foořžš'

    @pytest.mark.parametrize(
    ("method"),
    [ "POST", "PUT", "PATCH" ],
)
    @pytest.mark.asyncio
    async def test_methods_accept_request_body(self, browser: str, method: str) -> None:
        impit = AsyncClient(browser=browser)

        m = getattr(impit, method.lower())

        response = await m(
            get_httpbin_url(f'/{method.lower()}'),
            content = b'foo'
        );

        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'foo'
