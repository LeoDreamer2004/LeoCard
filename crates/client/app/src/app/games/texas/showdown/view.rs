use super::state::*;
use super::timeline::TexasShowdownSequence;
use crate::app::games::texas::{TexasTableLayout, texas_category_label};
use crate::app::presentation::{GameSummaryAnimation, add_title};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::TexasSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(in crate::app::games::texas) fn add_texas_showdown_reveal(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    assets: &UiAssets,
    animation: &GameSummaryAnimation,
    sequence: TexasShowdownSequence,
) {
    if animation.elapsed >= 0.0 {
        return;
    }
    for run in 1..=if game.second_community.is_empty() {
        1
    } else {
        2
    } {
        let Some(hand) = TexasShowdownHand::for_run(game, run) else {
            continue;
        };
        add_round_reveal(commands, table, game, hand, run, assets, sequence);
    }
}

fn add_round_reveal(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    hand: TexasShowdownHand<'_>,
    run: u8,
    assets: &UiAssets,
    sequence: TexasShowdownSequence,
) {
    let round = sequence.round(run);
    let TexasShowdownHand {
        award: main_award,
        winner,
        revealed,
        best,
        board,
    } = hand;
    let root = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    commands.entity(root).insert((
        TexasShowdownRevealRoot,
        round,
        GlobalZIndex(1080),
        Pickable::IGNORE,
    ));
    let backdrop = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.0)),
    );
    commands
        .entity(backdrop)
        .insert((TexasShowdownBackdrop, round, Pickable::IGNORE));

    let layout = TexasTableLayout::for_game(game);
    let relative = layout.player_slot(game, winner.player.id);
    let winner_zone = layout.chip_zone(relative);
    let hole_spacing = if revealed.cards.len() > 2 { 18.0 } else { 31.0 };
    let first_hole_offset = -hole_spacing * (revealed.cards.len().saturating_sub(1) as f32) / 2.0;
    for (index, card) in best.cards().into_iter().enumerate() {
        let hole_index = revealed.cards.iter().position(|hole| *hole == card);
        let (source, delay, start_scale) = if let Some(hole_index) = hole_index {
            (
                Vec2::new(
                    winner_zone.left
                        + winner_zone.width * 0.5
                        + first_hole_offset
                        + hole_index as f32 * hole_spacing,
                    winner_zone.top + 34.0,
                ),
                0.12 + hole_index as f32 * 0.10,
                Vec2::splat(0.55),
            )
        } else {
            let board_index = board
                .iter()
                .position(|community| *community == card)
                .unwrap_or(index);
            (
                TexasTableLayout::board_card_origin(board_index),
                0.68 + board_index as f32 * 0.055,
                TexasTableLayout::BOARD_CARD_SIZE / Vec2::new(70.0, 96.0),
            )
        };
        let target = Vec2::new(444.0 + index as f32 * 78.0, 238.0);
        commands.child(
            root,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(target.x),
                    top: px(target.y),
                    width: px(70),
                    height: px(96),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.face(card))
                    .with_color(Color::WHITE.with_alpha(0.0)),
                BorderColor::all(Color::srgb(0.88, 0.75, 0.35).with_alpha(0.0)),
                UiTransform {
                    translation: Val2::px(source.x - target.x, source.y - target.y),
                    scale: start_scale,
                    ..UiTransform::IDENTITY
                },
                TexasShowdownBestCard {
                    source,
                    target,
                    delay,
                    start_scale,
                },
                round,
                ZIndex(index as i32 + 5),
                Pickable::IGNORE,
            ),
        );
    }

    let title_area = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(360),
            top: px(350),
            width: px(560),
            height: px(42),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(title_area).insert((
        TexasShowdownTitle,
        round,
        UiTransform::from_translation(Val2::px(0.0, 16.0)),
        Pickable::IGNORE,
    ));
    let winner_names = main_award
        .winners
        .iter()
        .filter_map(|id| game.players.iter().find(|player| player.player.id == *id))
        .map(|player| player.player.name.as_str())
        .collect::<Vec<_>>()
        .join("、");
    let title = add_title(
        commands,
        title_area,
        format!("{winner_names} {}", texas_category_label(best.category())),
        30.0,
        Color::NONE,
        assets,
    );
    commands.entity(title).insert(TextShadow {
        offset: Vec2::new(1.2, 1.8),
        color: Color::BLACK.with_alpha(0.72),
    });
    commands
        .entity(title)
        .insert((TexasShowdownTitleText, round));
    let underline = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(470),
            top: px(394),
            width: px(340),
            height: px(2),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        Some(Color::srgb(0.90, 0.72, 0.27).with_alpha(0.0)),
    );
    commands.entity(underline).insert((
        TexasShowdownUnderline,
        round,
        UiTransform {
            scale: Vec2::new(0.0, 1.0),
            ..UiTransform::IDENTITY
        },
        Pickable::IGNORE,
    ));
}
