// Only generated, content-addressed resources belong in this cache.
export class ResourceCache {
    constructor(base, cache) {
        this.base = base;
        this.cache = cache;
        this.writes = [];
    }

    static async open(base) {
        let cache;
        try {
            cache = await globalThis.caches?.open(`leocard-resources-v1:${base.href}`);
        } catch (error) {
            console.info("浏览器缓存不可用，本次直接加载资源。", error);
        }
        return new ResourceCache(base, cache);
    }

    async read(resource, consume) {
        const url = new URL(resource.path, this.base).href;
        if (this.cache) {
            try {
                const cached = await this.cache.match(url);
                if (cached) {
                    try {
                        return await consume(cached);
                    } catch (error) {
                        await this.cache.delete(url);
                        console.info("重新下载损坏的资源缓存。", resource.path, error);
                    }
                }
            } catch (error) {
                console.info("无法读取浏览器资源缓存。", error);
            }
        }
        const response = await fetch(url, { cache: "force-cache", integrity: resource.integrity });
        if (!response.ok) throw new Error(`资源下载失败：${resource.path}`);
        const copy = this.cache ? response.clone() : null;
        const result = await consume(response);
        if (copy) {
            this.writes.push(this.cache.put(url, copy).catch(error => {
                console.info("无法保存资源缓存，本次游戏仍可继续。", error);
            }));
        }
        return result;
    }

    async prune(resources) {
        if (!this.cache) return;
        try {
            await Promise.all(this.writes);
            const keep = new Set(resources.map(resource => new URL(resource.path, this.base).href));
            const keys = await this.cache.keys();
            await Promise.all(keys.filter(key => !keep.has(key.url)).map(key => this.cache.delete(key)));
        } catch (error) {
            console.info("无法清理过期资源缓存。", error);
        }
    }
}
