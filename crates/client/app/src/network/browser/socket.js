const sockets = new Map();
let nextId = 1;

export function startConnection(url, join, ping, callback, settings) {
    const id = nextId++;
    const state = { options: JSON.parse(settings), id, url, join: new Uint8Array(join), ping: new Uint8Array(ping), callback, attempts: 0, everReady: false, ready: false, stopped: false };
    sockets.set(id, state);
    const resume = () => {
        if (document.hidden || state.stopped) return;
        if (state.socket?.readyState === WebSocket.OPEN) {
            if (Date.now() - state.received > state.options.heartbeat) state.socket.close();
            else if (state.ready) state.callback(4, null);
        } else if (state.socket?.readyState === WebSocket.CLOSED && !state.timer) connect(state);
    };
    document.addEventListener("visibilitychange", resume);
    state.detach = () => document.removeEventListener("visibilitychange", resume);
    try { connect(state); } catch (error) { state.detach(); sockets.delete(id); throw error; }
    return id;
}

function connect(state) {
    if (state.stopped) return;
    state.ready = false;
    const socket = new WebSocket(state.url);
    state.socket = socket;
    socket.binaryType = "arraybuffer";
    state.received = Date.now();
    state.connected = Date.now();
    state.opened = state.connected;
    const interval = setInterval(() => {
        if (document.hidden) return;
        if ((!state.ready && Date.now() - state.opened > state.options.heartbeat) || (socket.readyState !== WebSocket.OPEN && Date.now() - state.connected > state.options.connectTimeout) || Date.now() - state.received > state.options.heartbeat) { socket.close(); return; }
        if (socket.readyState === WebSocket.OPEN && state.ready) socket.send(state.ping);
    }, state.options.pingInterval);
    socket.onopen = () => { state.received = Date.now(); state.opened = Date.now(); socket.send(state.join); };
    socket.onmessage = event => {
        state.received = Date.now();
        if (!(event.data instanceof ArrayBuffer) || event.data.byteLength > state.options.maxFrame) {
            state.callback(2, "房主返回了无效的网络数据"); stopConnection(state.id); return;
        }
        const result = state.callback(0, new Uint8Array(event.data));
        if (result === 3) { socket.onmessage = null; socket.close(); return; }
        if (result === 2) { if (pendingRoom() === state.url) forgetRoom(); stopConnection(state.id); }
        if (result === 1) { sessionStorage.setItem("leocard.room", state.url); sessionStorage.setItem("leocard.token", state.options.token); state.ready = true; state.everReady = true; state.connected = Date.now(); }
    };
    socket.onerror = () => {};
    socket.onclose = () => {
        clearInterval(interval);
        if (state.stopped) return;
        if (!state.everReady) { state.callback(2, "无法连接房间，请检查 WSS 地址、隧道和证书"); stopConnection(state.id); return; }
        if (state.ready && Date.now() - state.connected >= 10000) state.attempts = 0;
        const delays = [0, 1000, 2000, 4000, 8000, 15000];
        if (state.attempts >= state.options.attempts) { state.callback(2, "已连续重连 6 次，仍无法连接房主"); stopConnection(state.id); return; }
        const delay = delays[state.attempts++];
        state.callback(1, `连接已断开，${delay / 1000} 秒后进行第 ${state.attempts}/6 次重连`);
        state.timer = setTimeout(() => { state.timer = undefined; connect(state); }, delay);
    };
    state.clearInterval = () => clearInterval(interval);
}

export function sendFrame(id, frame) {
    const state = sockets.get(id);
    if (!state?.ready || state.socket.readyState !== WebSocket.OPEN || state.socket.bufferedAmount > 2097152) return false;
    state.socket.send(new Uint8Array(frame));
    return true;
}

export function stopConnection(id) {
    const state = sockets.get(id);
    if (!state) return;
    state.stopped = true;
    clearTimeout(state.timer);
    state.clearInterval?.();
    state.detach?.();
    state.socket.onmessage = null;
    state.socket.close();
    state.callback = null;
    sockets.delete(id);
}

export function queryRoom(url, frame, callback) {
    const id = nextId++;
    const request = new Uint8Array(frame);
    const socket = new WebSocket(url);
    socket.binaryType = "arraybuffer";
    const state = { id, socket, callback };
    sockets.set(id, state);
    const finish = value => {
        if (!sockets.has(id)) return;
        callback(value);
        stopConnection(id);
    };
    socket.onopen = () => socket.send(request);
    socket.onmessage = event => finish(event.data instanceof ArrayBuffer ? new Uint8Array(event.data) : null);
    socket.onerror = () => finish(null);
    socket.onclose = () => finish(null);
    state.timer = setTimeout(() => finish(null), 3000);
    return id;
}

export function pendingRoom() { return sessionStorage.getItem("leocard.room"); }

export function rememberedToken(url) { return pendingRoom() === url ? sessionStorage.getItem("leocard.token") : null; }

export function forgetRoom() { sessionStorage.removeItem("leocard.room"); sessionStorage.removeItem("leocard.token"); }
