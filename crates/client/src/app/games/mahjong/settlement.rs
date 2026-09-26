use super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongWinTileSizes,
    mahjong_win_reveal_duration, render_mahjong_win_tile_row,
};
use crate::app::presentation::add_animated_summary_text;
use crate::app::presentation::{
    ACCENT, ButtonKind, DANGER, GameSummaryActions, GameSummaryAnimation, GameSummaryDivider,
    GameSummaryModal, GameSummaryPanelTexture, GameSummaryRow, MUTED, PANEL_ALT, PanelSkin, READY,
    SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_ROW_INTERVAL, SUMMARY_ROW_START_DELAY, SummaryDescriptor,
    TEXT, add_action_button, add_avatar, add_disabled_action_button, add_ready_avatar,
    decorate_panel_skin, spawn_node, summary_modal_visual, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::MahjongMatchLength;
use leocard_protocol::{MahjongHandResultView, MahjongPhaseView, MahjongSnapshot};

const FAN_INTERVAL: f32 = 0.14;

pub(crate) fn mahjong_summary_descriptor(game: &MahjongSnapshot) -> Option<SummaryDescriptor> {
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return None;
    };
    let winner_delay = result
        .winners
        .iter()
        .map(|winner| 0.50 + winner.score.fans.len() as f32 * FAN_INTERVAL)
        .sum::<f32>();
    let final_standings =
        result.match_complete && game.rules.match_length != MahjongMatchLength::SingleHand;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        texas_hand_number: None,
        settlement_index: Some(u32::from(result.sequence_index)),
        entry_count: game.players.len() * (1 + usize::from(final_standings))
            + (winner_delay / SUMMARY_ROW_INTERVAL).ceil() as usize,
        nonnegative_outcome: if result.match_complete {
            result
                .reference_changes
                .iter()
                .find(|change| change.player == game.you)
                .is_none_or(|change| change.delta >= 0)
        } else {
            result.deltas[usize::from(game.you.0)] >= 0
        },
        reveal_duration: if result.winners.is_empty() {
            0.0
        } else {
            mahjong_win_reveal_duration(result)
        },
    })
}

pub(super) struct MahjongSettlementVisuals<'a> {
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub animation: &'a GameSummaryAnimation,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(super) fn render_mahjong_settlement(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    result: &MahjongHandResultView,
    visuals: MahjongSettlementVisuals<'_>,
) {
    let MahjongSettlementVisuals {
        assets,
        avatars,
        animation,
        game_assets,
        materials,
    } = visuals;
    let multiple_hands = game.rules.match_length != MahjongMatchLength::SingleHand;
    let final_standings = multiple_hands && result.match_complete;

    let modal_visual = summary_modal_visual(animation.elapsed);
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(if multiple_hands { 12 } else { 22 }),
            right: percent(if multiple_hands { 12 } else { 22 }),
            top: percent(2),
            min_height: px(if multiple_hands { 0 } else { 390 }),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(if multiple_hands { 4 } else { 6 }),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        None,
    );
    commands.entity(modal).insert((
        GameSummaryModal,
        UiTransform::from_translation(Val2::px(0.0, modal_visual.offset_y)),
        GlobalZIndex(1200),
        FocusPolicy::Block,
        if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    let texture = decorate_panel_skin(commands, modal, PanelSkin::Window, assets);
    commands.entity(texture).insert(GameSummaryPanelTexture);
    add_animated_summary_text(
        commands,
        modal,
        "本局结算",
        if multiple_hands { 25.0 } else { 26.0 },
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    if result.exhaustive_draw {
        add_animated_summary_text(
            commands,
            modal,
            "本局荒牌",
            14.0,
            MUTED,
            0.0,
            animation.elapsed,
            assets,
        );
    }

    let mut next_delay = SUMMARY_ROW_START_DELAY;
    for winner in &result.winners {
        let name = game
            .players
            .iter()
            .find(|player| player.id == winner.player)
            .map(|player| player.name.as_str())
            .unwrap_or("玩家");
        let outcome = winner.from.map_or_else(
            || format!("{name} 自摸 {}番", winner.score.total_points),
            |from| {
                let source = game
                    .players
                    .iter()
                    .find(|player| player.id == from)
                    .map(|player| player.name.as_str())
                    .unwrap_or("玩家");
                format!("{source} 放炮给 {name} {}番", winner.score.total_points)
            },
        );
        add_animated_summary_text(
            commands,
            modal,
            outcome,
            17.0,
            READY,
            next_delay,
            animation.elapsed,
            assets,
        );
        next_delay += FAN_INTERVAL;
        let hand_delay = next_delay;
        let hand_progress = summary_row_progress(animation.elapsed, hand_delay);
        let hand = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                min_height: px(49),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        commands.entity(hand).insert((
            GameSummaryRow { delay: hand_delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - hand_progress))),
            if hand_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        if let Some(player) = game
            .players
            .iter()
            .find(|player| player.id == winner.player)
        {
            render_mahjong_win_tile_row(
                commands,
                hand,
                player,
                winner,
                MahjongWinTileSizes {
                    meld: MahjongTileSize::GuideHand,
                    hand: MahjongTileSize::GuideHand,
                },
                game_assets,
                materials,
            );
        }
        next_delay += 0.28;
        let fans = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                min_height: px(30),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: px(6),
                row_gap: px(6),
                ..default()
            },
            None,
        );
        for fan in &winner.score.fans {
            let delay = next_delay;
            let progress = summary_row_progress(animation.elapsed, delay);
            let color = if fan.points >= 48 {
                Color::srgb(0.98, 0.76, 0.25)
            } else if fan.points >= 6 {
                READY
            } else {
                TEXT
            };
            let badge = spawn_node(
                commands,
                fans,
                Node {
                    min_height: px(28),
                    padding: UiRect::axes(px(9), px(4)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                Some(PANEL_ALT.with_alpha(0.82 * progress)),
            );
            commands.entity(badge).insert((
                GameSummaryRow { delay },
                BorderColor::all(color.with_alpha(0.72)),
                UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
                if progress > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
            let fan_name = if fan.count > 1 {
                format!("{}×{}", fan.fan.name(), fan.count)
            } else {
                fan.fan.name().to_owned()
            };
            let text = add_animated_summary_text(
                commands,
                badge,
                format!("{fan_name}  {}番", fan.points),
                if fan.points >= 48 { 14.5 } else { 13.0 },
                color,
                delay,
                animation.elapsed,
                assets,
            );
            if fan.points >= 48 {
                commands.entity(text).insert(TextShadow {
                    offset: Vec2::new(0.8, 0.0),
                    color: color.with_alpha(0.82),
                });
            }
            next_delay += FAN_INTERVAL;
        }
        next_delay += 0.08;
    }

    let list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(if multiple_hands { 3 } else { 4 }),
            ..default()
        },
        None,
    );
    let mut players = game.players.iter().collect::<Vec<_>>();
    players.sort_by(|left, right| {
        result.match_scores[right.id.0 as usize]
            .cmp(&result.match_scores[left.id.0 as usize])
            .then_with(|| left.seat.0.cmp(&right.seat.0))
    });
    for (index, player) in players.iter().enumerate() {
        let delay = next_delay + index as f32 * SUMMARY_ROW_INTERVAL;
        let progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                height: if multiple_hands { px(47) } else { auto() },
                min_height: px(if multiple_hands { 47 } else { 38 }),
                padding: UiRect::axes(px(if multiple_hands { 9 } else { 10 }), px(3)),
                align_items: AlignItems::Center,
                column_gap: px(if multiple_hands { 8 } else { 10 }),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.82 * progress)),
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
            if progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
        add_ready_avatar(
            commands,
            row,
            &player.name,
            avatar,
            if multiple_hands { 32.0 } else { 28.0 },
            player.ready,
            assets,
        );
        let name = add_animated_summary_text(
            commands,
            row,
            &player.name,
            if multiple_hands { 15.5 } else { 16.0 },
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        commands.entity(name).insert((
            Node {
                width: if multiple_hands { px(120) } else { auto() },
                min_width: if multiple_hands { px(120) } else { px(0) },
                max_width: if multiple_hands { px(120) } else { auto() },
                flex_grow: if multiple_hands { 0.0 } else { 1.0 },
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            },
            TextLayout::no_wrap(),
        ));
        let delta = result.deltas[player.id.0 as usize];
        let total = result.match_scores[player.id.0 as usize];
        if multiple_hands {
            spawn_node(
                commands,
                row,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            add_animated_summary_text(
                commands,
                row,
                format!("累计 {total:+}"),
                16.0,
                ACCENT,
                delay,
                animation.elapsed,
                assets,
            );
        }
        add_animated_summary_text(
            commands,
            row,
            if !multiple_hands {
                format!("{delta:+} 分")
            } else {
                format!("{delta:+}")
            },
            if multiple_hands { 19.0 } else { 16.0 },
            if delta > 0 {
                READY
            } else if delta < 0 {
                DANGER
            } else {
                TEXT
            },
            delay,
            animation.elapsed,
            assets,
        );
        if !multiple_hands {
            let change = result
                .reference_changes
                .iter()
                .find(|change| change.player == player.id)
                .map_or(0, |change| change.delta);
            add_animated_summary_text(
                commands,
                row,
                format!("积分 {change:+}"),
                15.0,
                if change >= 0 { READY } else { DANGER },
                delay,
                animation.elapsed,
                assets,
            );
        }
    }
    let divider_delay = next_delay + players.len() as f32 * SUMMARY_ROW_INTERVAL;
    if final_standings {
        let divider_progress = summary_row_progress(animation.elapsed, divider_delay);
        let divider = spawn_node(
            commands,
            modal,
            Node {
                width: percent(96),
                height: px(1),
                margin: UiRect::vertical(px(1)),
                align_self: AlignSelf::Center,
                ..default()
            },
            Some(Color::srgb(0.52, 0.55, 0.54).with_alpha(0.28 * divider_progress)),
        );
        commands.entity(divider).insert((
            GameSummaryDivider {
                delay: divider_delay,
            },
            if divider_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        for (index, player) in players.iter().enumerate() {
            let delay = divider_delay + index as f32 * SUMMARY_ROW_INTERVAL;
            let progress = summary_row_progress(animation.elapsed, delay);
            let row = spawn_node(
                commands,
                modal,
                Node {
                    width: percent(100),
                    height: px(35),
                    padding: UiRect::axes(px(10), px(3)),
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                Some(PANEL_ALT.with_alpha(0.82 * progress)),
            );
            commands.entity(row).insert((
                GameSummaryRow { delay },
                UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
                if progress > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
            let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
            add_avatar(commands, row, &player.name, avatar, 26.0, assets);
            add_animated_summary_text(
                commands,
                row,
                format!("{}. {}", index + 1, player.name),
                14.5,
                TEXT,
                delay,
                animation.elapsed,
                assets,
            );
            let spacer = spawn_node(
                commands,
                row,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            commands.entity(spacer).insert(FocusPolicy::Pass);
            add_animated_summary_text(
                commands,
                row,
                format!("总分 {:+}", result.match_scores[player.id.0 as usize]),
                16.0,
                ACCENT,
                delay,
                animation.elapsed,
                assets,
            );
            if let Some(change) = result
                .reference_changes
                .iter()
                .find(|change| change.player == player.id)
            {
                add_animated_summary_text(
                    commands,
                    row,
                    format!("{:+}", change.delta),
                    15.0,
                    if change.delta >= 0 { READY } else { DANGER },
                    delay,
                    animation.elapsed,
                    assets,
                );
            }
        }
    }
    let actions_delay = divider_delay
        + if final_standings {
            players.len() as f32 * SUMMARY_ROW_INTERVAL
        } else {
            0.0
        }
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            height: if multiple_hands { px(46) } else { auto() },
            min_height: px(if multiple_hands { 46 } else { 48 }),
            align_items: AlignItems::Center,
            justify_content: if multiple_hands {
                JustifyContent::Center
            } else {
                JustifyContent::SpaceBetween
            },
            column_gap: px(10),
            ..default()
        },
        None,
    );
    commands.entity(actions).insert((
        GameSummaryActions {
            delay: actions_delay,
        },
        if animation.elapsed >= actions_delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    if final_standings {
        add_action_button(
            commands,
            actions,
            "返回大厅",
            UiAction::Lobby(LobbyUiAction::ReturnToLobby),
            ButtonKind::Primary,
            assets,
        );
    } else {
        if !multiple_hands {
            add_action_button(
                commands,
                actions,
                "退出游戏",
                UiAction::Lobby(LobbyUiAction::LeaveRoom),
                ButtonKind::Pass,
                assets,
            );
            if game.you == game.host {
                add_action_button(
                    commands,
                    actions,
                    "返回房间",
                    UiAction::Lobby(LobbyUiAction::ReturnToLobby),
                    ButtonKind::Warning,
                    assets,
                );
            }
        }
        let ready = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.ready);
        if ready {
            add_disabled_action_button(commands, actions, "已准备", assets);
        } else {
            add_action_button(
                commands,
                actions,
                if multiple_hands {
                    "准备下一局"
                } else {
                    "再来一局"
                },
                UiAction::Lobby(LobbyUiAction::PlayAgain),
                ButtonKind::Primary,
                assets,
            );
        }
    }
}
