import pytest

from impit import AsyncClient, BlockedAddressError, Client

from .servers import redirecting_server

# A private address that nothing in the test environment answers on, so a request that slipped past the blocklist
# would hang until the timeout instead of succeeding.
UNREACHABLE_PRIVATE_URL = 'http://10.255.255.1/internal'


def test_blocked_ip_literal() -> None:
    client = Client(blocked_addresses=['10.0.0.0/8'], timeout=5)

    with pytest.raises(BlockedAddressError, match=r'10\.255\.255\.1'):
        client.get(UNREACHABLE_PRIVATE_URL)


@pytest.mark.parametrize(
    'url',
    [
        pytest.param('http://127.0.0.1:1/', id='dotted'),
        pytest.param('http://2130706433:1/', id='decimal'),
        pytest.param('http://0x7f.0.0.1:1/', id='hex'),
        pytest.param('http://[::ffff:127.0.0.1]:1/', id='ipv4-mapped-ipv6'),
    ],
)
def test_blocked_ip_literal_spellings(url: str) -> None:
    client = Client(blocked_addresses=['127.0.0.0/8'], timeout=5)

    with pytest.raises(BlockedAddressError):
        client.get(url)


def test_blocked_hostname() -> None:
    """A hostname resolving to a blocked address is refused when it is resolved."""
    client = Client(blocked_addresses=['127.0.0.0/8', '::1/128'], timeout=5)

    with pytest.raises(BlockedAddressError, match='localhost'):
        client.get('http://localhost:1/')


def test_blocked_redirect_target() -> None:
    """A redirect to a blocked address is refused before the redirected request is sent."""
    with redirecting_server(UNREACHABLE_PRIVATE_URL) as port:
        client = Client(blocked_addresses=['10.0.0.0/8'], follow_redirects=True, timeout=5)

        with pytest.raises(BlockedAddressError, match=r'10\.255\.255\.1'):
            client.get(f'http://127.0.0.1:{port}/')


def test_manual_redirect_is_returned() -> None:
    """Without redirect following, the redirect response itself is returned and nothing is blocked."""
    with redirecting_server(UNREACHABLE_PRIVATE_URL) as port:
        client = Client(blocked_addresses=['10.0.0.0/8'], follow_redirects=False, timeout=5)

        response = client.get(f'http://127.0.0.1:{port}/')

    assert response.status_code == 302
    assert response.headers['location'] == UNREACHABLE_PRIVATE_URL


def test_invalid_entry_is_rejected() -> None:
    with pytest.raises(ValueError, match='localhost'):
        Client(blocked_addresses=['localhost'])


async def test_async_client_blocked_redirect_target() -> None:
    with redirecting_server(UNREACHABLE_PRIVATE_URL) as port:
        client = AsyncClient(blocked_addresses=['10.0.0.0/8'], follow_redirects=True, timeout=5)

        with pytest.raises(BlockedAddressError, match=r'10\.255\.255\.1'):
            await client.get(f'http://127.0.0.1:{port}/')
