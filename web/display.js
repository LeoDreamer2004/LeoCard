export function fullscreenSupported() {
    return Boolean(document.fullscreenEnabled && document.documentElement.requestFullscreen);
}

export function isFullscreen() {
    return Boolean(document.fullscreenElement);
}

export async function toggleFullscreen() {
    if (!fullscreenSupported()) return;
    try {
        if (isFullscreen()) await document.exitFullscreen();
        else {
            await document.documentElement.requestFullscreen();
            try { await screen.orientation.lock("landscape"); } catch { /* Orientation locking is optional. */ }
        }
    } catch (error) {
        console.info("浏览器未能切换全屏。", error);
    }
}

export function installDisplayControls() {
    document.querySelector("#fullscreen").onclick = () => { void toggleFullscreen(); };
    // Keep the request inside the trusted key event and suppress the browser's own F11 toggle.
    window.addEventListener("keydown", event => {
        if (event.code !== "F11" || event.altKey || event.ctrlKey || event.metaKey || !fullscreenSupported()) return;
        event.preventDefault();
        if (!event.repeat) void toggleFullscreen();
    }, { capture: true });
}
