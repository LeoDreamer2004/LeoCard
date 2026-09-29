use super::state::{ScoreCardsPopupPlacement, SeatVisuals};
use super::{add_round_play_for_optional_player, add_score_cards_popup, sort_cards_high_to_low};
use crate::app::presentation::CardSize;
use crate::app::presentation::{
    ACCENT, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT, TurnBorderAnimationKey,
    TurnBorderMaterial, add_card_image, add_player_portrait, add_text,
    add_turn_border_trace_with_radius, attach_start_game_seat_transition, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    FinishedHandScoreSource, OpponentBadge, PlayerGameScoreText, SeatSide, displayed_captured_score,
};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::QiGui523Snapshot;
use leocard_protocol::{GameKind, GamePhaseView, PlayerId, PlayerPublicState};
use leocard_qigui523::QiGuiCard;

pub(super) const QIGUI_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(super) const QIGUI_AVATAR_SIZE: f32 = 52.0 * 1.17;
pub(super) const QIGUI_PORTRAIT_HEIGHT: f32 = 76.0 * 1.17 + 34.0;

pub(super) fn add_opponent_slot(
    commands: &mut Commands,
    table: Entity,
    relative_seat: u8,
    player: Option<&PlayerPublicState>,
    active: bool,
    visuals: &SeatVisuals,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
) {
    const SIDE_SLOT_INSET: f32 = 78.0;
    const SIDE_PLAY_GAP: f32 = 24.0;
    const SIDE_SLOT_WIDTH: f32 = QIGUI_PORTRAIT_WIDTH + SIDE_PLAY_GAP + 148.0;
    let side = match relative_seat {
        1 | 2 => SeatSide::Left,
        3 => SeatSide::Top,
        4 | 5 => SeatSide::Right,
        _ => unreachable!("the local player occupies relative seat zero"),
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        min_height: px(if matches!(side, SeatSide::Top) {
            140
        } else {
            124
        }),
        align_items: AlignItems::Center,
        justify_content: match side {
            SeatSide::Left => JustifyContent::FlexStart,
            SeatSide::Top => JustifyContent::Center,
            SeatSide::Right => JustifyContent::FlexEnd,
        },
        // 两侧出牌区保持原位，让头像向各自的出牌区靠拢。
        column_gap: px(if matches!(side, SeatSide::Top) {
            8.0
        } else {
            SIDE_PLAY_GAP
        }),
        row_gap: px(5),
        ..default()
    };
    match relative_seat {
        1 => {
            node.left = px(SIDE_SLOT_INSET);
            node.bottom = px(36);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        2 => {
            node.left = px(SIDE_SLOT_INSET);
            node.top = px(78);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        3 => {
            node.left = percent(32);
            node.right = percent(32);
            node.top = px(4);
        }
        4 => {
            node.right = px(SIDE_SLOT_INSET);
            node.top = px(78);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        5 => {
            node.right = px(SIDE_SLOT_INSET);
            node.bottom = px(36);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        _ => unreachable!(),
    }
    let slot = spawn_node(commands, table, node, None);
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
        commands.entity(spacer).insert(FocusPolicy::Pass);
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
        player.is_some_and(|player| visuals.interaction_menu_open == Some(player.id));
    match player {
        Some(player) => {
            let avatar = player.avatar.and_then(|id| visuals.avatars.remote.get(&id));
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
                    player: player.id,
                    profile: PlayerMenuProfile {
                        name: &player.name,
                        avatar,
                        reference_points: player.reference_points,
                        completed_games: player.completed_games,
                        game_profiles: &player.game_profiles,
                    },
                    side,
                    avatar_size: QIGUI_AVATAR_SIZE,
                    auto_play: player.auto_play,
                    menu_open: interaction_menu_open,
                    menu_above: matches!(relative_seat, 1 | 5),
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
                player.id,
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
                        player.id,
                    ),
                    QIGUI_AVATAR_SIZE * 0.2,
                    QIGUI_AVATAR_SIZE,
                );
            }
            let score = displayed_captured_score(visuals.score_capture, player.id, player.score);
            add_qigui_score_value(
                commands,
                portrait.portrait,
                player.id,
                side,
                score,
                visuals.ui,
            );
            if let Some(cards) = finished_remaining_hand(visuals.game, player.id)
                && !cards.is_empty()
            {
                add_finished_remaining_hand(
                    commands,
                    portrait.portrait,
                    player.id,
                    cards,
                    matches!(relative_seat, 1 | 5),
                    visuals.ui,
                );
            }
            let score_popup = add_score_cards_popup(
                commands,
                portrait.portrait,
                player,
                visuals.client.0.model().captured_score_cards(player.id),
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

pub(super) fn add_qigui_score_value(
    commands: &mut Commands,
    portrait: Entity,
    player: PlayerId,
    side: SeatSide,
    score: u32,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        portrait,
        Node {
            position_type: PositionType::Absolute,
            top: px(76.0 * 1.17 + 2.0),
            width: percent(100),
            height: px(31),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    commands.entity(row).insert(FocusPolicy::Pass);
    let icon = spawn_node(
        commands,
        row,
        Node {
            width: px(25),
            height: px(25),
            flex_shrink: 0.0,
            border: UiRect::all(px(1.5)),
            border_radius: BorderRadius::all(percent(50)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(icon)
        .insert((BorderColor::all(ACCENT.with_alpha(0.85)), FocusPolicy::Pass));
    let glyph = add_text(commands, icon, "分", 15.0, ACCENT, assets);
    commands.entity(glyph).insert(FocusPolicy::Pass);
    let digits = score.to_string();
    let font_size = (28.0 - digits.len().saturating_sub(3) as f32 * 2.5).max(17.0);
    let value = add_text(commands, row, digits, font_size, ACCENT, assets);
    commands.entity(value).insert((
        PlayerGameScoreText::Opponent { player, side },
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        },
        FocusPolicy::Pass,
    ));
}

fn finished_remaining_hand(game: &QiGui523Snapshot, player: PlayerId) -> Option<&[QiGuiCard]> {
    let GamePhaseView::Finished {
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
    commands: &mut Commands,
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
        FocusPolicy::Pass,
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
