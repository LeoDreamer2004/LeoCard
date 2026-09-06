//! 七鬼五二三终局结算弹窗与逐行得分演出。

use super::*;
use leocard_protocol::GamePhaseView;
use leocard_protocol::QiGui523Snapshot;

pub fn add_game_summary_modal(
    commands: &mut Commands,
    table: Entity,
    game: &QiGui523Snapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    let GamePhaseView::Finished {
        finisher,
        scores,
        reference_changes,
        ..
    } = &game.phase
    else {
        return;
    };
    let finisher_name = game
        .players
        .iter()
        .find(|player| player.id == *finisher)
        .map_or("玩家", |player| player.name.as_str());
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
        format!("{finisher_name} 率先出完手牌"),
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
    let ranked_scores = sorted_summary_scores(scores);
    for (index, score) in ranked_scores.iter().enumerate() {
        let Some(player) = game.players.iter().find(|player| player.id == score.player) else {
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
            &player.name,
            16.0,
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        let displayed = animated_summary_score(animation.elapsed, score.score, delay);
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
            target: score.score,
            delay,
        });
        if let Some(change) = reference_changes
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

    let actions_delay = SUMMARY_ROW_START_DELAY
        + ranked_scores.len() as f32 * SUMMARY_ROW_INTERVAL
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
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if game.you == game.host {
        add_action_button(
            commands,
            actions,
            "返回房间",
            UiAction::ReturnToLobby,
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
            UiAction::PlayAgain,
            ButtonKind::Primary,
            assets,
        );
    }
}

pub fn add_animated_summary_text(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    delay: f32,
    elapsed: f32,
    assets: &UiAssets,
) -> Entity {
    let opacity = if delay == 0.0 {
        summary_modal_visual(elapsed).opacity
    } else {
        summary_row_progress(elapsed, delay)
    };
    let entity = add_text(
        commands,
        parent,
        text,
        size,
        color.with_alpha(opacity),
        assets,
    );
    commands
        .entity(entity)
        .insert(AnimatedSummaryText { color, delay });
    entity
}
