use super::UNO_FINISH_REVEAL_DURATION;
use crate::app::presentation::add_animated_summary_text;
use crate::app::presentation::{
    ACCENT, AnimatedSummaryScore, ButtonKind, DANGER, GameSummaryActions, GameSummaryAnimation,
    GameSummaryModal, GameSummaryPanelTexture, GameSummaryRow, MUTED, PANEL_ALT, PanelSkin, READY,
    SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_ROW_INTERVAL, SUMMARY_ROW_START_DELAY, SummaryDescriptor,
    TEXT, add_action_button, add_disabled_action_button, add_ready_avatar, animated_summary_score,
    decorate_panel_skin, spawn_node, summary_modal_visual, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{
    PlayerId, PlayerReferenceChange, UnoPhaseView, UnoPlayerResult, UnoSnapshot,
};

pub(crate) fn uno_summary_descriptor(game: &UnoSnapshot) -> Option<SummaryDescriptor> {
    let UnoPhaseView::Finished {
        results,
        reference_changes,
        ..
    } = &game.phase
    else {
        return None;
    };
    Some(SummaryDescriptor {
        match_id: game.match_id,
        texas_hand_number: None,
        settlement_index: None,
        entry_count: results.len(),
        nonnegative_outcome: reference_changes
            .iter()
            .find(|change| change.player == game.you)
            .is_none_or(|change| change.delta >= 0),
        reveal_duration: UNO_FINISH_REVEAL_DURATION,
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "the summary renderer combines game state with explicit animation resources"
)]
pub(super) fn add_uno_summary(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    winner: PlayerId,
    results: &[UnoPlayerResult],
    reference_changes: &[PlayerReferenceChange],
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    let modal_visual = summary_modal_visual(animation.elapsed);
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: percent(3),
            min_height: px(380),
            padding: UiRect::all(px(26)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
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
    let winner_name = game
        .players
        .iter()
        .find(|player| player.id == winner)
        .map(|player| player.name.as_str())
        .unwrap_or("玩家");
    add_animated_summary_text(
        commands,
        modal,
        "本局结算",
        26.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    add_animated_summary_text(
        commands,
        modal,
        format!("{winner_name} 率先出完手牌"),
        14.0,
        TEXT,
        0.0,
        animation.elapsed,
        assets,
    );

    let score_list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let mut ordered = results.to_vec();
    ordered.sort_by_key(|result| result.placement);
    for (index, result) in ordered.iter().enumerate() {
        let Some(player) = game
            .players
            .iter()
            .find(|player| player.id == result.player)
        else {
            continue;
        };
        let delay = SUMMARY_ROW_START_DELAY + index as f32 * SUMMARY_ROW_INTERVAL;
        let row_progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            score_list,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(10), px(4)),
                align_items: AlignItems::Center,
                column_gap: px(10),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.82 * row_progress)),
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - row_progress))),
            if row_progress > 0.0 {
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
            28.0,
            player.ready,
            assets,
        );
        let name = spawn_node(
            commands,
            row,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                ..default()
            },
            None,
        );
        add_animated_summary_text(
            commands,
            name,
            format!("第 {} 名 · {}", result.placement, player.name),
            16.0,
            if result.player == game.you {
                ACCENT
            } else {
                TEXT
            },
            delay,
            animation.elapsed,
            assets,
        );
        add_animated_summary_text(
            commands,
            row,
            if player.eliminated {
                "状态"
            } else {
                "手牌"
            },
            13.0,
            MUTED,
            delay,
            animation.elapsed,
            assets,
        );
        if player.eliminated {
            add_animated_summary_text(
                commands,
                row,
                "已淘汰",
                17.0,
                DANGER,
                delay,
                animation.elapsed,
                assets,
            );
        } else {
            let displayed =
                animated_summary_score(animation.elapsed, u32::from(result.hand_score), delay);
            let score_text = add_animated_summary_text(
                commands,
                row,
                format!("{displayed} 分"),
                19.0,
                ACCENT,
                delay,
                animation.elapsed,
                assets,
            );
            commands.entity(score_text).insert(AnimatedSummaryScore {
                target: u32::from(result.hand_score),
                delay,
            });
        }
        let delta = reference_changes
            .iter()
            .find(|change| change.player == result.player)
            .map_or(0, |change| change.delta);
        add_animated_summary_text(
            commands,
            row,
            format!("{delta:+}"),
            14.0,
            if delta >= 0 { READY } else { DANGER },
            delay,
            animation.elapsed,
            assets,
        );
    }

    let actions_delay = SUMMARY_ROW_START_DELAY
        + ordered.len() as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(52),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
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
            "再来一局",
            UiAction::Lobby(LobbyUiAction::PlayAgain),
            ButtonKind::Primary,
            assets,
        );
    }
}
