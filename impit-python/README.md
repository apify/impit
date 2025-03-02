# `impit` for Python

`impit` is a Python package that provides bindings for the [`impit`](https://github.com/apify/impit) library.

It allows you to switch the TLS fingerprints and the HTTP headers of your requests, while still using the same API as `httpx` or `requests`.

## Installation

```bash
pip install impit
```

### Compatibility

| Operating System | Architecture | libc implementation | Prebuilt wheels available on PyPI |
|--|--|--|--|
| Linux | x86_64 | glibc | ✅ |
| Linux | x86_64 | musl | ✅ |
| macOS | x86_64 | N/A | ✅ |
| Windows | x86_64 | N/A | ✅ |
| macOS | arm64 | N/A | ✅ |
| Windows | arm64 | N/A | ✅ |
| Linux | arm64 | musl | ✅ |
| Linux | arm64 | glibc | ❌* |

*The prebuilt binaries for Linux on arm64 with `glibc` are WIP and not available as prebuilt wheels on PyPI yet. You can build the package from sources in this repository.

## Usage

```python
import asyncio
from impit import AsyncClient

async def main():
    impit = AsyncClient(http3=True, browser='firefox')

    response = await impit.get(
        "https://example.com",
    );

    print(response.status_code)
    print(response.text)
    print(response.http_version)

asyncio.run(main())
```

Impit implements the HTTPX client interface, so you can use it as a drop-in replacement for `httpx.AsyncClient`.
Note that the implementation is partial and some features may not be supported yet.
