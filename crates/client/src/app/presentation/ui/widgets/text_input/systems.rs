use super::state::{InputPlaceholder, TextInputAction, TextInputEvent, TextInputSlot};
use crate::app::runtime::UiAssets;
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::{AcquireFocus, FocusCause, FocusedInput, InputFocus},
    prelude::*,
    text::EditableText,
};

/// 将输入框留白区域的原生聚焦请求转发到实际编辑实体。
pub(super) fn focus_input_slot(
    mut acquire: On<AcquireFocus>,
    slots: Query<&TextInputSlot>,
    mut focus: ResMut<InputFocus>,
) {
    if let Ok(slot) = slots.get(acquire.focused_entity) {
        focus.set(slot.editor, FocusCause::Pressed);
        acquire.propagate(false);
    }
}

pub(super) fn collect_submissions(
    input: On<FocusedInput<KeyboardInput>>,
    editors: Query<&EditableText>,
    mut events: MessageWriter<TextInputEvent>,
) {
    let entity = input.original_event_target();
    if input.focused_entity != entity
        || input.input.state != ButtonState::Pressed
        || input.input.repeat
    {
        return;
    }
    let Ok(editor) = editors.get(entity) else {
        return;
    };
    if editor.is_composing() {
        return;
    }
    // Bevy 0.20 clears focus during Escape dispatch, before Update runs.
    // Use the original input target so cancellation is still delivered once.
    let action = match input.input.key_code {
        KeyCode::Enter => TextInputAction::Submit,
        KeyCode::Escape => TextInputAction::Cancel,
        _ => return,
    };
    events.write(TextInputEvent { entity, action });
}

pub(super) fn sync_input_skin(
    focus: Res<InputFocus>,
    assets: Res<UiAssets>,
    mut slots: Query<(&TextInputSlot, &mut ImageNode)>,
    editors: Query<&EditableText>,
    mut placeholders: Query<(&ChildOf, &mut Visibility), With<InputPlaceholder>>,
) {
    for (slot, mut image) in &mut slots {
        let texture = if focus.get() == Some(slot.editor) {
            &assets.home.focused_input
        } else {
            &assets.home.input
        };
        if image.image != *texture {
            image.image = texture.clone();
        }
    }
    for (parent, mut visibility) in &mut placeholders {
        let Ok((slot, _)) = slots.get(parent.parent()) else {
            continue;
        };
        *visibility = if editors
            .get(slot.editor)
            .is_ok_and(|editor| editor.value() == "" && !editor.is_composing())
        {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
