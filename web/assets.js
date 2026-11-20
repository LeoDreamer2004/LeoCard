import { ResourceCache } from "./cache.js";

const assets = new Map();

function decompress(response, contentType = "application/octet-stream") {
    return new Response(response.body.pipeThrough(new DecompressionStream("gzip")), {
        headers: { "Content-Type": contentType },
    });
}

function unpack(buffer, pack) {
    if (buffer.byteLength !== pack.unpacked_size || buffer.byteLength < 4) {
        throw new Error(`资源包不完整：${pack.path}`);
    }
    const headerSize = new DataView(buffer).getUint32(0, true);
    const start = 4 + headerSize;
    if (start > buffer.byteLength) throw new Error(`资源包目录损坏：${pack.path}`);
    const entries = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(buffer, 4, headerSize)));
    if (entries.length !== pack.asset_count) throw new Error(`资源包数量不符：${pack.path}`);
    const unpacked = new Map();
    let end = start;
    for (const [path, offset, size] of entries) {
        if (typeof path !== "string" || !path || path.startsWith("/") || path.split("/").includes("..")
            || !Number.isSafeInteger(offset) || !Number.isSafeInteger(size) || size < 0
            || start + offset !== end || end + size > buffer.byteLength || unpacked.has(path)) {
            throw new Error(`资源包目录损坏：${pack.path}`);
        }
        unpacked.set(path, new Uint8Array(buffer, start + offset, size));
        end += size;
    }
    if (end !== buffer.byteLength) throw new Error(`资源包长度不符：${pack.path}`);
    return unpacked;
}

// Called by Bevy's web asset source once; ownership then moves into WASM memory.
export function takeAssets() {
    const entries = Array.from(assets);
    assets.clear();
    return entries;
}

export async function loadResources(onProgress) {
    if (!globalThis.DecompressionStream) throw new Error("请使用支持资源解压的新版浏览器。");
    const base = new URL("./", import.meta.url);
    const response = await fetch(new URL("assets-manifest.json", base), { cache: "no-cache" });
    if (!response.ok) throw new Error("无法读取资源清单");
    const manifest = await response.json();
    const resources = [...manifest.packs, manifest.wasm];
    const total = resources.reduce((sum, resource) => sum + resource.size, 0);
    const cache = await ResourceCache.open(base);
    let completed = 0;
    let loaded = 0;
    let next = 0;
    assets.clear();
    const report = resource => {
        completed++;
        loaded += resource.size;
        onProgress({ completed, count: resources.length, loaded, total });
    };
    const worker = async () => {
        while (next < manifest.packs.length) {
            const pack = manifest.packs[next++];
            const entries = await cache.read(pack, async response => unpack(await decompress(response).arrayBuffer(), pack));
            for (const [path, bytes] of entries) {
                if (assets.has(path)) throw new Error(`资源包包含重复素材：${path}`);
                assets.set(path, bytes);
            }
            report(pack);
        }
    };
    // Compilation can finish while the resource workers are still downloading.
    const [module] = await Promise.all([
        cache.read(manifest.wasm, response => WebAssembly.compileStreaming(decompress(response, "application/wasm")))
            .then(module => { report(manifest.wasm); return module; }),
        ...Array.from({ length: Math.min(6, manifest.packs.length) }, worker),
    ]);
    return { module, finish: () => cache.prune(resources) };
}
