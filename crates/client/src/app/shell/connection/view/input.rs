use super::super::form::{ConnectionField, InputField};
use crate::app::presentation::TextInput;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

pub(super) struct ConnectionInput<'a> {
    value: &'a str,
    field: InputField,
    assets: &'a UiAssets,
}

impl<'a> ConnectionInput<'a> {
    pub(super) fn new(value: &'a str, field: InputField, assets: &'a UiAssets) -> Self {
        Self {
            value,
            field,
            assets,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
        let mut input = TextInput::new(self.field.key(), self.value);
        input.max_characters = self.field.maximum_length();
        input.filter = self.field.filter();
        input.tab_index = self.field.tab_index();
        let editor = input.spawn(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(45),
                padding: UiRect::axes(px(13), px(9)),
                ..default()
            },
            self.assets,
        );
        commands.entity(editor).insert(ConnectionField(self.field));
    }
}
