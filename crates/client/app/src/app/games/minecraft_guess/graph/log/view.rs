use super::super::super::{MinecraftGuessAssets, MinecraftGuessUiAction};
use super::message::render_message;
use super::state::{GraphLog, GraphLogRoot, GraphLogScroll};
use crate::app::presentation::SilentButton;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_protocol::MatchId;
use leocard_ui::{CustomButtonMotion, ScrollContentHeight, Scrollbar, UiComposer, spawn_node};

pub(super) fn render_log(
    commands: &mut UiComposer,
    key: (MatchId, u8),
    log: &GraphLog,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) {
    let layer = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(1400),
        ))
        .id();
    let root = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(0),
            width: px(360),
            max_width: percent(40),
            height: px(log.height),
            max_height: percent(60),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(6)),
            row_gap: px(3),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.65)),
    );
    commands.entity(root).insert((
        GraphLogRoot {
            key,
            version: log.version,
            layer,
        },
        RelativeCursorPosition::default(),
        Pickable::default(),
    ));
    if !log.expanded {
        commands.entity(root).insert((
            Button,
            SilentButton,
            CustomButtonMotion,
            UiAction::MinecraftGuess(MinecraftGuessUiAction::ToggleGraphLog),
        ));
    }
    let body = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            height: px(0),
            min_height: px(0),

            flex_grow: 1.0,
            column_gap: px(4),
            overflow: Overflow::clip(),
            ..default()
        },
        None,
    );
    let list = spawn_node(
        commands,
        body,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            justify_content: if log.expanded {
                JustifyContent::FlexStart
            } else {
                JustifyContent::FlexEnd
            },
            overflow: if log.expanded {
                Overflow::scroll_y()
            } else {
                Overflow::clip()
            },
            ..default()
        },
        None,
    );
    if log.expanded {
        commands.entity(list).insert((
            GraphLogScroll,
            ScrollContentHeight::default(),
            RelativeCursorPosition::default(),
            ScrollPosition(Vec2::new(0.0, log.scroll_y)),
        ));
        Scrollbar::spawn(commands, body, list, game_assets.scrollbar());
    }
    let content = spawn_node(
        commands,
        list,
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            // Empty space stays above short histories; overflowing histories scroll normally.
            margin: UiRect::top(Val::Auto),
            ..default()
        },
        None,
    );
    for message in log.messages.iter().skip(if log.expanded {
        0
    } else {
        log.messages.len().saturating_sub(4)
    }) {
        render_message(commands, content, message, assets);
    }
}
