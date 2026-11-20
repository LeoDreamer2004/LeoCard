const controls = new Map();
let edits = [];

function queue(control, action) {
    edits.push({ entity: control.dataset.entity, value: control.value, action });
}

function create(spec) {
    const input = document.createElement(spec.multiline ? "textarea" : "input");
    input.className = "leocard-editor";
    input.dataset.entity = spec.entity;
    input.autocomplete = "off";
    input.spellcheck = false;
    if (!spec.multiline) input.type = spec.password ? "password" : "text";
    input.inputMode = spec.numeric ? "numeric" : "text";
    input.setAttribute("aria-label", spec.title);
    input.maxLength = spec.maxCharacters * 2;
    input.readOnly = spec.readonly;
    input.value = spec.value ?? "";
    input.onfocus = () => queue(input, 4);
    input.onblur = () => queue(input, 3);
    input.oncompositionstart = () => { input.composing = true; };
    input.oncompositionend = () => { input.composing = false; queue(input, 0); };
    input.oninput = event => { if (!event.isComposing && !input.composing) queue(input, 0); };
    input.onkeydown = event => {
        event.stopPropagation();
        if (event.isComposing || input.composing || event.keyCode === 229) return;
        if (event.key === "Escape" || (event.key === "Enter" && !spec.multiline)) {
            event.preventDefault();
            queue(input, event.key === "Escape" ? 2 : 1);
            input.blur();
        }
    };
    for (const name of ["keyup", "pointerdown", "pointerup", "click", "wheel"]) input.addEventListener(name, event => event.stopPropagation());
    document.body.append(input);
    controls.set(spec.entity, input);
    return input;
}

export function publishEditors(json) {
    const specifications = JSON.parse(json);
    const alive = new Set();
    for (const spec of specifications) {
        alive.add(spec.entity);
        const input = controls.get(spec.entity) ?? create(spec);
        if (spec.value !== null && !input.composing && input.value !== spec.value) {
            const start = input.selectionStart, end = input.selectionEnd;
            input.value = spec.value;
            input.setSelectionRange(Math.min(start, input.value.length), Math.min(end, input.value.length));
        }
        let [left, top, width, height] = spec.rect;
        const canvas = document.querySelector("#leocard").getBoundingClientRect();
        left += canvas.left; top += canvas.top;
        input.readOnly = spec.readonly;
        const viewport = window.visualViewport;
        const visibleBottom = (viewport?.height ?? innerHeight) + (viewport?.offsetTop ?? 0);
        if (document.activeElement === input) height = Math.min(height, Math.max(24, (viewport?.height ?? innerHeight) - 24));
        const adjustedTop = document.activeElement === input ? Math.min(top, Math.max(0, visibleBottom - height - 12)) : top;
        Object.assign(input.style, { left: `${left}px`, top: `${adjustedTop}px`, width: `${width}px`, height: `${height}px`, fontSize: `${Math.max(16, spec.fontSize)}px`, zIndex: `${100 + spec.order}` });
        if (spec.focused && document.activeElement !== input) input.focus({ preventScroll: true });
    }
    for (const [entity, input] of controls) {
        if (!alive.has(entity)) { input.onblur = null; input.remove(); controls.delete(entity); }
    }
}

export function takeEdits() {
    const result = JSON.stringify(edits);
    edits = [];
    return result;
}
