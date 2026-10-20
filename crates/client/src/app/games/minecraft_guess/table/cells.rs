use super::super::MinecraftGuessAssets;
use super::super::assets::EntryIcons;
use super::super::feedback::GuessCellReveal;
use super::details::AttributeHint;
use super::names::GuessName;
use super::style::{SEPARATOR_COLOR, SEPARATOR_WIDTH};
use crate::app::presentation::{MUTED, TEXT, TransitionOpacity, UiComposer, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_minecraft_guess::{Attribute, MatchQuality};

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
    container: Entity,
    content: Entity,
    color: Color,
    compact: bool,
}

impl TableCell {
    pub fn spawn(
        commands: &mut UiComposer,
        parent: Entity,
        width: f32,
        skin: CellSkin,
        assets: &MinecraftGuessAssets,
        compact: bool,
    ) -> Self {
        let container = spawn_node(
            commands,
            parent,
            Node {
                width: percent(width),
                min_width: px(0),
                flex_shrink: 0.0,
                padding: UiRect::all(px(if compact { 4.0 } else { 8.0 })),
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
        commands.entity(container).insert((
            image,
            BorderColor::all(SEPARATOR_COLOR),
            Pickable::IGNORE,
        ));
        let content = spawn_node(
            commands,
            container,
            Node {
                width: percent(100),
                min_width: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(4),
                ..default()
            },
            None,
        );
        Self {
            container,
            content,
            color: skin.color(),
            compact,
        }
    }

    pub fn revealing(self, commands: &mut UiComposer, row: Option<usize>, column: usize) -> Self {
        let Some(row) = row else {
            return self;
        };
        commands.entity(self.container).insert((
            GuessCellReveal::new(row, column, self.color, self.content),
            TransitionOpacity::default(),
        ));
        self
    }

    pub fn label(&self, commands: &mut UiComposer, text: &str, assets: &UiAssets) {
        self.text(
            commands,
            text,
            if self.compact { 13.0 } else { 18.0 },
            TEXT,
            assets,
        );
    }

    pub fn details(
        &self,
        commands: &mut UiComposer,
        layer: Entity,
        attribute: &Attribute,
        assets: &UiAssets,
        game_assets: &MinecraftGuessAssets,
    ) {
        AttributeHint::spawn(
            commands,
            layer,
            self.content,
            attribute,
            assets,
            game_assets,
        );
    }

    pub fn guess(
        &self,
        commands: &mut UiComposer,
        name: &str,
        player: &str,
        icon: Option<ImageNode>,
        assets: &UiAssets,
    ) {
        let group = spawn_node(
            commands,
            self.content,
            Node {
                width: percent(100),
                min_width: px(0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(4),
                ..default()
            },
            None,
        );
        commands.entity(group).insert(Pickable::IGNORE);
        if let Some(icon) = icon {
            EntryIcons::spawn(
                commands,
                group,
                icon,
                if self.compact { 24.0 } else { 32.0 },
            );
        }
        let label = add_text(
            commands,
            group,
            name,
            if self.compact { 14.0 } else { 18.0 },
            TEXT,
            assets,
        );
        commands.entity(label).insert((
            GuessName,
            Pickable::IGNORE,
            TextLayout::justify(Justify::Center),
            Node {
                min_width: px(0),
                ..default()
            },
        ));
        self.text(
            commands,
            player,
            if self.compact { 11.0 } else { 14.0 },
            MUTED,
            assets,
        );
    }

    fn text(
        &self,
        commands: &mut UiComposer,
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
