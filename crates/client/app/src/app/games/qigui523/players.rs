use super::state::{ScoreCardsPopupPlacement, SeatVisuals};
use super::{
    add_qigui_player_stats, add_round_play_for_optional_player, add_score_cards_popup,
    sort_cards_high_to_low,
};
use crate::app::presentation::{
    ACCENT, CardSize, MUTED, PlayerMenuProfile, PlayerPortraitSpec, SeatSide, TEXT,
    TrickSeatLayout, TurnBorderAnimationKey, TurnBorderMaterial, add_card_image,
    add_player_portrait, add_turn_border_trace_with_radius, attach_start_game_seat_transition,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{FinishedHandScoreSource, OpponentBadge, displayed_captured_score};
use bevy::picking::Pickable;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, QiGui523PhaseView, QiGui523PlayerView, QiGui523Snapshot};
use leocard_qigui523::QiGuiCard;
use leocard_ui::{UiComposer, spawn_node};

use bevy::prelude::*;

pub(super) const QIGUI_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(super) const QIGUI_AVATAR_SIZE: f32 = 52.0 * 1.17;
pub(super) const QIGUI_PORTRAIT_HEIGHT: f32 = 76.0 * 1.17 + 34.0;

pub(super) fn add_opponent_slot(
    commands: &mut UiComposer,
    table: Entity,
    relative_seat: u8,
    player: Option<&QiGui523PlayerView>,
    active: bool,
    visuals: &SeatVisuals,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
) {
    let layout = TrickSeatLayout::new(relative_seat, 6, 148.0);
    let side = layout.side;
    let slot = layout.spawn(commands, table);
    if matches!(side, SeatSide::Top) {
        let spacer = spawn_node(
            commands,
            slot,
            Node {
                width: px(148),
                min_width: px(148),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(spacer).insert(Pickable::IGNORE);
    }
    if matches!(side, SeatSide::Right) {
        add_round_play_for_optional_player(
            commands,
            slot,
            side,
            visuals.game,
            if visuals.intro_only { None } else { player },
            visuals.play_effect,
            visuals.last_play,
            visuals.ui,
            visuals.assets,
        );
    }
    let interaction_menu_open =
        player.is_some_and(|player| visuals.interaction_menu_open == Some(player.player.id));
    match player {
        Some(player) => {
            let avatar = player
                .player
                .avatar
                .and_then(|id| visuals.avatars.remote.get(&id));
            let portrait = add_player_portrait(
                commands,
                slot,
                Node {
                    width: px(QIGUI_PORTRAIT_WIDTH),
                    min_width: px(QIGUI_PORTRAIT_WIDTH),
                    height: px(QIGUI_PORTRAIT_HEIGHT),
                    flex_shrink: 0.0,
                    ..default()
                },
                PlayerPortraitSpec {
                    player: player.player.id,
                    profile: PlayerMenuProfile {
                        name: &player.player.name,
                        avatar,
                        reference_points: player.player.reference_points,
                        completed_games: player.player.completed_games,
                        game_profiles: &player.player.game_profiles,
                    },
                    avatar_size: QIGUI_AVATAR_SIZE,
                    auto_play: player.auto_play,
                    menu_open: interaction_menu_open,
                    name_color: if !player.connected {
                        MUTED
                    } else if active {
                        ACCENT
                    } else {
                        TEXT
                    },
                },
                visuals.ui,
            );
            attach_start_game_seat_transition(
                commands,
                portrait.portrait,
                player.player.id,
                visuals.start_transition_active,
            );
            if visuals.intro_only {
                if !matches!(side, SeatSide::Right) {
                    add_round_play_for_optional_player(
                        commands,
                        slot,
                        side,
                        visuals.game,
                        None,
                        visuals.play_effect,
                        visuals.last_play,
                        visuals.ui,
                        visuals.assets,
                    );
                }
                return;
            }
            if active {
                add_turn_border_trace_with_radius(
                    commands,
                    portrait.avatar_ring,
                    turn_border_materials,
                    TurnBorderAnimationKey::new(
                        GameKind::QiGui523,
                        visuals.game.match_id,
                        player.player.id,
                    ),
                    QIGUI_AVATAR_SIZE * 0.2,
                    QIGUI_AVATAR_SIZE,
                );
            }
            let score =
                displayed_captured_score(visuals.score_capture, player.player.id, player.score);
            add_qigui_player_stats(
                commands,
                portrait.portrait,
                player.player.id,
                side,
                score,
                player.hand_len,
                visuals.ui,
            );
            if let Some(cards) = finished_remaining_hand(visuals.game, player.player.id)
                && !cards.is_empty()
            {
                add_finished_remaining_hand(
                    commands,
                    portrait.portrait,
                    player.player.id,
                    cards,
                    matches!(relative_seat, 1 | 5),
                    visuals.ui,
                );
            }
            let score_popup = add_score_cards_popup(
                commands,
                portrait.portrait,
                player,
                visuals.client_state.captured_score_cards(player.player.id),
                ScoreCardsPopupPlacement::Opponent {
                    side,
                    above: matches!(relative_seat, 1 | 5),
                },
                visuals.score_capture,
                visuals.ui,
            );
            commands.entity(score_popup).insert(Visibility::Hidden);
            commands
                .entity(portrait.portrait)
                .entry::<OpponentBadge>()
                .and_modify(move |mut badge| badge.score_popup = Some(score_popup));
        }
        None => {
            spawn_node(
                commands,
                slot,
                Node {
                    width: px(QIGUI_PORTRAIT_WIDTH),
                    height: px(QIGUI_PORTRAIT_HEIGHT),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
        }
    }
    if !matches!(side, SeatSide::Right) {
        add_round_play_for_optional_player(
            commands,
            slot,
            side,
            visuals.game,
            if visuals.intro_only { None } else { player },
            visuals.play_effect,
            visuals.last_play,
            visuals.ui,
            visuals.assets,
        );
    }
}

fn finished_remaining_hand(game: &QiGui523Snapshot, player: PlayerId) -> Option<&[QiGuiCard]> {
    let QiGui523PhaseView::Finished {
        remaining_hands, ..
    } = &game.phase
    else {
        return None;
    };
    remaining_hands
        .iter()
        .find(|hand| hand.player == player)
        .map(|hand| hand.cards.as_slice())
}

fn add_finished_remaining_hand(
    commands: &mut UiComposer,
    badge: Entity,
    player: PlayerId,
    cards: &[QiGuiCard],
    above: bool,
    assets: &UiAssets,
) {
    let hand = spawn_node(
        commands,
        badge,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: if above {
                Val::Auto
            } else {
                px(QIGUI_PORTRAIT_HEIGHT + 2.0)
            },
            bottom: if above {
                px(QIGUI_PORTRAIT_HEIGHT + 2.0)
            } else {
                Val::Auto
            },
            width: percent(100),
            height: px(CardSize::FinishedHand.dimensions().1),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(hand).insert((
        FinishedHandScoreSource(player),
        GlobalZIndex(850),
        Pickable::IGNORE,
    ));
    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    let last_card = displayed_cards.len().saturating_sub(1);
    for (index, card) in displayed_cards.into_iter().enumerate() {
        add_card_image(
            commands,
            hand,
            card,
            CardSize::FinishedHand,
            index,
            index == last_card,
            false,
            assets,
        );
    }
}
