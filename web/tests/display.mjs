import assert from "node:assert/strict";
import { test } from "node:test";
import { fullscreenSupported, installDisplayControls, isFullscreen, toggleFullscreen } from "../display.js";

function browser(t) {
    const original = { document: globalThis.document, window: globalThis.window, screen: globalThis.screen };
    const button = {};
    const calls = { enter: 0, exit: 0 };
    globalThis.window = new EventTarget();
    globalThis.screen = { orientation: { lock: async () => {} } };
    globalThis.document = {
        fullscreenEnabled: true,
        fullscreenElement: null,
        querySelector: () => button,
        documentElement: { requestFullscreen: async () => { calls.enter++; document.fullscreenElement = document.documentElement; } },
        exitFullscreen: async () => { calls.exit++; document.fullscreenElement = null; },
    };
    t.after(() => Object.assign(globalThis, original));
    const key = (code, repeat = false) => {
        const event = new Event("keydown", { cancelable: true });
        Object.assign(event, { code, repeat });
        window.dispatchEvent(event);
        return event;
    };
    return { button, calls, key };
}

test("F11 and the web button share fullscreen state without repeated-key toggles", async t => {
    const { button, calls, key } = browser(t);
    installDisplayControls();
    assert.ok(fullscreenSupported());
    assert.equal(key("KeyF").defaultPrevented, false);
    assert.equal(key("F11").defaultPrevented, true);
    assert.ok(isFullscreen());
    assert.equal(key("F11", true).defaultPrevented, true);
    assert.deepEqual(calls, { enter: 1, exit: 0 });
    button.onclick();
    assert.equal(isFullscreen(), false);
    await toggleFullscreen();
    assert.ok(isFullscreen());
    // Escape or the browser UI can leave fullscreen without using our controls.
    document.fullscreenElement = null;
    assert.equal(isFullscreen(), false);
});

test("unsupported or rejected fullscreen requests leave the actual state unchanged", async t => {
    const { calls, key } = browser(t);
    installDisplayControls();
    document.fullscreenEnabled = false;
    assert.equal(fullscreenSupported(), false);
    assert.equal(key("F11").defaultPrevented, false);
    await toggleFullscreen();
    assert.deepEqual(calls, { enter: 0, exit: 0 });
    document.fullscreenEnabled = true;
    document.documentElement.requestFullscreen = async () => { throw new Error("NotAllowedError"); };
    t.mock.method(console, "info", () => {});
    await toggleFullscreen();
    assert.equal(isFullscreen(), false);
});
