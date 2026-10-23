use super::{
    TextInputSkin, TextInputStyle,
    state::{InputPlaceholder, TextInputKey, TextInputSlot},
};
use crate::{UiComposer, UiPressTarget, ui_component};
use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::Pickable,
    prelude::*,
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    ui_widgets::TextInput as NativeTextInput,
};

pub struct TextInput<'a> {
    pub key: &'static str,
    pub value: &'a str,
    pub placeholder: &'static str,
    pub max_characters: usize,
    pub filter: fn(char) -> bool,
    pub font_size: f32,
    pub tab_index: i32,
    pub skin: Option<TextInputSkin>,
    pub password: bool,
}

impl<'a> TextInput<'a> {
    pub fn new(key: &'static str, value: &'a str) -> Self {
        Self {
            key,
            value,
            placeholder: "",
            max_characters: usize::MAX,
            filter: |character| !character.is_control(),
            font_size: 15.0,
            tab_index: 0,
            skin: None,
            password: false,
        }
    }

    pub fn spawn(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        mut node: Node,
        style: TextInputStyle,
    ) -> Entity {
        let skin = self.skin.unwrap_or(style.skin);
        let image = skin.image(false).clone();
        node.align_items = AlignItems::Center;
        let padding = node.padding;
        let slot = commands.mount(
            parent,
            format!("input:{}", self.key),
            bsn! {
                node
                @ui_component(image)
                @ui_component(skin)
                UiPressTarget
            },
        );
        let editor = commands.entity_for(slot, "editor");
        commands
            .entity(editor)
            .entry::<EditableText>()
            .or_insert(EditableText {
                max_characters: Some(self.max_characters),
                allow_newlines: false,
                ..EditableText::new(self.value)
            });
        commands.entity(editor).insert((
            TextInputKey(self.key),
            NativeTextInput,
            EditableTextFilter::new(self.filter),
            TextFont::from_font_size(self.font_size).with_font(style.font.clone()),
            TextColor(if self.password {
                Color::NONE
            } else {
                style.text
            }),
            TextCursorStyle {
                color: style.text,
                selection_color: Color::srgb(0.20, 0.42, 0.72),
                unfocused_selection_color: Color::NONE,
                ..default()
            },
            TextLayout::no_wrap(),
            TabIndex(self.tab_index),
            Node {
                width: percent(100),
                min_width: px(0),
                overflow: Overflow::clip(),
                ..default()
            },
        ));
        if self.password {
            commands.child(
                slot,
                (
                    ZIndex(1),
                    super::password::PasswordMask {
                        editor,
                        font_size: self.font_size,
                        color: style.text,
                    },
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: padding.left,
                        right: padding.right,
                        height: px(self.font_size * 1.5),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                ),
            );
        }
        #[cfg(target_os = "android")]
        commands.entity(editor).insert(super::state::AndroidEditor {
            title: if self.placeholder.is_empty() {
                "编辑文本"
            } else {
                self.placeholder
            }
            .to_owned(),
            numeric: !(self.filter)('a') && !(self.filter)('.'),
            filter: self.filter,
        });
        commands
            .entity(slot)
            .insert(TextInputSlot { editor })
            .add_child(editor);
        if !self.placeholder.is_empty() {
            // 原生编辑器必须保持为叶节点，才能使用文本测量计算高度。
            let placeholder = commands.child(
                slot,
                (
                    Text::new(self.placeholder),
                    TextFont::from_font_size(self.font_size).with_font(style.font),
                    TextColor(style.placeholder),
                ),
            );
            commands.entity(placeholder).insert((
                InputPlaceholder,
                if self.value.is_empty() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Pickable::IGNORE,
                TextLayout::no_wrap(),
                Node {
                    position_type: PositionType::Absolute,
                    left: padding.left,
                    right: padding.right,
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));
        }
        editor
    }
}
