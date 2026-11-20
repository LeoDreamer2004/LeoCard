import { readFile, writeFile, listFiles } from "./storage.js";

let dropEnabled = false;
let dropped = [];

async function importImage(file) {
    if (file.size > 33554432) throw new Error("图片不能超过 32 MiB");
    const extension = file.type === "image/png" ? "png" : "jpg";
    const name = Array.from(crypto.getRandomValues(new Uint32Array(4))).map(value => value.toString(16)).join("-");
    const path = `imports/${name}.${extension}`;
    writeFile(path, new Uint8Array(await file.arrayBuffer()));
    return path;
}

export function initializeDrop() {
    document.addEventListener("dragover", event => {
        if (event.dataTransfer?.types.includes("Files")) event.preventDefault();
    });
    document.addEventListener("drop", async event => {
        if (!event.dataTransfer?.files.length) return;
        event.preventDefault();
        if (!dropEnabled) return;
        const file = event.dataTransfer.files[0];
        if (!["image/png", "image/jpeg"].includes(file.type)) return;
        try { dropped.push(await importImage(file)); }
        catch (error) { window.dispatchEvent(new CustomEvent("leocard-storage-error", { detail: error.message })); }
    });
}

export function enableImageDrop(enabled) { dropEnabled = enabled; }

export function takeDroppedFiles() { const result = dropped; dropped = []; return result; }

export function chooseImage(title) {
    return new Promise((resolve, reject) => {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = "image/png,image/jpeg";
        input.setAttribute("aria-label", title);
        input.style.display = "none";
        document.body.append(input);
        input.oncancel = () => { input.remove(); resolve(null); };
        input.onchange = async () => {
            try {
                const file = input.files[0];
                if (!file) { resolve(null); return; }
                resolve(await importImage(file));
            } catch (error) { reject(error); }
            finally { input.remove(); }
        };
        input.click();
    });
}

export function openUrl(url) {
    const parsed = new URL(url);
    if (!["https:", "http:"].includes(parsed.protocol)) throw new Error("无效的网页地址");
    if (!window.open(parsed.href, "_blank", "noopener,noreferrer")) {
        // Browsers may return null for a successfully opened noopener window.
        console.info("已请求打开网页");
    }
}

export function openDirectory(path) {
    const page = window.open("about:blank", "leocard-files");
    if (!page) throw new Error("请允许打开文件列表窗口");
    const doc = page.document;
    doc.title = "LeoCard 本地文件";
    doc.body.replaceChildren();
    doc.body.style.cssText = "background:#181722;color:#e5def8;font:16px sans-serif;padding:32px";
    const heading = doc.createElement("h2");
    heading.textContent = "浏览器中的本地文件";
    doc.body.append(heading);
    const files = listFiles(path);
    for (const file of files) {
        const link = doc.createElement("a");
        link.textContent = file.slice(path.length + 1);
        link.download = file.split("/").at(-1);
        link.href = URL.createObjectURL(new Blob([readFile(file)], { type: "application/octet-stream" }));
        link.style.cssText = "display:block;color:#c7b8ee;margin:16px 0";
        doc.body.append(link);
        page.addEventListener("pagehide", () => URL.revokeObjectURL(link.href), { once: true });
    }
    if (!files.length) doc.body.append("这里暂时没有文件。");
}
