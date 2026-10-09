use super::state::{RetainedEditors, TextInputKey, TextInputSlot};
use bevy::{ecs::system::SystemParam, input_focus::InputFocus, prelude::*};

/// 重建页面时保留真实编辑实体，避免打断光标、选区与 IME 组合输入。
#[derive(SystemParam)]
pub(crate) struct TextInputRetention<'w, 's> {
    retained: ResMut<'w, RetainedEditors>,
    editors: Query<'w, 's, (Entity, &'static TextInputKey)>,
}

impl TextInputRetention<'_, '_> {
    pub fn capture(&mut self, commands: &mut Commands) {
        self.retained.rebuilding = true;
        self.retained.editors.clear();
        for (entity, key) in &self.editors {
            self.retained.editors.insert(*key, entity);
            commands.entity(entity).remove::<ChildOf>();
        }
    }
}

pub(super) fn restore_editors(
    mut commands: Commands,
    mut retained: ResMut<RetainedEditors>,
    mut slots: Query<(Entity, &mut TextInputSlot)>,
    mut focus: ResMut<InputFocus>,
) {
    if !retained.rebuilding {
        return;
    }
    retained.rebuilding = false;
    for (entity, mut slot) in &mut slots {
        if let Some(editor) = retained.editors.remove(&slot.key) {
            commands.entity(slot.editor).despawn();
            commands.entity(entity).add_child(editor);
            slot.editor = editor;
        }
    }
    for (_, editor) in retained.editors.drain() {
        if focus.get() == Some(editor) {
            focus.clear();
        }
        commands.entity(editor).despawn();
    }
}
