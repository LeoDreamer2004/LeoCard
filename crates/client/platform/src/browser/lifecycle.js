let pending = false;
let releasing = false;
let roomActive = false;
let hasPrevious = false;

export function initializeLifecycle() {
    hasPrevious = history.length > 1;
    history.pushState({ leocard: true }, "", location.href);
    window.addEventListener("popstate", () => {
        if (releasing) return;
        history.pushState({ leocard: true }, "", location.href);
        pending = true;
    });
    window.addEventListener("beforeunload", event => {
        if (!roomActive) return;
        event.preventDefault();
        event.returnValue = "";
    });
}

export function takeBackRequest() { const result = pending; pending = false; return result; }

export function allowBack() { releasing = true; history.go(hasPrevious ? -2 : -1); }

export function setRoomActive(active) { roomActive = active; }
