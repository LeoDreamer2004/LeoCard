const files = new Map();
const mobileAgent = /Android|iPhone|iPad|iPod/.test(navigator.userAgent);
const coarsePointer = matchMedia("(pointer: coarse)");
let database;
let storageFailure;

function transaction(operations) {
    if (!database) throw new Error("浏览器存储尚未就绪");
    if (storageFailure) throw storageFailure;
    const tx = database.transaction("files", "readwrite");
    const store = tx.objectStore("files");
    operations(store);
    tx.onabort = tx.onerror = () => {
        storageFailure = new Error(`浏览器保存失败：${tx.error?.message ?? "存储空间不足"}`);
        window.dispatchEvent(new CustomEvent("leocard-storage-error", { detail: storageFailure.message }));
    };
}

export async function initializeStorage() {
    database = await new Promise((resolve, reject) => {
        const request = indexedDB.open("leocard", 1);
        request.onupgradeneeded = () => request.result.createObjectStore("files");
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
        request.onblocked = () => reject(new Error("请关闭其他旧版本 LeoCard 页面后重试"));
    });
    database.onversionchange = () => database.close();
    await new Promise((resolve, reject) => {
        const tx = database.transaction("files", "readonly");
        const request = tx.objectStore("files").openCursor();
        request.onsuccess = () => {
            const cursor = request.result;
            if (!cursor) return;
            files.set(cursor.key, new Uint8Array(cursor.value));
            cursor.continue();
        };
        tx.oncomplete = resolve;
        tx.onerror = () => reject(tx.error);
    });
}

export function readFile(path) {
    return files.get(path);
}

export function writeFile(path, bytes) {
    const copy = new Uint8Array(bytes);
    transaction(store => store.put(copy, path));
    files.set(path, copy);
}

export function renameFile(from, to) {
    const bytes = files.get(from);
    if (!bytes) throw new Error("文件不存在");
    transaction(store => { store.put(bytes, to); store.delete(from); });
    files.set(to, bytes);
    files.delete(from);
}

export function removeFile(path) {
    if (!files.has(path)) return false;
    transaction(store => store.delete(path));
    return files.delete(path);
}

export function listFiles(directory) {
    const prefix = directory.replace(/\/$/, "") + "/";
    return [...files.keys()].filter(key => key.startsWith(prefix) && !key.slice(prefix.length).includes("/")).sort();
}

export function fileExists(path) {
    return files.has(path);
}

export function isMobile() {
    return mobileAgent || (navigator.maxTouchPoints > 1 && coarsePointer.matches);
}
