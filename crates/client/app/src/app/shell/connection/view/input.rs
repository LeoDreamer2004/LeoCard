use super::super::form::{ConnectionField, InputField};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::{TextInput, UiComposer};

pub(super) struct ConnectionInput<'a> {
    value: &'a str,
    field: InputField,
    assets: &'a UiAssets,
    compact: bool,
}

impl<'a> ConnectionInput<'a> {
    pub(super) fn new(value: &'a str, field: InputField, assets: &'a UiAssets) -> Self {
        Self {
            value,
            field,
            assets,
            compact: false,
        }
    }

    pub(super) fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub(super) fn render(self, commands: &mut UiComposer, parent: Entity) {
        let mut input = TextInput::new(self.field.key(), self.value);
        input.max_characters = self.field.maximum_length();
        input.filter = self.field.filter();
        input.tab_index = self.field.tab_index();
        let editor = input.spawn(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(if self.compact { 34 } else { 45 }),
                padding: UiRect::axes(px(13), px(if self.compact { 4 } else { 9 })),
                ..default()
            },
            self.assets.text_input_style(),
        );
        commands.entity(editor).insert(ConnectionField(self.field));
    }
}
