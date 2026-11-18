use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::text::{EditableText, TextEdit};

pub(super) fn scroll_multiline_inputs(
    mut wheel: MessageReader<MouseWheel>,
    mut editors: Query<(&mut EditableText, &PickingInteraction)>,
) {
    for event in wheel.read() {
        for (mut editor, interaction) in &mut editors {
            if editor.allow_newlines && *interaction != PickingInteraction::None {
                editor.queue_edit(match event.unit {
                    MouseScrollUnit::Line => TextEdit::ScrollByLines(-event.y * 3.0),
                    MouseScrollUnit::Pixel => TextEdit::ScrollBy(Vec2::new(0.0, -event.y)),
                });
            }
        }
    }
}
