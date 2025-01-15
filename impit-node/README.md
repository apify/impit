# `impit` for JavaScript

`impit` is a Node.JS module that provides bindings for the [`impit`](https://github.com/apify/impit) library.

It allows you to switch the TLS fingerprints and the HTTP headers of your requests, while still using the same API as the built-in (since Node.JS 18) `fetch` function.

## Installation

```bash
npm install impit
```

Installing the root package (`impit`) with the package manager of your choice will also install the correct prebuilt binary for your platform.

### Compatibility

| Operating System | Architecture | libc implementation | Prebuilt binaries available | 
|--|--|--|--|
| Linux | x86_64 | glibc | ✅ |
| Linux | x86_64 | musl | ✅ |
| macOS | x86_64 | N/A | ✅ |
| Windows | x86_64 | N/A | ✅ |
| macOS | arm64 | N/A | ✅ |
| Windows | arm64 | N/A | ✅ |
| Linux | arm64 | glibc | ❌* |
| Linux | arm64 | musl | ❌* |

*The prebuilt binaries for Linux on arm64 are WIP and not available on npm yet. You can build the package from sources in this repository.

## Usage

```typescript
import { Impit } from 'impit';

// Set up the Impit instance
const impit = new Impit({
    browser: "Chrome", // or "Firefox"
    proxyUrl: "http://localhost:8080",
    ignoreTlsErrors: true,
});

// Use the `fetch` method as you would with the built-in `fetch` function
const response = await impit.fetch("https://example.com");

console.log(response.status);
console.log(response.headers);
console.log(await response.text());
// console.log(await response.json());
// ...
```

