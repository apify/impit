import json

import pytest

from impit import Browser, Client

from .httpbin import get_httpbin_url


@pytest.mark.parametrize(
    ('browser'),
    [
        'chrome',
        'firefox',
        None,
    ],
)
class TestBasicRequests:
    @pytest.mark.parametrize(
        ('protocol'),
        ['http://', 'https://'],
    )
    def test_basic_requests(self, protocol: str, browser: Browser) -> None:
        impit = Client(browser=browser)

        resp = impit.get(f'{protocol}example.org')
        assert resp.status_code == 200

    def test_content_encoding(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        resp = impit.get(get_httpbin_url('/encoding/utf8'))
        assert resp.status_code == 200
        assert resp.encoding == 'utf-8'

    def test_headers_work(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.get(
            get_httpbin_url('/headers'), headers={'Impit-Test': 'foo', 'Cookie': 'test=123; test2=456'}
        )
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    def test_http3_works(self, browser: Browser) -> None:
        impit = Client(browser=browser, http3=True)

        response = impit.get('https://curl.se', force_http3=True)
        assert response.status_code == 200
        assert 'curl' in response.text
        assert response.http_version == 'HTTP/3'

    @pytest.mark.parametrize(
        ('method'),
        ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'],
    )
    def test_methods_work(self, browser: Browser, method: str) -> None:
        impit = Client(browser=browser)

        m = getattr(impit, method.lower())

        m('https://example.org')


@pytest.mark.parametrize(
    ('browser'),
    [
        'chrome',
        'firefox',
        None,
    ],
)
class TestRequestBody:
    def test_passing_string_body(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.post(
            get_httpbin_url('/post'),
            content=bytearray('{"Impit-Test":"foořžš"}', 'utf-8'),
            headers={'Content-Type': 'application/json'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == '{"Impit-Test":"foořžš"}'

    def test_passing_string_body_in_data(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.post(
            get_httpbin_url('/post'),
            data=bytearray('{"Impit-Test":"foořžš"}', 'utf-8'),  # type: ignore[arg-type]
            headers={'Content-Type': 'application/json'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == '{"Impit-Test":"foořžš"}'

    def test_form_non_ascii(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.post(
            get_httpbin_url('/post'),
            data={'Impit-Test': '👾🕵🏻‍♂️🧑‍💻'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['form']['Impit-Test'] == '👾🕵🏻‍♂️🧑‍💻'

    def test_passing_binary_body(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.post(
            get_httpbin_url('/post'),
            content=[
                0x49,
                0x6D,
                0x70,
                0x69,
                0x74,
                0x2D,
                0x54,
                0x65,
                0x73,
                0x74,
                0x3A,
                0x66,
                0x6F,
                0x6F,
                0xC5,
                0x99,
                0xC5,
                0xBE,
                0xC5,
                0xA1,
            ],
            headers={'Content-Type': 'application/json'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'Impit-Test:foořžš'

    @pytest.mark.parametrize(
        ('method'),
        ['POST', 'PUT', 'PATCH'],
    )
    def test_methods_accept_request_body(self, browser: Browser, method: str) -> None:
        impit = Client(browser=browser)

        m = getattr(impit, method.lower())

        response = m(get_httpbin_url(f'/{method.lower()}'), content=b'foo')
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'foo'
