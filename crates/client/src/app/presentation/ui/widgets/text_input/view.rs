use super::state::{InputPlaceholder, TextInputKey, TextInputSlot};
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::Pickable,
    prelude::*,
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    ui::VisualBox,
};

pub(crate) struct TextInput<'a> {
    pub key: &'static str,
    pub value: &'a str,
    pub placeholder: &'static str,
    pub max_characters: usize,
    pub filter: fn(char) -> bool,
    pub font_size: f32,
    pub tab_index: i32,
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
        }
    }

    pub fn spawn(
        self,
        commands: &mut Commands,
        parent: Entity,
        mut node: Node,
        assets: &UiAssets,
    ) -> Entity {
        let mut image = ImageNode::new(assets.home.input.clone()).with_mode(NodeImageMode::Sliced(
            TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.45,
            },
        ));
        image.visual_box = VisualBox::BorderBox;
        node.align_items = AlignItems::Center;
        let padding = node.padding;
        let slot = commands.spawn((node, image)).id();
        commands.entity(parent).add_child(slot);
        let editor = commands
            .spawn((
                TextInputKey(self.key),
                EditableText {
                    max_characters: Some(self.max_characters),
                    allow_newlines: false,
                    ..EditableText::new(self.value)
                },
                EditableTextFilter::new(self.filter),
                TextFont::from_font_size(self.font_size).with_font(assets.font.clone()),
                TextColor(TEXT),
                TextCursorStyle {
                    color: TEXT,
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
            ))
            .id();
        commands
            .entity(slot)
            .insert(TextInputSlot {
                key: TextInputKey(self.key),
                editor,
            })
            .add_child(editor);
        if !self.placeholder.is_empty() {
            // 原生编辑器必须保持为叶节点，才能使用文本测量计算高度。
            let placeholder = add_text(
                commands,
                slot,
                self.placeholder,
                self.font_size,
                MUTED,
                assets,
            );
            commands.entity(placeholder).insert((
                InputPlaceholder,
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
