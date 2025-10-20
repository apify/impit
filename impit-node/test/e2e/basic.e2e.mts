import { Impit } from "../../index.wrapper.js";
import assert from "node:assert";

const impit = new Impit({
    browser: "chrome",
    ignoreTlsErrors: true,
});

const response = await impit.fetch("https://api.apify.com/v2/browser-info");

assert.equal(response.status, 200, "Response status should be 200");
assert.ok(response.headers.get("content-type")?.includes("application/json"), "Response should be JSON");
assert.equal(await response.json().then(data => data.headers['accept-encoding']), "gzip, deflate, br, zstd", "Accept-Encoding header should be correct");

console.log(`[${import.meta.filename.split('/').pop()}] All assertions passed.`);
