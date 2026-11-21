let installed = false;

export function installAudioUnlock() {
    const NativeAudioContext = globalThis.AudioContext;
    if (installed || !NativeAudioContext) return;
    installed = true;

    const contexts = new Set();
    let interacted = false;

    function resume(context) {
        if (context.state === "running" || context.state === "closed") return;
        // Call inside the input handler, before yielding user activation to WASM.
        void context.resume().catch(error => console.warn("无法恢复网页音频", error));
    }

    function resumeAll() {
        if (!interacted || document.hidden) return;
        for (const context of contexts) resume(context);
    }

    // Bevy owns CPAL's AudioContext privately. Track its real context before the
    // generated WASM bindings capture the constructor, keeping its normal mixer.
    globalThis.AudioContext = new Proxy(NativeAudioContext, {
        construct(target, argumentsList, newTarget) {
            const context = Reflect.construct(target, argumentsList, newTarget);
            contexts.add(context);
            context.addEventListener("statechange", () => {
                if (context.state === "closed") contexts.delete(context);
            });
            if (navigator.userActivation?.isActive) resume(context);
            return context;
        },
    });

    for (const type of ["pointerdown", "pointerup", "touchend", "keydown"]) {
        document.addEventListener(type, event => {
            if (!event.isTrusted) return;
            interacted = true;
            resumeAll();
        }, { capture: true, passive: true });
    }
    document.addEventListener("visibilitychange", resumeAll);
    window.addEventListener("pageshow", resumeAll);
}
