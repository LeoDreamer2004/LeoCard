use super::super::MinecraftGuessAssets;
use super::style::{SEPARATOR_COLOR, SEPARATOR_WIDTH};
use crate::app::{
    presentation::{MUTED, TEXT, add_text, spawn_node},
    runtime::UiAssets,
};
use bevy::{picking::Pickable, prelude::*};
use leocard_minecraft_guess::MatchQuality;

pub(super) enum CellSkin {
    Heading,
    Feedback(MatchQuality),
    Unknown,
}

impl CellSkin {
    fn color(&self) -> Color {
        match self {
            Self::Heading => Color::WHITE,
            Self::Feedback(MatchQuality::Exact) => Color::srgb(0.57, 0.97, 0.74),
            Self::Feedback(MatchQuality::Partial) => Color::srgb(0.98, 0.84, 0.46),
            Self::Feedback(MatchQuality::Wrong) => Color::srgb(0.95, 0.55, 0.60),
            Self::Unknown => Color::srgb(0.65, 0.65, 0.65),
        }
    }
}

pub(super) struct TableCell {
    content: Entity,
}

impl TableCell {
    pub fn spawn(
        commands: &mut Commands,
        parent: Entity,
        width: f32,
        skin: CellSkin,
        assets: &MinecraftGuessAssets,
    ) -> Self {
        let content = spawn_node(
            commands,
            parent,
            Node {
                width: percent(width),
                min_width: px(0),
                flex_shrink: 0.0,
                padding: UiRect::all(px(8)),
                border: UiRect {
                    right: px(SEPARATOR_WIDTH),
                    bottom: px(SEPARATOR_WIDTH),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(4),
                ..default()
            },
            None,
        );
        let image = match skin {
            CellSkin::Heading => assets.surface(),
            CellSkin::Feedback(_) | CellSkin::Unknown => assets.cell(skin.color()),
        };
        commands.entity(content).insert((
            image,
            BorderColor::all(SEPARATOR_COLOR),
            Pickable::IGNORE,
        ));
        Self { content }
    }

    pub fn label(&self, commands: &mut Commands, text: &str, assets: &UiAssets) {
        self.text(commands, text, 18.0, TEXT, assets);
    }

    pub fn guess(&self, commands: &mut Commands, name: &str, player: &str, assets: &UiAssets) {
        self.text(commands, name, 20.0, TEXT, assets);
        self.text(commands, player, 14.0, MUTED, assets);
    }

    fn text(
        &self,
        commands: &mut Commands,
        text: &str,
        size: f32,
        color: Color,
        assets: &UiAssets,
    ) {
        let text = add_text(commands, self.content, text, size, color, assets);
        commands.entity(text).insert((
            Pickable::IGNORE,
            TextLayout::justify(Justify::Center),
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
        ));
    }
}
