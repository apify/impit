import json

import pytest

from impit import AsyncClient, Browser, Cookies, TooManyRedirects

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
    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        ('protocol'),
        ['http://', 'https://'],
    )
    async def test_basic_requests(self, protocol: str, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        resp = await impit.get(f'{protocol}example.org')
        assert resp.status_code == 200

    @pytest.mark.asyncio
    async def test_context_manager(self, browser: Browser) -> None:
        async with AsyncClient(browser=browser) as impit:
            resp = await impit.get('https://example.org')
            assert resp.status_code == 200

    @pytest.mark.asyncio
    async def test_boringssl_based_server(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.get('https://www.google.com')
        assert response.status_code == 200
        assert response.text

    @pytest.mark.asyncio
    async def test_headers_work(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.get(get_httpbin_url('/headers'), headers={'Impit-Test': 'foo'})
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    @pytest.mark.asyncio
    async def test_client_wide_headers_work(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser, headers={'Impit-Test': 'foo'})

        response = await impit.get(get_httpbin_url('/headers'))
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    @pytest.mark.asyncio
    async def test_request_headers_over_client_headers(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser, headers={'Auth': '123', 'Exception': 'nope'})

        response = await impit.get(get_httpbin_url('/headers'), headers={'Exception': 'yes'})
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Auth'] == '123'
        assert json.loads(response.text)['headers']['Exception'] == 'yes'

    @pytest.mark.asyncio
    async def test_cookie_jar_works(self, browser: Browser) -> None:
        cookies = Cookies({'preset-cookie': '123'})

        impit = AsyncClient(
            browser=browser,
            cookie_jar=cookies.jar,
        )

        response = json.loads(
            (
                await impit.get(
                    get_httpbin_url('/cookies/'),
                )
            ).text
        )

        assert response['cookies'] == {'preset-cookie': '123'}

        await impit.get(
            get_httpbin_url('/cookies/set', query={'set-by-server': '321'}),
        )

        response = json.loads(
            (
                await impit.get(
                    get_httpbin_url('/cookies/'),
                )
            ).text
        )

        assert response['cookies'] == {
            'preset-cookie': '123',
            'set-by-server': '321',
        }

        assert len(cookies.jar) == 2

    @pytest.mark.asyncio
    async def test_cookies_param_works(self, browser: Browser) -> None:
        cookies = Cookies({'preset-cookie': '123'})

        impit = AsyncClient(
            browser=browser,
            cookies=cookies,
        )

        response = json.loads(
            (
                await impit.get(
                    get_httpbin_url('/cookies/'),
                )
            ).text
        )

        assert response['cookies'] == {'preset-cookie': '123'}

        await impit.get(
            get_httpbin_url('/cookies/set', query={'set-by-server': '321'}),
        )

        response = json.loads(
            (
                await impit.get(
                    get_httpbin_url('/cookies/'),
                )
            ).text
        )

        assert response['cookies'] == {
            'preset-cookie': '123',
            'set-by-server': '321',
        }

        assert len(cookies) == 2
        assert cookies.get('preset-cookie') == '123'
        assert cookies.get('set-by-server') == '321'

    @pytest.mark.asyncio
    async def test_overwriting_headers_work(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.get(get_httpbin_url('/headers'), headers={'User-Agent': 'this is impit!'})
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['User-Agent'] == 'this is impit!'

    @pytest.mark.skip(reason='Flaky under the CI environment')
    @pytest.mark.asyncio
    async def test_http3_works(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser, http3=True)

        response = await impit.get('https://curl.se', force_http3=True)
        assert response.status_code == 200
        assert 'curl' in response.text
        assert response.http_version == 'HTTP/3'

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        ('method'),
        ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'],
    )
    async def test_methods_work(self, browser: Browser, method: str) -> None:
        impit = AsyncClient(browser=browser)

        m = getattr(impit, method.lower())

        await m('https://example.org')

    @pytest.mark.asyncio
    async def test_default_no_redirect(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        target_url = 'https://example.org/'
        redirect_url = get_httpbin_url('/redirect-to', query={'url': target_url})

        response = await impit.get(redirect_url)

        assert response.status_code == 302
        assert response.is_redirect

        assert response.url == redirect_url
        assert response.headers.get('location') == target_url

    @pytest.mark.asyncio
    async def test_follow_redirects(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser, follow_redirects=True)

        target_url = 'https://example.org/'
        redirect_url = get_httpbin_url('/redirect-to', query={'url': target_url})

        response = await impit.get(redirect_url)

        assert response.status_code == 200
        assert not response.is_redirect

        assert response.url == target_url

    @pytest.mark.asyncio
    async def test_limit_redirects(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser, follow_redirects=True, max_redirects=1)

        redirect_url = get_httpbin_url('/absolute-redirect/3')

        with pytest.raises(TooManyRedirects):
            await impit.get(redirect_url)


@pytest.mark.parametrize(
    ('browser'),
    [
        'chrome',
        'firefox',
        None,
    ],
)
class TestRequestBody:
    @pytest.mark.asyncio
    async def test_passing_string_body(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.post(
            get_httpbin_url('/post'),
            content=bytearray('{"Impit-Test":"foořžš"}', 'utf-8'),
            headers={'Content-Type': 'application/json'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == '{"Impit-Test":"foořžš"}'

    @pytest.mark.asyncio
    async def test_passing_string_body_in_data(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.post(
            get_httpbin_url('/post'),
            data=bytearray('{"Impit-Test":"foořžš"}', 'utf-8'),  # type: ignore[arg-type]
            headers={'Content-Type': 'application/json'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == '{"Impit-Test":"foořžš"}'

    @pytest.mark.asyncio
    async def test_form_non_ascii(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.post(
            get_httpbin_url('/post'),
            data={'Impit-Test': '👾🕵🏻‍♂️🧑‍💻'},
        )
        assert response.status_code == 200
        assert json.loads(response.text)['form']['Impit-Test'] == '👾🕵🏻‍♂️🧑‍💻'

    @pytest.mark.asyncio
    async def test_passing_binary_body(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.post(
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
    @pytest.mark.asyncio
    async def test_methods_accept_request_body(self, browser: Browser, method: str) -> None:
        impit = AsyncClient(browser=browser)

        m = getattr(impit, method.lower())

        response = await m(get_httpbin_url(f'/{method.lower()}'), content=b'foo')
        assert response.status_code == 200
        assert json.loads(response.text)['data'] == 'foo'

    @pytest.mark.asyncio
    async def test_content(self, browser: Browser) -> None:
        impit = AsyncClient(browser=browser)

        response = await impit.get(get_httpbin_url('/'))

        assert response.status_code == 200
        assert isinstance(response.content, bytes)
        assert isinstance(response.text, str)
        assert response.content.decode('utf-8') == response.text
