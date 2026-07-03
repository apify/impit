"""Copyright © 2019, [Encode OSS Ltd](https://www.encode.io/).

All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

* Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

* Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

* Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""
# ruff: noqa: D102, D105, E501, FBT001, FBT002, PERF203, PLW1641, SIM118, SLF001

## The Headers class below is a modification of the `httpx.Headers` class, licensed under the BSD 3-Clause "New" License.

from __future__ import annotations

import typing
from collections.abc import Mapping

HeaderTypes = typing.Union[
    'Headers',
    typing.Mapping[str, str],
    typing.Mapping[bytes, bytes],
    typing.Sequence[tuple[str, str]],
    typing.Sequence[tuple[bytes, bytes]],
]

SENSITIVE_HEADERS = {'authorization', 'proxy-authorization'}


def _normalize_header_key(key: str | bytes, encoding: str | None = None) -> bytes:
    return key if isinstance(key, bytes) else key.encode(encoding or 'ascii')


def _normalize_header_value(value: str | bytes, encoding: str | None = None) -> bytes:
    return value if isinstance(value, bytes) else value.encode(encoding or 'ascii')


def _obfuscate_sensitive_headers(
    items: typing.Iterable[tuple[str, str]],
) -> typing.Iterator[tuple[str, str]]:
    for k, v in items:
        yield k, ('[secure]' if k.lower() in SENSITIVE_HEADERS else v)


class Headers(typing.MutableMapping[str, str]):
    """HTTP headers, as a case-insensitive multi-dict.

    Mirrors the `httpx.Headers` interface, including the `raw` property that exposes the exact
    header value bytes received on the wire.
    """

    def __init__(self, headers: HeaderTypes | None = None, encoding: str | None = None) -> None:
        self._list: list[tuple[bytes, bytes, bytes]] = []

        if isinstance(headers, Headers):
            self._list = list(headers._list)
        elif isinstance(headers, Mapping):
            for k, v in headers.items():
                bytes_key = _normalize_header_key(k, encoding)
                bytes_value = _normalize_header_value(v, encoding)
                self._list.append((bytes_key, bytes_key.lower(), bytes_value))
        elif headers is not None:
            for k, v in headers:
                bytes_key = _normalize_header_key(k, encoding)
                bytes_value = _normalize_header_value(v, encoding)
                self._list.append((bytes_key, bytes_key.lower(), bytes_value))

        self._encoding = encoding

    @property
    def encoding(self) -> str:
        """Header encoding is mandated as ascii, but we allow fallbacks to utf-8 or iso-8859-1."""
        if self._encoding is None:
            for encoding in ['ascii', 'utf-8']:
                for key, value in self.raw:
                    try:
                        key.decode(encoding)
                        value.decode(encoding)
                    except UnicodeDecodeError:
                        break
                else:
                    self._encoding = encoding
                    break
            else:
                # ISO-8859-1 covers all 256 byte values, so it never raises decode errors.
                self._encoding = 'iso-8859-1'
        return self._encoding

    @encoding.setter
    def encoding(self, value: str) -> None:
        self._encoding = value

    @property
    def raw(self) -> list[tuple[bytes, bytes]]:
        """Return the raw header items, as `(name, value)` byte pairs."""
        return [(raw_key, value) for raw_key, _, value in self._list]

    def keys(self) -> typing.KeysView[str]:
        return {key.decode(self.encoding): None for _, key, value in self._list}.keys()

    def values(self) -> typing.ValuesView[str]:
        values_dict: dict[str, str] = {}
        for _, key, value in self._list:
            str_key = key.decode(self.encoding)
            str_value = value.decode(self.encoding)
            if str_key in values_dict:
                values_dict[str_key] += f', {str_value}'
            else:
                values_dict[str_key] = str_value
        return values_dict.values()

    def items(self) -> typing.ItemsView[str, str]:
        """Return `(key, value)` items, concatenating repeated keys with commas."""
        values_dict: dict[str, str] = {}
        for _, key, value in self._list:
            str_key = key.decode(self.encoding)
            str_value = value.decode(self.encoding)
            if str_key in values_dict:
                values_dict[str_key] += f', {str_value}'
            else:
                values_dict[str_key] = str_value
        return values_dict.items()

    def multi_items(self) -> list[tuple[str, str]]:
        """Return `(key, value)` pairs without concatenating repeated keys."""
        return [(key.decode(self.encoding), value.decode(self.encoding)) for _, key, value in self._list]

    def get(self, key: str, default: typing.Any = None) -> typing.Any:
        """Return a header value, concatenating repeated occurrences with commas."""
        try:
            return self[key]
        except KeyError:
            return default

    def get_list(self, key: str, split_commas: bool = False) -> list[str]:
        """Return all header values for `key`, optionally splitting on commas."""
        get_header_key = key.lower().encode(self.encoding)
        values = [
            item_value.decode(self.encoding)
            for _, item_key, item_value in self._list
            if item_key.lower() == get_header_key
        ]
        if not split_commas:
            return values
        split_values = []
        for value in values:
            split_values.extend([item.strip() for item in value.split(',')])
        return split_values

    def update(self, headers: HeaderTypes | None = None) -> None:  # type: ignore[override]
        headers = Headers(headers)
        for key in headers.keys():
            if key in self:
                self.pop(key)
        self._list.extend(headers._list)

    def copy(self) -> Headers:
        return Headers(self, encoding=self.encoding)

    def __getitem__(self, key: str) -> str:
        """Return a single header value; repeated keys are joined with commas (RFC 7230 §3.2.2)."""
        normalized_key = key.lower().encode(self.encoding)
        items = [
            header_value.decode(self.encoding)
            for _, header_key, header_value in self._list
            if header_key == normalized_key
        ]
        if items:
            return ', '.join(items)
        raise KeyError(key)

    def __setitem__(self, key: str, value: str) -> None:
        """Set `key` to `value`, removing duplicates and retaining insertion order."""
        set_key = key.encode(self._encoding or 'utf-8')
        set_value = value.encode(self._encoding or 'utf-8')
        lookup_key = set_key.lower()
        found_indexes = [idx for idx, (_, item_key, _) in enumerate(self._list) if item_key == lookup_key]
        for idx in reversed(found_indexes[1:]):
            del self._list[idx]
        if found_indexes:
            idx = found_indexes[0]
            self._list[idx] = (set_key, lookup_key, set_value)
        else:
            self._list.append((set_key, lookup_key, set_value))

    def __delitem__(self, key: str) -> None:
        """Remove the header `key`."""
        del_key = key.lower().encode(self.encoding)
        pop_indexes = [idx for idx, (_, item_key, _) in enumerate(self._list) if item_key.lower() == del_key]
        if not pop_indexes:
            raise KeyError(key)
        for idx in reversed(pop_indexes):
            del self._list[idx]

    def __contains__(self, key: typing.Any) -> bool:
        header_key = key.lower().encode(self.encoding)
        return header_key in [key for _, key, _ in self._list]

    def __iter__(self) -> typing.Iterator[typing.Any]:
        return iter(self.keys())

    def __len__(self) -> int:
        return len(self._list)

    def __eq__(self, other: object) -> bool:
        try:
            other_headers = Headers(other)  # type: ignore[arg-type]
        except ValueError:
            return False
        self_list = [(key, value) for _, key, value in self._list]
        other_list = [(key, value) for _, key, value in other_headers._list]
        return sorted(self_list) == sorted(other_list)

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        encoding_str = f', encoding={self.encoding!r}' if self.encoding != 'ascii' else ''
        as_list = list(_obfuscate_sensitive_headers(self.multi_items()))
        as_dict = dict(as_list)
        if len(as_dict) == len(as_list):
            return f'{class_name}({as_dict!r}{encoding_str})'
        return f'{class_name}({as_list!r}{encoding_str})'
