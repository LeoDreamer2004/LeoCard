//! 共用排名结算面板；各游戏只提供展示数据。

use crate::app::presentation::{
    ACCENT, AnimatedSummaryScore, DANGER, GameSummaryActions, GameSummaryAnimation,
    GameSummaryModal, GameSummaryPanelTexture, GameSummaryRow, PANEL_ALT, PanelSkin,
    PlayerMenuProfile, READY, SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_ROW_INTERVAL,
    SUMMARY_ROW_START_DELAY, TEXT, add_animated_summary_text, add_ready_avatar,
    animated_summary_score, decorate_panel_skin, sorted_summary_scores, summary_modal_visual,
    summary_row_progress,
};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, PlayerReferenceChange, PlayerScore};
use leocard_ui::{UiComposer, spawn_node};

pub(crate) struct SummaryPlayer<'a> {
    pub id: PlayerId,
    pub profile: PlayerMenuProfile<'a>,
    pub ready: bool,
}

pub(crate) struct RankedSummary<'a> {
    pub players: Vec<SummaryPlayer<'a>>,
    pub scores: &'a [PlayerScore],
    pub reference_changes: &'a [PlayerReferenceChange],
    pub title: &'a str,
    pub caption: &'a str,
    pub score_unit: &'static str,
}

/// 返回随结算演出显现的操作区，具体按钮由调用方提供。
pub(crate) fn add_ranked_summary(
    commands: &mut UiComposer,
    table: Entity,
    game: &RankedSummary<'_>,
    assets: &UiAssets,
    animation: &GameSummaryAnimation,
) -> Entity {
    let scores = game.scores;
    let reference_changes = game.reference_changes;
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
        Pickable::default(),
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
        game.title,
        26.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    spawn_node(
        commands,
        modal,
        Node {
            width: px(86),
            height: px(2),
            ..default()
        },
        Some(Color::srgb(0.64, 0.59, 0.93)),
    );
    add_animated_summary_text(
        commands,
        modal,
        game.caption,
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
        add_ready_avatar(
            commands,
            row,
            player.id,
            player.profile,
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
            player.profile.name,
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
            format!("{displayed} {}", game.score_unit),
            19.0,
            ACCENT,
            delay,
            animation.elapsed,
            assets,
        );
        commands.entity(score_text).insert(AnimatedSummaryScore {
            target: score.score,
            delay,
            unit: game.score_unit,
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
    actions
}
