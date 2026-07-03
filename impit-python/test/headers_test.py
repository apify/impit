from impit import Headers


def test_raw_exposes_exact_wire_bytes() -> None:
    utf8 = 'attachment; filename="naïve.pdf"'
    headers = Headers([(b'content-type', b'text/plain'), (b'x-utf8', utf8.encode('utf-8'))])

    assert headers.raw == [(b'content-type', b'text/plain'), (b'x-utf8', utf8.encode('utf-8'))]


def test_utf8_only_headers_decode_as_utf8() -> None:
    utf8 = 'attachment; filename="naïve.pdf"'
    headers = Headers([(b'x-utf8', utf8.encode('utf-8'))])

    assert headers.encoding == 'utf-8'
    assert headers['x-utf8'] == utf8


def test_invalid_utf8_falls_back_to_iso_8859_1() -> None:
    # A lone 0xE4 ('ä') is not valid UTF-8, so the whole set decodes as ISO-8859-1 (httpx behavior).
    headers = Headers([(b'x-latin1', b'M\xe4rz')])

    assert headers.encoding == 'iso-8859-1'
    assert headers['x-latin1'] == 'März'


def test_ascii_headers_use_ascii_encoding() -> None:
    assert Headers([(b'a', b'b')]).encoding == 'ascii'


def test_case_insensitive_access() -> None:
    headers = Headers([(b'Content-Type', b'application/json')])

    assert headers['content-type'] == 'application/json'
    assert headers['CONTENT-TYPE'] == 'application/json'
    assert 'Content-Type' in headers
    assert headers.get('missing', 'default') == 'default'


def test_repeated_keys_join_and_list() -> None:
    headers = Headers([(b'set-cookie', b'a=1'), (b'set-cookie', b'b=2')])

    assert headers['set-cookie'] == 'a=1, b=2'
    assert headers.get_list('set-cookie') == ['a=1', 'b=2']


def test_construct_from_str_mapping() -> None:
    headers = Headers({'Content-Type': 'application/json'})

    assert headers['content-type'] == 'application/json'
    assert headers.raw == [(b'Content-Type', b'application/json')]
