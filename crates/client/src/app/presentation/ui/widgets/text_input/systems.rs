use super::state::{InputPlaceholder, TextInputAction, TextInputEvent, TextInputSlot};
use crate::app::runtime::UiAssets;
use bevy::{
    input_focus::{AcquireFocus, FocusCause, InputFocus},
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
    keyboard: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    editors: Query<&EditableText>,
    mut events: MessageWriter<TextInputEvent>,
) {
    let Some(entity) = focus.get() else {
        return;
    };
    let Ok(editor) = editors.get(entity) else {
        return;
    };
    if editor.is_composing() {
        return;
    }
    let action = if keyboard.just_pressed(KeyCode::Enter) {
        TextInputAction::Submit
    } else if keyboard.just_pressed(KeyCode::Escape) {
        TextInputAction::Cancel
    } else {
        return;
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
