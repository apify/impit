import json
from http.cookiejar import CookieJar

import pytest

from impit import Browser, Client, Cookies, StreamClosed, StreamConsumed, TooManyRedirects

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

        resp = impit.get(f'{protocol}apify.com')
        assert resp.status_code == 200 if protocol == 'https://' else resp.status_code == 301

    def test_context_manager(self, browser: Browser) -> None:
        with Client(browser=browser) as impit:
            resp = impit.get(get_httpbin_url('/get'))
            assert resp.status_code == 200

    def test_boringssl_based_server(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.get('https://www.google.com')
        assert response.status_code == 200
        assert response.text

    def test_content_encoding(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        resp = impit.get(get_httpbin_url('/encoding/utf8'))
        assert resp.status_code == 200
        assert resp.encoding == 'utf-8'

    def test_headers_work(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.get(get_httpbin_url('/headers'), headers={'Impit-Test': 'foo'})
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    def test_client_wide_headers_work(self, browser: Browser) -> None:
        impit = Client(browser=browser, headers={'Impit-Test': 'foo'})

        response = impit.get(get_httpbin_url('/headers'))
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Impit-Test'] == 'foo'

    def test_request_headers_over_client_headers(self, browser: Browser) -> None:
        impit = Client(browser=browser, headers={'Auth': '123', 'Exception': 'nope'})

        response = impit.get(get_httpbin_url('/headers'), headers={'Exception': 'yes'})
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['Auth'] == '123'
        assert json.loads(response.text)['headers']['Exception'] == 'yes'

    def test_cookies_nonstandard(self, browser: Browser) -> None:
        cookies_jar = CookieJar()

        impit = Client(browser=browser, cookie_jar=cookies_jar, follow_redirects=True)

        impit.get(get_httpbin_url('/cookies/set', query={'set-by-server': '321'}))

        for cookie in cookies_jar:
            assert cookie.has_nonstandard_attr('HttpOnly') is not None

    def test_complex_cookies(self, browser: Browser) -> None:
        cookies_jar = CookieJar()

        impit = Client(browser=browser, cookie_jar=cookies_jar, follow_redirects=True)

        url = get_httpbin_url(
            '/response-headers',
            query={
                'set-cookie': [
                    'basic=1; Path=/; HttpOnly; SameSite=Lax',
                    'withpath=2; Path=/html; SameSite=None',
                    'strict=3; Path=/; SameSite=Strict',
                    'secure=4; Path=/; HttpOnly; Secure; SameSite=Strict',
                    'short=5; Path=/;',
                    'domain=6; Path=/; Domain=.127.0.0.1;',
                ]
            },
        )

        impit.get(url)

        assert len(cookies_jar) == 6
        for cookie in cookies_jar:
            if cookie.name == 'basic':
                assert cookie.value == '1'
                assert cookie.secure is False
                assert cookie.has_nonstandard_attr('HttpOnly') is True
                assert cookie.get_nonstandard_attr('SameSite') == 'Lax'
            elif cookie.name == 'withpath':
                assert cookie.value == '2'
                assert cookie.secure is False
                assert cookie.get_nonstandard_attr('SameSite') == 'None'
                assert cookie.has_nonstandard_attr('HttpOnly') is False
                assert cookie.path == '/html'
            elif cookie.name == 'strict':
                assert cookie.value == '3'
                assert cookie.secure is False
                assert cookie.has_nonstandard_attr('HttpOnly') is False
                assert cookie.get_nonstandard_attr('SameSite') == 'Strict'
            elif cookie.name == 'secure':
                assert cookie.value == '4'
                assert cookie.secure is True
                assert cookie.has_nonstandard_attr('HttpOnly') is True
                assert cookie.get_nonstandard_attr('SameSite') == 'Strict'
            elif cookie.name == 'short':
                assert cookie.value == '5'
                assert cookie.secure is False
                assert cookie.has_nonstandard_attr('SameSite') is False
            elif cookie.name == 'domain':
                assert cookie.value == '6'
                assert cookie.secure is False
                # Crate cookies, ignores the starting dot in the domain
                # but it's ok - https://www.rfc-editor.org/rfc/rfc6265#section-4.1.2.3
                assert cookie.domain == '127.0.0.1'

    def test_cookie_jar_works(self, browser: Browser) -> None:
        cookies = Cookies({'preset-cookie': '123'})

        impit = Client(
            browser=browser,
            cookie_jar=cookies.jar,
        )

        response = json.loads(
            impit.get(
                get_httpbin_url('/cookies/'),
            ).text
        )

        assert response['cookies'] == {'preset-cookie': '123'}

        impit.get(
            get_httpbin_url('/cookies/set', query={'set-by-server': '321'}),
        )

        response = json.loads(
            impit.get(
                get_httpbin_url('/cookies/'),
            ).text
        )

        assert response['cookies'] == {
            'preset-cookie': '123',
            'set-by-server': '321',
        }

        assert len(cookies.jar) == 2

    def test_cookies_param_works(self, browser: Browser) -> None:
        cookies = Cookies({'preset-cookie': '123'})

        impit = Client(
            browser=browser,
            cookies=cookies,
        )

        response = json.loads(
            impit.get(
                get_httpbin_url('/cookies/'),
            ).text
        )

        assert response['cookies'] == {'preset-cookie': '123'}

        impit.get(
            get_httpbin_url('/cookies/set', query={'set-by-server': '321'}),
        )

        response = json.loads(
            impit.get(
                get_httpbin_url('/cookies/'),
            ).text
        )

        assert response['cookies'] == {
            'preset-cookie': '123',
            'set-by-server': '321',
        }

        assert len(cookies) == 2
        assert cookies.get('preset-cookie') == '123'
        assert cookies.get('set-by-server') == '321'

    def test_overwriting_headers_work(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.get(get_httpbin_url('/headers'), headers={'User-Agent': 'this is impit!'})
        assert response.status_code == 200
        assert json.loads(response.text)['headers']['User-Agent'] == 'this is impit!'

    @pytest.mark.skip(reason='Flaky under the CI environment')
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

        m(get_httpbin_url('/anything'))

    def test_default_no_redirect(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        target_url = 'https://crawlee.dev/'
        redirect_url = get_httpbin_url('/redirect-to', query={'url': target_url})

        response = impit.get(redirect_url)

        assert response.status_code == 302
        assert response.is_redirect

        assert response.url == redirect_url
        assert response.headers.get('location') == target_url

    def test_follow_redirects(self, browser: Browser) -> None:
        impit = Client(browser=browser, follow_redirects=True)

        target_url = 'https://crawlee.dev/'
        redirect_url = get_httpbin_url('/redirect-to', query={'url': target_url})

        response = impit.get(redirect_url)

        assert response.status_code == 200
        assert not response.is_redirect

        assert response.url == target_url

    def test_limit_redirects(self, browser: Browser) -> None:
        impit = Client(browser=browser, follow_redirects=True, max_redirects=1)

        redirect_url = get_httpbin_url('/absolute-redirect/3')

        with pytest.raises(TooManyRedirects):
            impit.get(redirect_url)


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

    def test_content(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        response = impit.get(get_httpbin_url('/'))

        assert response.status_code == 200
        assert isinstance(response.content, bytes)
        assert isinstance(response.text, str)
        assert response.content.decode('utf-8') == response.text


@pytest.mark.parametrize(
    ('browser'),
    [
        'chrome',
        'firefox',
        None,
    ],
)
class TestStreamRequest:
    def test_read(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200
            assert response.is_closed is False
            assert response.is_stream_consumed is False

            content = response.read()

            assert isinstance(content, bytes)
            assert content.decode('utf-8') == response.text
            assert response.content == content

            assert response.is_closed is True
            assert response.is_stream_consumed is True  # type: ignore[unreachable] # Mypy can't detect a change of state

    def test_iter_bytes(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200
            assert response.is_closed is False
            assert response.is_stream_consumed is False

            content = b''.join(response.iter_bytes())

            assert isinstance(content, bytes)
            assert len(content) > 0

            # After `iter_bytes`` we should get an error since `content` and `text` are not cached
            with pytest.raises(StreamConsumed):
                _ = response.text

            with pytest.raises(StreamConsumed):
                _ = response.content

            assert response.is_closed is True
            assert response.is_stream_consumed is True  # type: ignore[unreachable] # Mypy can't detect a change of state

    def test_response_with_context_manager(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200
            assert response.is_closed is False
            assert response.is_stream_consumed is False

        assert response.is_closed is True
        assert response.is_stream_consumed is False  # type: ignore[unreachable] # Mypy can't detect a change of state

    def test_read_after_close(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200

        assert response.is_closed is True

        with pytest.raises(StreamClosed):
            _ = response.read()

    def test_two_read_calls(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200

            content = response.read()
            assert isinstance(content, bytes)
            assert content == response.content

            # Return content from cache
            assert response.read() == response.content

    def test_two_iter_bytes_calls(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200

            content = b''.join(response.iter_bytes())
            assert isinstance(content, bytes)
            assert len(content) > 0

            # `iter_bytes` don't cache content
            with pytest.raises(StreamConsumed):
                _ = b''.join(response.iter_bytes())

    def test_iter_bytes_without_consumed(self, browser: Browser) -> None:
        impit = Client(browser=browser)

        with impit.stream('GET', get_httpbin_url('/')) as response:
            assert response.status_code == 200

            iterator = response.iter_bytes()

            _ = next(iterator)

        assert response.is_closed is True
        assert response.is_stream_consumed is False

        with pytest.raises(StreamClosed):
            _ = response.text

        with pytest.raises(StreamClosed):
            _ = response.content
