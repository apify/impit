const URL_ = 'https://127.0.0.1:8443/';
const N = 1000;
const ROUNDS = 15;

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

async function makeImpit() {
    const { Impit } = await import('./index.js');
    const client = new Impit({ browser: 'chrome', ignoreTlsErrors: true });
    return async () => (await client.fetch(URL_)).text();
}

async function makeUndici() {
    const { Client } = await import('undici');
    const client = new Client('https://127.0.0.1:8443', {
        connect: { rejectUnauthorized: false },
        allowH2: true,
    });
    return async () => {
        const res = await client.request({ path: '/', method: 'GET' });
        return res.body.text();
    };
}

async function makeGotScraping() {
    const { gotScraping } = await import('got-scraping');
    const client = gotScraping.extend({
        https: { rejectUnauthorized: false },
        http2: false, // h2 session dies with GOAWAY(INTERNAL_ERROR) after ~1-2k requests
        retry: { limit: 0 },
    });
    return async () => (await client(URL_)).body;
}

async function makeNodeTlsClient() {
    const { Session, ClientIdentifier, initTLS } = await import('node-tls-client');
    await initTLS();
    const session = new Session({
        clientIdentifier: ClientIdentifier.chrome_131,
        insecureSkipVerify: true,
    });
    return async () => (await session.get(URL_)).text();
}

async function makeCycleTls() {
    const initCycleTLS = (await import('cycletls')).default;
    const cycleTLS = await initCycleTLS();
    return async () => {
        const res = await cycleTLS.get(URL_, { insecureSkipVerify: true, userAgent: 'Mozilla/5.0' });
        return typeof res.body === 'string' ? res.body : JSON.stringify(res.data);
    };
}

const FACTORIES = {
    impit: makeImpit,
    undici: makeUndici,
    'got-scraping': makeGotScraping,
    'node-tls-client': makeNodeTlsClient,
    cycletls: makeCycleTls,
};

const only = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const smoke = process.argv.includes('--smoke');

const clients = {};
for (const [name, factory] of Object.entries(FACTORIES)) {
    if (only.length && !only.includes(name)) continue;
    try {
        const fetch = await factory();
        const body = await fetch();
        if (!body || body.length < 900) throw new Error(`unexpected body: ${String(body).slice(0, 200)}`);
        clients[name] = fetch;
        if (smoke) console.log(`${name} ok ${body.length}`);
    } catch (e) {
        console.log(`${name} FAILED ${e.message}`);
    }
}

if (smoke) process.exit(0);

for (const fetch of Object.values(clients)) for (let i = 0; i < 200; i++) await fetch();

const samples = Object.fromEntries(Object.keys(clients).map((n) => [n, []]));
for (let r = 0; r < ROUNDS; r++) {
    for (const [name, fetch] of Object.entries(clients)) {
        const start = process.hrtime.bigint();
        try {
            for (let i = 0; i < N; i++) await fetch();
        } catch (e) {
            console.log(`${name} DIED in round ${r + 1}: ${e.message.slice(0, 80)}`);
            delete clients[name];
            delete samples[name];
            continue;
        }
        samples[name].push((N * 1e9) / Number(process.hrtime.bigint() - start));
    }
    console.error(`round ${r + 1}/${ROUNDS}`);
}

const median = (a) => [...a].sort((x, y) => x - y)[a.length >> 1];
const stdev = (a) => {
    const m = a.reduce((s, x) => s + x, 0) / a.length;
    return Math.sqrt(a.reduce((s, x) => s + (x - m) ** 2, 0) / (a.length - 1));
};

console.log('library\tbest\tmedian\tstdev');
for (const [name, rs] of Object.entries(samples).sort((a, b) => median(b[1]) - median(a[1]))) {
    console.log(`${name}\t${Math.max(...rs).toFixed(0)}\t${median(rs).toFixed(0)}\t${stdev(rs).toFixed(0)}`);
}
process.exit(0);
