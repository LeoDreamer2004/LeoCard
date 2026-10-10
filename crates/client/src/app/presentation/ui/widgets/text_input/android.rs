//! Native Android editing keeps IME composition, selection and clipboard in the system widget.

use super::state::{TextInputAction, TextInputEvent};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText};
use leocard_client::platform::{show_text_editor, take_text_results};

#[derive(Component)]
pub(super) struct AndroidEditor {
    pub title: String,
    pub numeric: bool,
    pub filter: fn(char) -> bool,
}

pub(super) fn edit_native_text(
    mut focus: ResMut<InputFocus>,
    mut editors: Query<(Entity, &mut EditableText, &AndroidEditor)>,
    mut events: MessageWriter<TextInputEvent>,
) {
    for result in take_text_results() {
        let Some((entity, mut editor, spec)) = editors
            .iter_mut()
            .find(|(entity, _, _)| entity.to_bits() == result.token)
        else {
            continue;
        };
        if result.accepted {
            let value: String = result
                .value
                .chars()
                .filter(|character| (spec.filter)(*character))
                .take(editor.max_characters.unwrap_or(usize::MAX))
                .collect();
            editor.editor_mut().set_text(&value);
            events.write(TextInputEvent {
                entity,
                action: TextInputAction::Submit,
            });
        }
    }
    let Some(entity) = focus.get() else {
        return;
    };
    let Ok((_, editor, spec)) = editors.get(entity) else {
        return;
    };
    let result = show_text_editor(
        entity.to_bits(),
        &spec.title,
        &editor.value().to_string(),
        spec.numeric,
    );
    // The native editor owns keyboard focus until it returns. Prevent Winit from opening a second IME.
    focus.clear();
    if let Err(error) = result {
        warn!("{error}");
    }
}
