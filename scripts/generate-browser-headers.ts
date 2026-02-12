/**
 * Generates src/browser-headers.ts with modern user agents and browser-like headers
 * for HTML document fetching. Run: npm run generate:headers
 *
 * Next-gen fingerprint strategy:
 *   - US-only locale variants (arrow.com is US-targeted)
 *   - Rebalanced browser mix: ~35% Firefox, ~30% Chrome, ~20% Safari, ~15% Edge
 *   - Each preset includes an `os` field for TLS fingerprint matching
 *   - No impossible combos (no Safari on Linux/Windows)
 *   - Fewer but higher-quality presets focused on recent browser versions
 */

import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// ---------------------------------------------------------------------------
// US-only locale variants — realistic Accept-Language diversity
// Simulates different US user profiles (monolingual, bilingual, etc.)
// ---------------------------------------------------------------------------
const US_LOCALES = [
    { id: 'en', lang: 'en-US,en;q=0.9' },
    { id: 'en2', lang: 'en-US,en;q=0.8' },
    { id: 'en-es', lang: 'en-US,en;q=0.9,es;q=0.8' },           // US bilingual English+Spanish
    { id: 'en-es2', lang: 'en-US,en;q=0.9,es-US;q=0.8,es;q=0.7' }, // US Spanish variant
    { id: 'en-zh', lang: 'en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7' }, // US bilingual English+Chinese
    { id: 'en-fr', lang: 'en-US,en;q=0.9,fr;q=0.8' },           // US bilingual English+French
    { id: 'en-vi', lang: 'en-US,en;q=0.9,vi;q=0.8' },           // US bilingual English+Vietnamese
    { id: 'en-ko', lang: 'en-US,en;q=0.9,ko;q=0.8' },           // US bilingual English+Korean
] as const;

// ---------------------------------------------------------------------------
// Browser definitions — each entry produces a UNIQUE User-Agent string.
// Chrome UAs include real build numbers (Chrome/MAJOR.0.BUILD.PATCH).
// ---------------------------------------------------------------------------
type BrowserDef = {
    browser: 'chrome' | 'firefox' | 'safari' | 'edge';
    /** Major version used in headers (sec-ch-ua) */
    version: string;
    /** Full version string for the UA (e.g. "131.0.6778.86") */
    fullVersion: string;
    /** The "Not-A.Brand" style token for sec-ch-ua (Chrome/Edge only) */
    notABrand?: string;
    /** Chromium version that Edge is based on (Edge only) */
    chromiumVersion?: string;
};

// ~12 Chrome entries — focus on recent versions (131-144)
const CHROME_VERSIONS: BrowserDef[] = [
    { browser: 'chrome', version: '131', fullVersion: '131.0.6778.86', notABrand: '"Not_A Brand";v="24"' },
    { browser: 'chrome', version: '131', fullVersion: '131.0.6778.140', notABrand: '"Not_A Brand";v="24"' },
    { browser: 'chrome', version: '132', fullVersion: '132.0.6834.84', notABrand: '"Not_A Brand";v="24"' },
    { browser: 'chrome', version: '132', fullVersion: '132.0.6834.160', notABrand: '"Not_A Brand";v="24"' },
    { browser: 'chrome', version: '133', fullVersion: '133.0.6917.65', notABrand: '"Not(A:Brand";v="99"' },
    { browser: 'chrome', version: '133', fullVersion: '133.0.6917.134', notABrand: '"Not(A:Brand";v="99"' },
    { browser: 'chrome', version: '134', fullVersion: '134.0.6998.45', notABrand: '"Not(A:Brand";v="99"' },
    { browser: 'chrome', version: '134', fullVersion: '134.0.6998.118', notABrand: '"Not(A:Brand";v="99"' },
    { browser: 'chrome', version: '136', fullVersion: '136.0.7103.93', notABrand: '"Not.A/Brand";v="99"' },
    { browser: 'chrome', version: '136', fullVersion: '136.0.7103.114', notABrand: '"Not.A/Brand";v="99"' },
    { browser: 'chrome', version: '142', fullVersion: '142.0.7301.68', notABrand: '"Not_A Brand";v="99"' },
    { browser: 'chrome', version: '144', fullVersion: '144.0.7250.47', notABrand: '"Not-A.Brand";v="24"' },
];

// ~18 Firefox entries — expanded range for maximum diversity
const FIREFOX_VERSIONS: BrowserDef[] = [
    { browser: 'firefox', version: '120', fullVersion: '120.0' },
    { browser: 'firefox', version: '121', fullVersion: '121.0' },
    { browser: 'firefox', version: '122', fullVersion: '122.0' },
    { browser: 'firefox', version: '124', fullVersion: '124.0.2' },
    { browser: 'firefox', version: '125', fullVersion: '125.0.3' },
    { browser: 'firefox', version: '126', fullVersion: '126.0' },
    { browser: 'firefox', version: '127', fullVersion: '127.0.2' },
    { browser: 'firefox', version: '128', fullVersion: '128.0' },     // ESR
    { browser: 'firefox', version: '128', fullVersion: '128.0.3' },   // ESR point release
    { browser: 'firefox', version: '130', fullVersion: '130.0' },
    { browser: 'firefox', version: '131', fullVersion: '131.0.3' },
    { browser: 'firefox', version: '133', fullVersion: '133.0' },
    { browser: 'firefox', version: '135', fullVersion: '135.0' },
    { browser: 'firefox', version: '136', fullVersion: '136.0.4' },
    { browser: 'firefox', version: '140', fullVersion: '140.0' },     // ESR
    { browser: 'firefox', version: '140', fullVersion: '140.0.4' },   // ESR point release
    { browser: 'firefox', version: '144', fullVersion: '144.0' },
    { browser: 'firefox', version: '147', fullVersion: '147.0' },
];

// ~12 Safari versions — macOS only
const SAFARI_VERSIONS: BrowserDef[] = [
    { browser: 'safari', version: '17', fullVersion: '17.0' },
    { browser: 'safari', version: '17', fullVersion: '17.2' },
    { browser: 'safari', version: '17', fullVersion: '17.4' },
    { browser: 'safari', version: '17', fullVersion: '17.5' },
    { browser: 'safari', version: '17', fullVersion: '17.6' },
    { browser: 'safari', version: '18', fullVersion: '18.0' },
    { browser: 'safari', version: '18', fullVersion: '18.1' },
    { browser: 'safari', version: '18', fullVersion: '18.2' },
    { browser: 'safari', version: '18', fullVersion: '18.3' },
    { browser: 'safari', version: '18', fullVersion: '18.4' },
    { browser: 'safari', version: '18', fullVersion: '18.5' },
    { browser: 'safari', version: '18', fullVersion: '18.6' },
];

// ~6 Edge versions — recent only
const EDGE_VERSIONS: BrowserDef[] = [
    { browser: 'edge', version: '131', fullVersion: '131.0.2903.51', chromiumVersion: '131', notABrand: '"Not_A Brand";v="24"' },
    { browser: 'edge', version: '132', fullVersion: '132.0.2957.127', chromiumVersion: '132', notABrand: '"Not_A Brand";v="24"' },
    { browser: 'edge', version: '133', fullVersion: '133.0.3065.69', chromiumVersion: '133', notABrand: '"Not(A:Brand";v="99"' },
    { browser: 'edge', version: '134', fullVersion: '134.0.3124.51', chromiumVersion: '134', notABrand: '"Not(A:Brand";v="99"' },
    { browser: 'edge', version: '136', fullVersion: '136.0.3240.50', chromiumVersion: '136', notABrand: '"Not.A/Brand";v="99"' },
    { browser: 'edge', version: '136', fullVersion: '136.0.3240.92', chromiumVersion: '136', notABrand: '"Not.A/Brand";v="99"' },
];

const ALL_BROWSERS: BrowserDef[] = [...CHROME_VERSIONS, ...FIREFOX_VERSIONS, ...SAFARI_VERSIONS, ...EDGE_VERSIONS];

// ---------------------------------------------------------------------------
// Platforms — with OS field for TLS fingerprint matching
// ---------------------------------------------------------------------------
type PlatformDef = {
    id: string;
    /** OS identifier passed to impit for TLS fingerprint selection */
    os: 'windows' | 'macos' | 'linux';
    chromeUA: string;
    firefoxUA: string;
    /** Safari UA platform string (only macOS platforms are used for Safari) */
    safariUA?: string;
    /** macOS version string for Safari Version/X.Y format */
    safariMacOSVersion?: string;
    secChUaPlatform: string;
    /** Whether this platform is macOS (Safari only runs on macOS/iOS) */
    isMac?: boolean;
};

const PLATFORMS: PlatformDef[] = [
    {
        id: 'win10',
        os: 'windows',
        chromeUA: 'Windows NT 10.0; Win64; x64',
        firefoxUA: 'Windows NT 10.0; Win64; x64',
        secChUaPlatform: '"Windows"',
    },
    {
        id: 'win11',
        os: 'windows',
        chromeUA: 'Windows NT 10.0; Win64; x64',  // Win11 still reports NT 10.0 in UA
        firefoxUA: 'Windows NT 10.0; Win64; x64',
        secChUaPlatform: '"Windows"',
    },
    {
        id: 'mac1015',
        os: 'macos',
        chromeUA: 'Macintosh; Intel Mac OS X 10_15_7',
        firefoxUA: 'Macintosh; Intel Mac OS X 10.15',
        safariUA: 'Macintosh; Intel Mac OS X 10_15_7',
        safariMacOSVersion: '10_15_7',
        secChUaPlatform: '"macOS"',
        isMac: true,
    },
    {
        id: 'mac13',
        os: 'macos',
        chromeUA: 'Macintosh; Intel Mac OS X 13_6_9',
        firefoxUA: 'Macintosh; Intel Mac OS X 13.6',
        safariUA: 'Macintosh; Intel Mac OS X 13_6_9',
        safariMacOSVersion: '13_6_9',
        secChUaPlatform: '"macOS"',
        isMac: true,
    },
    {
        id: 'mac14',
        os: 'macos',
        chromeUA: 'Macintosh; Intel Mac OS X 14_7_1',
        firefoxUA: 'Macintosh; Intel Mac OS X 14.7',
        safariUA: 'Macintosh; Intel Mac OS X 14_7_1',
        safariMacOSVersion: '14_7_1',
        secChUaPlatform: '"macOS"',
        isMac: true,
    },
    {
        id: 'mac15',
        os: 'macos',
        chromeUA: 'Macintosh; Intel Mac OS X 15_2',
        firefoxUA: 'Macintosh; Intel Mac OS X 15.2',
        safariUA: 'Macintosh; Intel Mac OS X 15_2',
        safariMacOSVersion: '15_2',
        secChUaPlatform: '"macOS"',
        isMac: true,
    },
    {
        id: 'linux',
        os: 'linux',
        chromeUA: 'X11; Linux x86_64',
        firefoxUA: 'X11; Linux x86_64',
        secChUaPlatform: '"Linux"',
    },
];

// ---------------------------------------------------------------------------
// Preset generation
// ---------------------------------------------------------------------------
type PresetInput = {
    id: string;
    browser: string;
    version: string;
    platform: string;
    os: string;
    userAgent: string;
    headers: Record<string, string>;
};

function buildChromeUA(platformUA: string, fullVersion: string): string {
    return `Mozilla/5.0 (${platformUA}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${fullVersion} Safari/537.36`;
}

function buildFirefoxUA(platformUA: string, fullVersion: string): string {
    return `Mozilla/5.0 (${platformUA}; rv:${fullVersion}) Gecko/20100101 Firefox/${fullVersion}`;
}

function buildSafariUA(platformUA: string, fullVersion: string): string {
    return `Mozilla/5.0 (${platformUA}) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/${fullVersion} Safari/605.1.15`;
}

function buildEdgeUA(platformUA: string, edgeFullVersion: string, chromiumVersion: string): string {
    return `Mozilla/5.0 (${platformUA}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromiumVersion}.0.0.0 Safari/537.36 Edg/${edgeFullVersion}`;
}

function generatePresets(): PresetInput[] {
    const presets: PresetInput[] = [];

    for (const locale of US_LOCALES) {
        for (const browserDef of ALL_BROWSERS) {
            for (const platform of PLATFORMS) {
                const { browser } = browserDef;
                const isChrome = browser === 'chrome';
                const isFirefox = browser === 'firefox';
                const isSafari = browser === 'safari';
                const isEdge = browser === 'edge';

                // Safari only runs on macOS — skip non-Mac platforms entirely
                if (isSafari && !platform.isMac) continue;

                // Win11 shares the same NT 10.0 UA as Win10 — skip duplicates
                // by only pairing Win11 with even-indexed browsers
                if (platform.id === 'win11') {
                    const idx = ALL_BROWSERS.indexOf(browserDef);
                    if (idx % 2 === 0) continue; // skip half to avoid duplicate UAs
                }

                let userAgent: string;
                if (isChrome) {
                    userAgent = buildChromeUA(platform.chromeUA, browserDef.fullVersion);
                } else if (isFirefox) {
                    userAgent = buildFirefoxUA(platform.firefoxUA, browserDef.fullVersion);
                } else if (isSafari) {
                    userAgent = buildSafariUA(platform.safariUA!, browserDef.fullVersion);
                } else {
                    // Edge
                    userAgent = buildEdgeUA(platform.chromeUA, browserDef.fullVersion, browserDef.chromiumVersion!);
                }

                // Use fullVersion in the id to distinguish builds of the same major version
                const safeVersion = browserDef.fullVersion.replace(/\./g, '_');
                const id = `${browserDef.browser}-${safeVersion}-${platform.id}-${locale.id}`;

                const headers: Record<string, string> = {};

                if (isChrome) {
                    headers['sec-ch-ua'] =
                        `"Chromium";v="${browserDef.version}", "Google Chrome";v="${browserDef.version}", ${browserDef.notABrand}`;
                    headers['sec-ch-ua-mobile'] = '?0';
                    headers['sec-ch-ua-platform'] = platform.secChUaPlatform;
                } else if (isEdge) {
                    headers['sec-ch-ua'] =
                        `"Microsoft Edge";v="${browserDef.version}", "Chromium";v="${browserDef.chromiumVersion}", ${browserDef.notABrand}`;
                    headers['sec-ch-ua-mobile'] = '?0';
                    headers['sec-ch-ua-platform'] = platform.secChUaPlatform;
                }
                // Safari and Firefox do not send sec-ch-ua headers

                headers['Upgrade-Insecure-Requests'] = '1';
                headers['User-Agent'] = userAgent;

                if (isChrome || isEdge) {
                    headers['Accept'] =
                        'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8';
                } else if (isSafari) {
                    // Safari has a simpler Accept header
                    headers['Accept'] =
                        'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8';
                } else {
                    // Firefox
                    headers['Accept'] =
                        'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8';
                }

                headers['Sec-Fetch-Site'] = 'none';
                headers['Sec-Fetch-Mode'] = 'navigate';
                // Safari does not send Sec-Fetch-User
                if (!isSafari) {
                    headers['Sec-Fetch-User'] = '?1';
                }
                headers['Sec-Fetch-Dest'] = 'document';
                headers['Accept-Language'] = locale.lang;

                const majorVersion = parseInt(browserDef.version, 10);
                if ((isChrome || isEdge) && majorVersion >= 131) {
                    headers['Accept-Encoding'] = 'gzip, deflate, br, zstd';
                } else if (isFirefox && majorVersion >= 133) {
                    // Firefox 133+ supports zstd content encoding
                    headers['Accept-Encoding'] = 'gzip, deflate, br, zstd';
                } else {
                    headers['Accept-Encoding'] = 'gzip, deflate, br';
                }

                // Firefox-specific: add Priority header for HTTP/2
                if (isFirefox) {
                    headers['Priority'] = 'u=0, i';
                }
                // Chrome 125+ includes priority header (lowercase)
                if ((isChrome || isEdge) && majorVersion >= 125) {
                    headers['priority'] = 'u=0, i';
                }

                presets.push({
                    id,
                    browser: browserDef.browser,
                    version: browserDef.version,
                    platform: platform.id,
                    os: platform.os,
                    userAgent,
                    headers,
                });
            }
        }
    }

    return presets;
}

// ---------------------------------------------------------------------------
// Output generation
// ---------------------------------------------------------------------------
function headersToLiteral(headers: Record<string, string>): string {
    const lines = Object.entries(headers).map(
        ([k, v]) => `            ${JSON.stringify(k)}: ${JSON.stringify(v)},`,
    );
    return `{\n${lines.join('\n')}\n        }`;
}

function generateOutput(): string {
    const now = new Date().toISOString().split('T')[0];
    const presets = generatePresets();

    // Count unique UAs
    const uniqueUAs = new Set(presets.map((p) => p.userAgent));

    // Count per browser
    const browserCounts: Record<string, number> = {};
    for (const p of presets) {
        browserCounts[p.browser] = (browserCounts[p.browser] || 0) + 1;
    }
    const total = presets.length;
    const browserPcts = Object.entries(browserCounts)
        .map(([b, c]) => `${b}: ${c} (${((c / total) * 100).toFixed(0)}%)`)
        .join(', ');

    const presetLines = presets
        .map(
            (p) => `    {
        id: ${JSON.stringify(p.id)},
        browser: ${JSON.stringify(p.browser)},
        version: ${JSON.stringify(p.version)},
        platform: ${JSON.stringify(p.platform)},
        os: ${JSON.stringify(p.os)},
        userAgent: ${JSON.stringify(p.userAgent)},
        headers: ${headersToLiteral(p.headers)},
    }`,
        )
        .join(',\n');

    return `/**
 * Auto-generated by scripts/generate-browser-headers.ts
 * Generated: ${now}
 *
 * ${presets.length} presets with ${uniqueUAs.size} UNIQUE User-Agent strings.
 * Distribution: ${browserPcts}
 * US-only locales with ${US_LOCALES.length} Accept-Language variants.
 * Each preset includes an \`os\` field for TLS fingerprint matching.
 */

export type BrowserPreset = {
    id: string;
    browser: string;
    version: string;
    platform: string;
    /** OS identifier for TLS fingerprint matching: 'windows' | 'macos' | 'linux' */
    os: string;
    userAgent: string;
    headers: Record<string, string>;
};

// @ts-ignore - Array literal is too large for TypeScript's union type limit
export const BROWSER_PRESETS: readonly BrowserPreset[] = [
${presetLines}
] as BrowserPreset[];

/**
 * Returns a random browser preset for header diversity.
 */
export function getRandomPreset(): BrowserPreset {
    const preset = BROWSER_PRESETS[Math.floor(Math.random() * BROWSER_PRESETS.length)];
    return preset!;
}

/**
 * Returns a preset by id, or undefined if not found.
 */
export function getPresetById(id: string): BrowserPreset | undefined {
    return BROWSER_PRESETS.find((p) => p.id === id);
}

/**
 * Returns a deterministic preset for a session id.
 * Same session always gets the same preset for header consistency.
 */
export function getPresetForSession(sessionId: string): BrowserPreset {
    let hash = 5381;
    for (let i = 0; i < sessionId.length; i++) {
        hash = hash * 33 + sessionId.charCodeAt(i);
    }
    const index = Math.abs(hash) % BROWSER_PRESETS.length;
    return BROWSER_PRESETS[index]!;
}

/**
 * Maps a preset browser+version to the closest impit browser profile string.
 * Each distinct impit profile has a unique TLS fingerprint (JA3/JA4) and HTTP/2 SETTINGS.
 *
 * Available impit profiles:
 *   Chrome:  chrome131, chrome133, chrome136, chrome142
 *   Firefox: firefox128, firefox133, firefox135, firefox144
 *   Safari:  safari17, safari18, safari184
 *   Edge:    edge131, edge136
 */
export function mapToImpitBrowser(browser: string, version: string): string {
    const v = parseInt(version, 10);

    if (browser === 'firefox') {
        if (v <= 128) return 'firefox128';
        if (v <= 133) return 'firefox133';
        if (v <= 139) return 'firefox135';
        return 'firefox144';
    }

    if (browser === 'safari') {
        if (v <= 17) return 'safari17';
        if (v <= 18) return 'safari18';
        return 'safari184';
    }

    if (browser === 'edge') {
        if (v <= 131) return 'edge131';
        return 'edge136';
    }

    // Chrome — spread across impit profiles for TLS diversity
    if (v <= 131) return 'chrome131';
    if (v <= 133) return 'chrome133';
    if (v <= 136) return 'chrome136';
    return 'chrome142';
}

/**
 * Maps a platform id to the OS string expected by impit.
 */
export function mapPlatformToOs(platform: string): string {
    if (platform.startsWith('win')) return 'windows';
    if (platform.startsWith('mac')) return 'macos';
    return 'linux';
}

/**
 * Maps a preset browser+version to the closest curl-impersonate native preset.
 * curl-impersonate only supports: chrome-110, chrome-116, firefox-109, firefox-117.
 * We spread usage across all 4 for TLS diversity.
 */
export type CurlPreset = 'chrome-110' | 'chrome-116' | 'firefox-109' | 'firefox-117';

export function mapToCurlPreset(browser: string, version: string): CurlPreset {
    const v = parseInt(version, 10);

    if (browser === 'firefox') {
        return v <= 128 ? 'firefox-109' : 'firefox-117';
    }

    // Alternate between chrome-110 and chrome-116
    return v <= 126 ? 'chrome-110' : 'chrome-116';
}

/**
 * Maps a preset browser+version to a got-scraping TLS profile.
 * got-scraping uses Node.js OpenSSL, producing a third family of JA3/JA4 fingerprints.
 */
export type GotTlsProfile = 'chrome-auto' | 'firefox-auto' | 'safari-auto' | 'chrome-shuffled' | 'edge-auto';

export function mapToGotScrapingTlsProfile(browser: string, version: string): GotTlsProfile {
    const v = parseInt(version, 10);

    if (browser === 'firefox') {
        return 'firefox-auto';
    }

    // Spread Chrome across 4 different TLS profiles
    const profiles: GotTlsProfile[] = ['chrome-auto', 'chrome-shuffled', 'safari-auto', 'edge-auto'];
    return profiles[v % profiles.length]!;
}
`;
}

const outPath = join(__dirname, '..', 'src', 'browser-headers.ts');
const presets = generatePresets();
const uniqueUAs = new Set(presets.map((p) => p.userAgent));

// Count per browser for console output
const browserCounts: Record<string, number> = {};
for (const p of presets) {
    browserCounts[p.browser] = (browserCounts[p.browser] || 0) + 1;
}
const total = presets.length;
const browserPcts = Object.entries(browserCounts)
    .map(([b, c]) => `${b}: ${c} (${((c / total) * 100).toFixed(0)}%)`)
    .join(', ');

writeFileSync(outPath, generateOutput(), 'utf-8');
console.log(`Generated ${outPath} with ${presets.length} presets and ${uniqueUAs.size} unique User-Agents`);
console.log(`Distribution: ${browserPcts}`);
