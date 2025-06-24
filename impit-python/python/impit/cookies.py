# ruff: noqa: SIM102
from __future__ import annotations

import typing
from http.cookiejar import Cookie, CookieJar

from .impit import CookieConflict

CookieTypes = typing.Union['Cookies', CookieJar, dict[str, str], list[tuple[str, str]]]


class Cookies(typing.MutableMapping[str, str]):
    """HTTP Cookies, as a mutable mapping."""

    def __init__(self, cookies: CookieTypes | None = None) -> None:
        if cookies is None or isinstance(cookies, dict):
            self.jar = CookieJar()
            if isinstance(cookies, dict):
                for key, value in cookies.items():
                    self.set(key, value)
        elif isinstance(cookies, list):
            self.jar = CookieJar()
            for key, value in cookies:
                self.set(key, value)
        elif isinstance(cookies, Cookies):
            self.jar = CookieJar()
            for cookie in cookies.jar:
                self.jar.set_cookie(cookie)
        else:
            self.jar = cookies

    def set(self, name: str, value: str, domain: str = '', path: str = '/') -> None:
        """Set a cookie value by name. May optionally include domain and path."""
        kwargs = {
            'version': 0,
            'name': name,
            'value': value,
            'port': None,
            'port_specified': False,
            'domain': domain,
            'domain_specified': bool(domain),
            'domain_initial_dot': domain.startswith('.'),
            'path': path,
            'path_specified': bool(path),
            'secure': False,
            'expires': None,
            'discard': True,
            'comment': None,
            'comment_url': None,
            'rest': {'HttpOnly': None},
            'rfc2109': False,
        }
        cookie = Cookie(**kwargs)
        self.jar.set_cookie(cookie)

    def get(
        self,
        name: str,
        default: str | None = None,
        domain: str | None = None,
        path: str | None = None,
    ) -> str | None:
        """Get a cookie by name.

        May optionally include domain and path in order to specify exactly which cookie to retrieve.
        """
        value = None
        for cookie in self.jar:
            if cookie.name == name:
                if domain is None or cookie.domain == domain:
                    if path is None or cookie.path == path:
                        if value is not None:
                            message = f'Multiple cookies exist with name={name}'
                            raise CookieConflict(message)
                        value = cookie.value

        if value is None:
            return default
        return value

    def delete(
        self,
        name: str,
        domain: str | None = None,
        path: str | None = None,
    ) -> None:
        """Delete a cookie by name.

        May optionally include domain and path in order to specify exactly which cookie to delete.
        """
        if domain is not None and path is not None:
            return self.jar.clear(domain, path, name)

        remove = [
            cookie
            for cookie in self.jar
            if cookie.name == name
            and (domain is None or cookie.domain == domain)
            and (path is None or cookie.path == path)
        ]

        for cookie in remove:
            self.jar.clear(cookie.domain, cookie.path, cookie.name)
        return None

    def clear(self, domain: str | None = None, path: str | None = None) -> None:
        """Delete all cookies.

        Optionally include a domain and path in order to only delete a subset of all the cookies.
        """
        args = []
        if domain is not None:
            args.append(domain)
        if path is not None:
            assert domain is not None  # noqa: S101
            args.append(path)
        self.jar.clear(*args)

    def update(self, cookies: CookieTypes | None = None) -> None:
        """Update the cookie jar with new cookies. Accepts various types."""
        cookies = Cookies(cookies)
        for cookie in cookies.jar:
            self.jar.set_cookie(cookie)

    def __setitem__(self, name: str, value: str) -> None:
        """Set a cookie by name."""
        return self.set(name, value)

    def __getitem__(self, name: str) -> str:
        """Get a cookie by name."""
        value = self.get(name)
        if value is None:
            raise KeyError(name)
        return value

    def __delitem__(self, name: str) -> None:
        """Delete a cookie by name."""
        return self.delete(name)

    def __len__(self) -> int:
        """Return the number of cookies in the jar."""
        return len(self.jar)

    def __iter__(self) -> typing.Iterator[str]:
        """Return an iterator over the names of the cookies in the jar."""
        return (cookie.name for cookie in self.jar)

    def __bool__(self) -> bool:
        """Return True if there are any cookies in the jar."""
        for _ in self.jar:
            return True
        return False

    def __repr__(self) -> str:
        """Return a string representation of the Cookies object."""
        cookies_repr = ', '.join(
            [f'<Cookie {cookie.name}={cookie.value} for {cookie.domain} />' for cookie in self.jar]
        )

        return f'<Cookies[{cookies_repr}]>'
