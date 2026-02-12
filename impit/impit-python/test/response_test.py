import pytest

from impit import Response


async def test_setattr() -> None:
    response = Response(200)

    assert response.status_code == 200
    assert getattr(response, 'test', None) is None
    setattr(response, 'test', 'test_value')  # noqa: B010
    assert getattr(response, 'test', None) == 'test_value'


def test_response_constructor_with_status() -> None:
    # Create a new response object with a specific status code
    response = Response(404)

    assert response.status_code == 404
    assert response.content == b''
    assert response.text == ''

    assert response.reason_phrase == 'Not Found'


def test_response_constructor_with_content() -> None:
    # Create a new response object with content
    response = Response(200, content=b'Test content')

    assert response.status_code == 200
    assert response.content == b'Test content'
    assert response.text == 'Test content'


def test_response_constructor_with_headers() -> None:
    # Create a new response object with headers
    response = Response(200, headers={'Content-Type': 'application/json'})

    assert response.status_code == 200
    assert response.headers['Content-Type'] == 'application/json'


def test_response_headers_encoding() -> None:
    response = Response(
        200, headers={'Content-Type': 'text/plain; charset=cp1250'}, content=b'\x9e\x64\xe1\xf8\x65\x6e\xed'
    )

    assert response.text == 'ždáření'


def test_response_headers_explicit_encoding() -> None:
    response = Response(200, content=b'\xa6\xcd\xd0\xa6\xd4\x20\xd2\xd5\xcc\xc5\xd3', default_encoding='koi8-u')

    assert response.text == 'імпіт рулес'
    assert response.content == b'\xa6\xcd\xd0\xa6\xd4 \xd2\xd5\xcc\xc5\xd3'


@pytest.mark.parametrize(
    ('status_code', 'reason_phrase'),
    [
        (200, 'OK'),
        (301, 'Moved Permanently'),
        (404, 'Not Found'),
        (500, 'Internal Server Error'),
        (600, 'Unknown'),
    ],
)
def test_response_constructor_with_status_reason(status_code: int, reason_phrase: str) -> None:
    response = Response(status_code)

    assert response.status_code == status_code
    assert response.reason_phrase == reason_phrase
