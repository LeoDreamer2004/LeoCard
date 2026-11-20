const status = document.querySelector("#status");
const progress = document.querySelector("#progress");
const loading = document.querySelector("#loading");
const retry = document.querySelector("#retry");
const fullscreen = document.querySelector("#fullscreen");

function fail(message) {
    status.textContent = message;
    loading.hidden = false;
    retry.hidden = false;
}

retry.onclick = () => location.reload();
window.addEventListener("leocard-storage-error", event => fail(event.detail));
window.addEventListener("error", event => {
    console.error(event.error ?? event.message);
    if (!loading.hidden) fail("游戏启动失败，请查看浏览器控制台并重新加载。");
});

async function preload() {
    const response = await fetch(new URL("./assets-manifest.json", import.meta.url));
    if (!response.ok) throw new Error("无法读取素材清单");
    const assets = await response.json();
    const total = assets.reduce((sum, asset) => sum + asset.size, 0);
    let downloaded = 0;
    let completed = 0;
    let next = 0;
    const worker = async () => {
        while (next < assets.length) {
            const asset = assets[next++];
            const response = await fetch(new URL(asset.path, import.meta.url), { cache: "no-cache" });
            if (!response.ok) throw new Error(`素材下载失败：${asset.path}`);
            const bytes = await response.arrayBuffer();
            if (bytes.byteLength !== asset.size) throw new Error(`素材下载不完整：${asset.path}`);
            downloaded += asset.size;
            completed++;
            progress.value = total ? downloaded / total : 1;
            status.textContent = `正在下载素材 ${completed}/${assets.length}　${(downloaded / 1048576).toFixed(1)}/${(total / 1048576).toFixed(1)} MiB`;
        }
    };
    await Promise.all(Array.from({ length: 8 }, worker));
}

async function boot() {
    await preload();
    status.textContent = "正在启动牌桌…";
    const { default: initialize } = await import("./pkg/leocard.js");
    await initialize();
    // The Rust entry point waits for persistent storage before launching Bevy.
    const observer = new MutationObserver(() => {
        if (document.querySelector("#leocard")?.dataset.started) { loading.hidden = true; observer.disconnect(); }
    });
    observer.observe(document.querySelector("#leocard"), { attributes: true });
    if (document.querySelector("#leocard")?.dataset.started) { loading.hidden = true; observer.disconnect(); }
    fullscreen.hidden = !document.fullscreenEnabled;
}

fullscreen.onclick = async () => {
    try {
        if (document.fullscreenElement) await document.exitFullscreen();
        else {
            await document.documentElement.requestFullscreen();
            try { await screen.orientation.lock("landscape"); } catch { /* Orientation locking is optional. */ }
        }
    } catch (error) { console.info(error); }
};

boot().catch(error => { console.error(error); fail(error.message ?? "游戏启动失败"); });
