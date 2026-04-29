const http = require('node:http');
const https = require('node:https');
const { Readable } = require('node:stream');
const tls = require('node:tls');
const zlib = require('node:zlib');

function getSetCookieValues(headers) {
    if (!headers) {
        return [];
    }

    if (typeof headers.getSetCookie === 'function') {
        return [...headers.getSetCookie()];
    }

    const nodeSetCookie = headers['set-cookie'];
    if (Array.isArray(nodeSetCookie)) {
        return [...nodeSetCookie];
    }

    if (typeof nodeSetCookie === 'string') {
        return [nodeSetCookie];
    }

    if (typeof headers.get === 'function') {
        const combined = headers.get('set-cookie');
        return combined ? [combined] : [];
    }

    return [];
}

function cloneHeadersWithSetCookie(headers) {
    const cloned = new Headers(headers);
    const setCookies = getSetCookieValues(headers);
    if (setCookies.length > 0) {
        Object.defineProperty(cloned, 'getSetCookie', {
            value: () => [...setCookies],
            configurable: true,
        });
    }

    return cloned;
}

const VANILLA_FALLBACK_ERROR_CODES = new Set([
    'ConnectTimeout',
    'PoolTimeout',
    'ConnectError',
    'ProxyError',
    'ProxyTunnelError',
    'ProxyAuthRequired',
    'UnsupportedProtocol',
]);

const VANILLA_FALLBACK_MESSAGE_PATTERNS = [
    /sendrequest/i,
    /hyper_util::client::legacy::Error\(\s*Connect/i,
    /unexpectedmessage/i,
    /SelectedUnusableCipherSuiteForVersion/i,
    /UnexpectedEof/i,
    /Failed to connect to the server/i,
    /Proxy CONNECT failed/i,
    /Proxy authentication required/i,
];

const VANILLA_FALLBACK_RESET_PATTERNS = [
    /connection reset by peer/i,
    /connection reset/i,
    /socket hang up/i,
];

const COMPATIBILITY_HEADER_ORDER = [
    'Host',
    'Connection',
    'Content-Length',
    'Upgrade-Insecure-Requests',
    'User-Agent',
    'Accept',
    'Sec-Fetch-Site',
    'Sec-Fetch-Mode',
    'Sec-Fetch-User',
    'Sec-Fetch-Dest',
    'Accept-Encoding',
    'Accept-Language',
    'Sec-Ch-Ua',
    'Sec-Ch-Ua-Mobile',
    'Sec-Ch-Ua-Platform',
    'Priority',
    'Cookie',
    'Origin',
    'Referer',
    'Content-Type',
];

const COMPATIBILITY_BROWSER_HEADERS = {
    chrome124: [
        ['Sec-Ch-Ua', '"Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99"'],
        ['Sec-Ch-Ua-Mobile', '?0'],
        ['Sec-Ch-Ua-Platform', '"macOS"'],
        ['Upgrade-Insecure-Requests', '1'],
        ['User-Agent', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36'],
        ['Accept', 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7'],
        ['Sec-Fetch-Site', 'none'],
        ['Sec-Fetch-Mode', 'navigate'],
        ['Sec-Fetch-User', '?1'],
        ['Sec-Fetch-Dest', 'document'],
        ['Accept-Encoding', 'gzip, deflate, br'],
        ['Accept-Language', 'en-US,en;q=0.9'],
        ['Priority', 'u=0, i'],
    ],
};

function getNativeErrorCode(error) {
    const match = error?.message?.match(/^(\w+): /);
    return match?.[1];
}

function shouldRetryWithVanillaFallback(error) {
    const code = getNativeErrorCode(error);
    if (code && VANILLA_FALLBACK_ERROR_CODES.has(code)) {
        return true;
    }

    const message = error?.message ?? '';
    if (VANILLA_FALLBACK_MESSAGE_PATTERNS.some((pattern) => pattern.test(message))) {
        return true;
    }

    return VANILLA_FALLBACK_RESET_PATTERNS.some((pattern) => pattern.test(message))
        && /sendrequest|kind:\s*request|hyper_util::client::legacy::Error\(\s*Connect/i.test(message);
}

function toCompatibilityHeaderCase(header) {
    if (header.toLowerCase().startsWith('x-')) {
        return header;
    }

    return header
        .split('-')
        .map((part) => part ? part[0].toUpperCase() + part.slice(1).toLowerCase() : part)
        .join('-');
}

function orderCompatibilityHeaders(headers) {
    const orderIndex = new Map(COMPATIBILITY_HEADER_ORDER.map((name, index) => [name, index]));

    return headers.sort(([left], [right]) => {
        const leftIndex = orderIndex.has(left) ? orderIndex.get(left) : Number.MAX_SAFE_INTEGER;
        const rightIndex = orderIndex.has(right) ? orderIndex.get(right) : Number.MAX_SAFE_INTEGER;
        if (leftIndex !== rightIndex) {
            return leftIndex - rightIndex;
        }

        return left.localeCompare(right);
    });
}

function getCompatibilityBrowserProfile(browser) {
    switch (browser) {
        case 'chrome':
        case 'chrome124':
            return 'chrome124';
        default:
            return null;
    }
}

function createCompatibilityDecodeStream(contentEncoding, responseStream) {
    const encodings = String(contentEncoding ?? '')
        .split(',')
        .map((value) => value.trim().toLowerCase())
        .filter(Boolean);

    if (encodings.length === 0) {
        return {
            decoded: false,
            stream: responseStream,
        };
    }

    const transforms = [];
    for (const encoding of [...encodings].reverse()) {
        if (encoding === 'gzip' || encoding === 'x-gzip') {
            transforms.push(zlib.createGunzip());
            continue;
        }

        if (encoding === 'br') {
            transforms.push(zlib.createBrotliDecompress());
            continue;
        }

        if (encoding === 'deflate') {
            transforms.push(zlib.createInflate());
            continue;
        }

        return {
            decoded: false,
            stream: responseStream,
        };
    }

    const decodedStream = transforms.reduce(
        (stream, transform) => stream.pipe(transform),
        responseStream,
    );

    return {
        decoded: true,
        stream: decodedStream,
    };
}

class VanillaFallbackController {
    #compatibilityProfile;
    #proxyUrl;
    #http3Enabled;
    #localAddress;
    #ignoreTlsErrors;

    constructor(options) {
        this.#compatibilityProfile = getCompatibilityBrowserProfile(options?.browser);
        this.#proxyUrl = options?.proxyUrl;
        this.#http3Enabled = Boolean(options?.http3);
        this.#localAddress = options?.localAddress;
        this.#ignoreTlsErrors = Boolean(options?.ignoreTlsErrors);
    }

    shouldRetry(error) {
        return shouldRetryWithVanillaFallback(error);
    }

    canUseCompatibilityTransport(url) {
        if (!this.#compatibilityProfile || this.#http3Enabled || url.protocol !== 'https:') {
            return false;
        }

        if (!this.#proxyUrl) {
            return true;
        }

        try {
            const proxyProtocol = new URL(this.#proxyUrl).protocol;
            return proxyProtocol === 'http:' || proxyProtocol === 'https:';
        } catch {
            return false;
        }
    }

    #buildCompatibilityHeaders(url, headers, body) {
        const normalizedMap = new Map();
        const pushHeader = (key, value) => {
            const headerName = toCompatibilityHeaderCase(key);
            normalizedMap.set(headerName.toLowerCase(), [headerName, value]);
        };

        for (const [key, value] of COMPATIBILITY_BROWSER_HEADERS[this.#compatibilityProfile] ?? []) {
            pushHeader(key, value);
        }

        for (const [key, value] of headers ?? []) {
            pushHeader(key, value);
        }

        if (!normalizedMap.has('host')) {
            normalizedMap.set('host', ['Host', url.host]);
        }

        if (!normalizedMap.has('connection')) {
            normalizedMap.set('connection', ['Connection', 'keep-alive']);
        }

        if (!normalizedMap.has('content-length') && !normalizedMap.has('transfer-encoding')) {
            const bodyLength = body ? body.length : 0;
            normalizedMap.set('content-length', ['Content-Length', String(bodyLength)]);
        }

        return orderCompatibilityHeaders([...normalizedMap.values()]);
    }

    async #createProxyTunnel(url, options = {}) {
        const proxy = new URL(this.#proxyUrl);
        const proxyPort = Number(proxy.port || (proxy.protocol === 'https:' ? 443 : 80));
        const targetHost = `${url.hostname}:${url.port || 443}`;
        const headers = { host: targetHost };
        const basicAuth = proxy.username
            ? `Basic ${Buffer.from(
                `${decodeURIComponent(proxy.username)}:${decodeURIComponent(proxy.password)}`,
            ).toString('base64')}`
            : null;
        if (basicAuth) {
            headers['proxy-authorization'] = basicAuth;
        }

        const requestFactory = proxy.protocol === 'https:' ? https.request : http.request;

        const tunnelSocket = await new Promise((resolve, reject) => {
            let upstreamSocket = null;
            const cleanup = () => {
                options.signal?.removeEventListener?.('abort', onAbort);
            };
            const onAbort = () => {
                const reason = options.signal?.reason ?? new Error('The operation was aborted');
                connectRequest.destroy(reason);
                upstreamSocket?.destroy(reason);
            };
            const connectRequest = requestFactory({
                host: proxy.hostname,
                port: proxyPort,
                localAddress: this.#localAddress,
                method: 'CONNECT',
                path: targetHost,
                headers,
                agent: false,
                rejectUnauthorized: true,
            });
            if (options.timeout) {
                connectRequest.setTimeout(options.timeout, () => {
                    connectRequest.destroy(new Error(`Proxy CONNECT timed out after ${options.timeout}ms`));
                });
            }
            if (options.signal) {
                if (options.signal.aborted) {
                    onAbort();
                    return;
                }
                options.signal.addEventListener('abort', onAbort, { once: true });
            }

            connectRequest.once('connect', (response, socket, head) => {
                upstreamSocket = socket;
                if (response.statusCode !== 200 || head.length > 0) {
                    cleanup();
                    socket.destroy();
                    reject(new Error(`Proxy CONNECT failed with status ${response.statusCode || 'unknown'}`));
                    return;
                }

                const secureSocket = tls.connect({
                    socket,
                    servername: url.hostname,
                    rejectUnauthorized: !this.#ignoreTlsErrors,
                }, () => {
                    if (options.signal?.aborted) {
                        const reason = options.signal.reason ?? new Error('The operation was aborted');
                        cleanup();
                        secureSocket.destroy(reason);
                        reject(reason);
                        return;
                    }
                    cleanup();
                    resolve(secureSocket);
                });
                secureSocket.once('error', (error) => {
                    cleanup();
                    reject(error);
                });
            });

            connectRequest.once('error', (error) => {
                cleanup();
                reject(error);
            });
            connectRequest.end();
        });

        return tunnelSocket;
    }

    async fetchWithCompatibilityTransport(rawUrl, options) {
        const url = new URL(rawUrl);
        const isHttps = url.protocol === 'https:';
        const body = options.body ? Buffer.from(options.body) : null;
        const orderedHeaders = this.#buildCompatibilityHeaders(url, options.headers, body);
        const directHttpsAgent = !this.#proxyUrl && isHttps
            ? new https.Agent({
                keepAlive: true,
                scheduling: 'lifo',
                timeout: 5_000,
                noDelay: true,
            })
            : null;
        let activeSocket = null;
        let request = null;

        const requestOptions = {
            protocol: url.protocol,
            hostname: url.hostname,
            port: url.port || (isHttps ? 443 : 80),
            localAddress: this.#localAddress,
            path: `${url.pathname}${url.search}`,
            method: options.method || 'GET',
            rejectUnauthorized: !this.#ignoreTlsErrors,
            agent: directHttpsAgent ?? undefined,
            createConnection: this.#proxyUrl && isHttps
                ? (_connectOptions, callback) => {
                    this.#createProxyTunnel(url, {
                        signal: options.signal,
                        timeout: options.timeout,
                    }).then(
                        (socket) => {
                            if (options.signal?.aborted) {
                                const reason = options.signal.reason ?? new Error('The operation was aborted');
                                socket.destroy(reason);
                                callback(reason);
                                return;
                            }
                            activeSocket = socket;
                            callback(null, socket);
                        },
                        (error) => callback(error),
                    );
                }
                : undefined,
        };

        const transport = isHttps ? https : http;

        return new Promise((resolve, reject) => {
            const destroyDirectAgent = () => {
                directHttpsAgent?.destroy();
            };
            const cleanup = () => {
                options.signal?.removeEventListener?.('abort', onAbort);
            };
            const onAbort = () => {
                const reason = options.signal?.reason ?? new Error('The operation was aborted');
                request?.destroy(reason);
                activeSocket?.destroy(reason);
                destroyDirectAgent();
            };

            request = transport.request(requestOptions, (response) => {
                activeSocket = response.socket ?? activeSocket;
                const { decoded, stream } = createCompatibilityDecodeStream(
                    response.headers['content-encoding'],
                    response,
                );
                stream.once('close', destroyDirectAgent);
                stream.once('error', (error) => {
                    cleanup();
                    destroyDirectAgent();
                    reject(error);
                });
                response.once('error', (error) => {
                    cleanup();
                    destroyDirectAgent();
                    reject(error);
                });

                const compatibilityHeaders = cloneHeadersWithSetCookie(response.headers);
                if (decoded) {
                    compatibilityHeaders.delete('content-encoding');
                    compatibilityHeaders.delete('content-length');
                }

                const compatibilityResponse = new Response(Readable.toWeb(stream), {
                    status: response.statusCode,
                    statusText: response.statusMessage,
                    headers: compatibilityHeaders,
                });

                const arrayBuffer = compatibilityResponse.arrayBuffer.bind(compatibilityResponse);
                Object.defineProperty(compatibilityResponse, 'bytes', {
                    value: async () => new Uint8Array(await arrayBuffer()),
                    configurable: true,
                });
                Object.defineProperty(compatibilityResponse, 'decodeBuffer', {
                    value: (buffer) => Buffer.from(buffer).toString('utf8'),
                    configurable: true,
                });
                Object.defineProperty(compatibilityResponse, 'abort', {
                    value: () => {
                        request.destroy();
                        activeSocket?.destroy();
                        response.destroy();
                        stream.destroy?.();
                        destroyDirectAgent();
                    },
                    configurable: true,
                });
                Object.defineProperty(compatibilityResponse, 'url', {
                    value: rawUrl,
                    enumerable: true,
                    configurable: true,
                });
                Object.defineProperty(compatibilityResponse, '__impitCompat', {
                    value: true,
                    configurable: true,
                });

                cleanup();
                resolve(compatibilityResponse);
            });

            if (options.timeout) {
                request.setTimeout(options.timeout, () => {
                    request.destroy(new Error(`Compatibility transport timed out after ${options.timeout}ms`));
                });
            }
            if (options.signal) {
                if (options.signal.aborted) {
                    onAbort();
                    return;
                }
                options.signal.addEventListener('abort', onAbort, { once: true });
            }

            request.on('socket', (socket) => {
                activeSocket = socket;
                if (options.signal?.aborted) {
                    socket.destroy(options.signal.reason ?? new Error('The operation was aborted'));
                }
            });
            request.on('error', (error) => {
                cleanup();
                destroyDirectAgent();
                reject(error);
            });

            for (const [key, value] of orderedHeaders) {
                request.setHeader(key, value);
            }

            if (body) {
                request.end(body);
            } else {
                request.end('');
            }
        });
    }
}

module.exports = {
    VanillaFallbackController,
    cloneHeadersWithSetCookie,
};
