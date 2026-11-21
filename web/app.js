import { loadResources } from "./assets.js";
import { installAudioUnlock } from "./audio.js";

installAudioUnlock();

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

async function boot() {
    const [{ default: initialize }, resources] = await Promise.all([
        import("./pkg/leocard.js"),
        loadResources(({ completed, count, loaded, total }) => {
            progress.value = total ? loaded / total : 1;
            status.textContent = `正在准备资源 ${completed}/${count}　${(loaded / 1048576).toFixed(1)}/${(total / 1048576).toFixed(1)} MiB`;
        }),
    ]);
    status.textContent = "正在启动牌桌…";
    await initialize({ module_or_path: resources.module });
    void resources.finish();
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
