import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { gzipSync } from "node:zlib";
import { loadResources, takeAssets } from "../assets.js";

const base = new URL("../", import.meta.url);

class MemoryCache {
    entries = new Map();

    async match(url) { return this.entries.get(url)?.clone(); }
    async put(url, response) { this.entries.set(url, new Response(await response.arrayBuffer())); }
    async delete(key) { return this.entries.delete(key.url ?? key); }
    async keys() { return Array.from(this.entries.keys(), url => ({ url })); }
}

function fixture(t, cache = new MemoryCache()) {
    const files = new Map();
    const requests = [];
    const resource = (name, raw, extra = {}) => {
        const bytes = gzipSync(raw);
        const hash = createHash("sha256").update(bytes);
        const digest = hash.digest();
        const path = `${name}-${digest.toString("hex")}.pack`;
        files.set(path, bytes);
        return { path, size: bytes.length, unpacked_size: raw.length, integrity: `sha256-${digest.toString("base64")}`, ...extra };
    };
    const pack = (name, path, data) => {
        const header = Buffer.from(JSON.stringify([[path, 0, data.length]]));
        const length = Buffer.alloc(4);
        length.writeUInt32LE(header.length);
        return resource(`packs/${name}`, Buffer.concat([length, header, data]), { asset_count: 1 });
    };
    const manifest = {
        packs: [pack("common", "fonts/test.ttf", Buffer.from("font")), pack("mahjong", "cards/mahjong/一万.png", Buffer.from("tile"))],
        wasm: resource("pkg/wasm", Buffer.from([0, 97, 115, 109, 1, 0, 0, 0])),
    };
    const originalFetch = globalThis.fetch;
    const originalCaches = globalThis.caches;
    const fetchFile = async (input, options) => {
        const url = new URL(input);
        const path = decodeURI(url.href.slice(base.href.length));
        requests.push(path);
        if (path === "assets-manifest.json") return Response.json(manifest);
        assert.equal(options.cache, "force-cache");
        assert.ok(options.integrity.startsWith("sha256-"));
        return new Response(files.get(path), { status: files.has(path) ? 200 : 404 });
    };
    globalThis.fetch = fetchFile;
    globalThis.caches = { open: async () => cache };
    t.after(() => { globalThis.fetch = originalFetch; globalThis.caches = originalCaches; takeAssets(); });
    return { cache, files, requests, manifest, fetchFile, pack };
}

test("cold load mounts assets; warm load only requests the manifest and prunes obsolete packs", async t => {
    const { cache, requests } = fixture(t);
    cache.entries.set(new URL("packs/obsolete.pack", base).href, new Response("old"));
    const progress = [];
    const cold = await loadResources(value => progress.push(value));
    assert.ok(cold.module instanceof WebAssembly.Module);
    await cold.finish();
    assert.equal(requests.length, 4);
    assert.equal(progress.at(-1).completed, 3);
    assert.equal(cache.entries.size, 3);
    assert.deepEqual(new Map(takeAssets()), new Map([
        ["fonts/test.ttf", new Uint8Array(Buffer.from("font"))],
        ["cards/mahjong/一万.png", new Uint8Array(Buffer.from("tile"))],
    ]));
    assert.deepEqual(takeAssets(), []);
    requests.length = 0;
    const warm = await loadResources(() => {});
    await warm.finish();
    assert.deepEqual(requests, ["assets-manifest.json"]);
});

test("a changed pack downloads alone; corrupt cached data is replaced", async t => {
    const { cache, manifest, requests, pack: makePack } = fixture(t);
    await (await loadResources(() => {})).finish();
    const previous = new URL(manifest.packs[1].path, base).href;
    const pack = makePack("mahjong", "cards/mahjong/一万.png", Buffer.from("changed tile"));
    manifest.packs[1] = pack;
    const original = new URL(pack.path, base).href;
    requests.length = 0;
    await (await loadResources(() => {})).finish();
    assert.deepEqual(requests, ["assets-manifest.json", pack.path]);
    assert.equal(cache.entries.has(previous), false);
    const saved = cache.entries.get(original);
    cache.entries.set(original, new Response("broken gzip"));
    requests.length = 0;
    await (await loadResources(() => {})).finish();
    assert.deepEqual(requests, ["assets-manifest.json", pack.path]);
    assert.deepEqual(await cache.entries.get(original).clone().bytes(), await saved.clone().bytes());
});

test("cache quota failures do not prevent startup", async t => {
    const cache = new MemoryCache();
    cache.put = async () => { throw new Error("QuotaExceededError"); };
    fixture(t, cache);
    const loaded = await loadResources(() => {});
    await loaded.finish();
    assert.ok(loaded.module instanceof WebAssembly.Module);
    assert.equal(takeAssets().length, 2);
});

test("WASM compilation proceeds while a resource pack is still downloading", { timeout: 3000 }, async t => {
    const { fetchFile, manifest } = fixture(t);
    const blocked = Promise.withResolvers();
    const compiled = Promise.withResolvers();
    const compile = WebAssembly.compileStreaming;
    t.mock.method(WebAssembly, "compileStreaming", async response => {
        const module = await compile(response);
        compiled.resolve();
        return module;
    });
    globalThis.fetch = async (url, options) => {
        if (String(url).endsWith(manifest.packs[0].path)) await blocked.promise;
        return fetchFile(url, options);
    };
    let finished = false;
    const loading = loadResources(() => {}).then(value => { finished = true; return value; });
    await compiled.promise;
    assert.equal(finished, false);
    blocked.resolve();
    await (await loading).finish();
});
