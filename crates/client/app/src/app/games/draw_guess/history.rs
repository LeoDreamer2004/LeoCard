use super::feedback::DrawingHistoryMessage;
use super::state::{GuessHistory, HistoryScroll};
use crate::app::presentation::{READY, TEXT, add_text, game_side_control_scale};
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_draw_guess::GuessMessage;
use leocard_protocol::{DrawGuessSnapshot, GameSnapshot};
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    paper: Entity,
    game: &DrawGuessSnapshot,
    footer_height: f32,
    assets: &UiAssets,
) {
    if game.state.history.is_empty() {
        return;
    }
    let panel = spawn_node(
        commands,
        paper,
        Node {
            position_type: PositionType::Absolute,
            right: px(10),
            bottom: px(10.0 + footer_height),
            width: px(240.0 * game_side_control_scale()),
            max_width: percent(42),
            max_height: percent(60),
            row_gap: px(6.0 * game_side_control_scale()),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    commands.entity(panel).insert((GuessHistory, ZIndex(4)));
    let list_row = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(156.0 * game_side_control_scale()),
            max_height: percent(100),
            min_height: px(0),
            ..default()
        },
        None,
    );
    let list = commands.keyed_child(
        list_row,
        "history-list",
        (
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0 * game_side_control_scale()),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
        ),
    );
    commands.entity(list).init(HistoryScroll::default());
    for message in &game.state.history {
        render_message(commands, list, game, message, assets);
    }
}

fn render_message(
    commands: &mut UiComposer,
    list: Entity,
    game: &DrawGuessSnapshot,
    message: &GuessMessage,
    assets: &UiAssets,
) {
    let scale = game_side_control_scale();
    let row = commands.keyed_child(
        list,
        format!("message:{}", message.serial),
        (
            Node {
                width: percent(100),
                min_width: px(0),
                flex_shrink: 0.0,
                padding: UiRect::axes(px(8.0 * scale), px(6.0 * scale)),
                border_radius: BorderRadius::all(px(8.0 * scale)),
                ..default()
            },
            BackgroundColor(if message.correct {
                Color::srgba(0.08, 0.30, 0.16, 0.82)
            } else {
                Color::srgba(0.10, 0.12, 0.15, 0.76)
            }),
            Pickable::IGNORE,
            UiTransform::IDENTITY,
            DrawingHistoryMessage {
                serial: message.serial,
            },
        ),
    );
    let name = game
        .players
        .iter()
        .find(|player| player.player.id == message.player)
        .map_or("?", |player| player.player.name.as_str());
    let color = if message.correct { READY } else { TEXT };
    let font = TextFont {
        weight: FontWeight::BOLD,
        style: if message.private {
            FontStyle::Oblique(Some(14.0))
        } else {
            FontStyle::Normal
        },
        ..TextFont::from_font_size(18.0 * scale).with_font(assets.font.clone())
    };
    let text = add_text(
        commands,
        row,
        format!("{name}："),
        18.0 * scale,
        color,
        assets,
    );
    commands.entity(text).insert((
        font.clone(),
        Pickable::IGNORE,
        Node {
            width: percent(100),
            min_width: px(0),
            ..default()
        },
    ));
    commands.child(
        text,
        (
            TextSpan::new(if message.correct {
                format!("猜中了 +{}", message.points)
            } else {
                message.text.clone()
            }),
            TextFont {
                weight: if message.correct {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                },
                ..font
            },
            TextColor(color),
        ),
    );
}

pub(super) fn scroll_latest(
    client: Option<Res<ClientResource>>,
    mut lists: Query<(&ComputedNode, &mut ScrollPosition, &mut HistoryScroll)>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::draw_guess)
    else {
        return;
    };
    let serial = game
        .state
        .history
        .last()
        .map_or(0, |message| message.serial);
    for (node, mut position, mut state) in &mut lists {
        if state.serial != serial {
            position.y =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            state.serial = serial;
        }
    }
}
