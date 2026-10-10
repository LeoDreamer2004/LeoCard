//! 国标麻将报番与结算；多局模式另播放牌桌分数演出。

use super::super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongUiAction, MahjongWinTileSizes,
    mahjong_settlement_timing, render_mahjong_win_tile_row,
};
use super::*;
use super::{
    continuation::render_settlement_continue, final_summary::FinalSummaryView,
    scores::ScoreStageView,
};
use crate::app::presentation::{
    ACCENT, GameSummaryAnimation, MUTED, PanelSkin, READY, TEXT, add_avatar, add_text,
    decorate_panel_skin, spawn_node, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot};

pub(super) fn settlement_row_texture(assets: &UiAssets) -> ImageNode {
    let mut texture = ImageNode::new(assets.home.game_card.clone()).with_mode(
        NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(22.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.42,
        }),
    );
    texture.visual_box = bevy::ui::VisualBox::BorderBox;
    texture
}

pub(in crate::app::games::mahjong) struct MahjongSettlementVisuals<'a> {
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub animation: &'a GameSummaryAnimation,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
    pub fan_summary_continued: bool,
    pub final_summary_opened_at: Option<f32>,
}

pub(in crate::app::games::mahjong) fn render_mahjong_settlement(
    commands: &mut Commands,
    table: Entity,
    content: Entity,
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
        fan_summary_continued,
        final_summary_opened_at,
    } = visuals;
    let timing = mahjong_settlement_timing(result);
    let timeline = SettlementTimeline::new(result);
    let final_standings = result.match_complete;
    let final_summary_open = final_standings && final_summary_opened_at.is_some();
    // 牌桌和右侧抽屉一同压暗；胜家页面和头像舞台另绘在遮罩之上。
    let shade = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(
            Color::BLACK
                .with_alpha(0.80 * (animation.elapsed / SCORE_FADE_DURATION).clamp(0.0, 1.0)),
        ),
    );
    commands.entity(shade).insert((
        MahjongScoreShade { start: 0.0 },
        GlobalZIndex(1180),
        Pickable::default(),
    ));

    for (index, (winner, winner_timing)) in result
        .winners
        .iter()
        .zip(&timing.winners)
        .enumerate()
        .filter(|_| !final_summary_open)
    {
        let end = timing
            .winners
            .get(index + 1)
            .map_or(timeline.pages_end, |next| next.outcome_delay);
        let player = game
            .players
            .iter()
            .find(|player| player.id == winner.player);
        let name = player.map_or("玩家", |player| player.name.as_str());
        let panel = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(154),
                right: px(154),
                top: px(70),
                bottom: px(62),
                padding: UiRect::axes(px(32), px(25)),
                flex_direction: FlexDirection::Column,
                row_gap: px(17),
                ..default()
            },
            None,
        );
        commands.entity(panel).insert((
            MahjongFanPage {
                start: winner_timing.outcome_delay,
                end,
            },
            GlobalZIndex(1200),
            Pickable::default(),
            if animation.elapsed >= winner_timing.outcome_delay && animation.elapsed < end {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        decorate_panel_skin(commands, panel, PanelSkin::Window, assets);
        let heading = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                height: px(75),
                align_items: AlignItems::Center,
                column_gap: px(15),
                ..default()
            },
            None,
        );
        if let Some(player) = player {
            let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
            add_avatar(commands, heading, name, avatar, 62.0, assets);
        }
        let title = spawn_node(
            commands,
            heading,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            },
            None,
        );
        add_text(commands, title, name, 25.0, TEXT, assets);
        let outcome = winner.from.map_or_else(
            || "自摸和牌".to_owned(),
            |from| {
                let source = game
                    .players
                    .iter()
                    .find(|player| player.id == from)
                    .map_or("玩家", |player| player.name.as_str());
                format!("{source} 放炮")
            },
        );
        add_text(commands, title, outcome, 15.0, MUTED, assets);
        let spacer = spawn_node(
            commands,
            heading,
            Node {
                flex_grow: 1.0,
                ..default()
            },
            None,
        );
        commands.entity(spacer).insert(Pickable::IGNORE);
        let total = add_text(
            commands,
            heading,
            format!("{} 番", winner.score.total_points),
            40.0,
            Color::srgb(0.98, 0.79, 0.40),
            assets,
        );
        commands.entity(total).insert((
            MahjongFanEntry {
                delay: winner_timing.total_delay,
                end,
            },
            UiTransform::default(),
            if animation.elapsed >= winner_timing.total_delay && animation.elapsed < end {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        let rule = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                height: px(2),
                ..default()
            },
            Some(ACCENT.with_alpha(0.7)),
        );
        commands.entity(rule).insert(Pickable::IGNORE);
        let hand = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                min_height: px(72),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        commands.entity(hand).insert((
            MahjongFanEntry {
                delay: winner_timing.hand_delay,
                end,
            },
            if animation.elapsed >= winner_timing.hand_delay && animation.elapsed < end {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            UiTransform::default(),
        ));
        if let Some(player) = player {
            render_mahjong_win_tile_row(
                commands,
                hand,
                player,
                winner,
                MahjongWinTileSizes {
                    meld: MahjongTileSize::SettlementHand,
                    hand: MahjongTileSize::SettlementHand,
                },
                game_assets,
                materials,
            );
        }
        let fan_area = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                column_gap: px(8),
                row_gap: px(7),
                overflow: Overflow::clip(),
                ..default()
            },
            None,
        );
        let dense_fans = winner.score.fans.len() > 12;
        for (fan, &delay) in winner.score.fans.iter().zip(&winner_timing.fan_delays) {
            let color = if fan.points >= 48 {
                Color::srgb(0.98, 0.79, 0.40)
            } else if fan.points >= 6 {
                READY
            } else {
                TEXT
            };
            let row = spawn_node(
                commands,
                fan_area,
                Node {
                    width: percent(if dense_fans { 32 } else { 49 }),
                    min_height: px(if dense_fans { 35 } else { 38 }),
                    padding: UiRect::axes(px(if dense_fans { 11 } else { 16 }), px(5)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                None,
            );
            commands.entity(row).insert((
                MahjongFanEntry { delay, end },
                UiTransform::from_translation(Val2::px(
                    0.0,
                    12.0 * (1.0 - summary_row_progress(animation.elapsed, delay)),
                )),
                if animation.elapsed >= delay && animation.elapsed < end {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
            commands
                .entity(row)
                .insert((settlement_row_texture(assets), Pickable::IGNORE));
            let name = if fan.count > 1 {
                format!("{} × {}", fan.fan.name(), fan.count)
            } else {
                fan.fan.name().to_owned()
            };
            add_text(
                commands,
                row,
                name,
                if dense_fans { 15.0 } else { 17.0 },
                color,
                assets,
            );
            spawn_node(
                commands,
                row,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            add_text(
                commands,
                row,
                format!("{} 番", fan.points),
                if dense_fans { 16.0 } else { 18.0 },
                color,
                assets,
            );
        }
        let curtain = spawn_node(
            commands,
            panel,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            Some(
                Color::srgb(0.035, 0.039, 0.055).with_alpha(
                    ((animation.elapsed - (end - PAGE_FADE_DURATION)) / PAGE_FADE_DURATION)
                        .clamp(0.0, 1.0),
                ),
            ),
        );
        commands.entity(curtain).insert((
            MahjongFanPageCurtain { end },
            ZIndex(20),
            Pickable::default(),
        ));
    }

    if !fan_summary_continued && let Some(pause_at) = mahjong_fan_pause_at(result) {
        render_settlement_continue(
            commands,
            content,
            assets,
            animation.elapsed,
            pause_at,
            timeline.pages_end,
            "点击任意位置继续",
            Some(UiAction::Mahjong(MahjongUiAction::ContinueFanSummary)),
        );
    }

    if let Some(timing) = timeline.scores {
        ScoreStageView {
            game,
            result,
            assets,
            game_assets,
            elapsed: animation.elapsed,
            timing,
            end: if final_summary_open {
                timeline.continue_at
            } else {
                f32::INFINITY
            },
        }
        .render(commands, table);
    }

    if let Some(opened_at) = final_summary_opened_at.filter(|_| final_standings) {
        FinalSummaryView {
            game,
            result,
            assets,
            avatars,
            opened_at,
        }
        .render(commands, table);
    }

    if !final_summary_open {
        let ready = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.ready);
        render_settlement_continue(
            commands,
            content,
            assets,
            animation.elapsed,
            timeline.continue_at,
            f32::INFINITY,
            if !final_standings && ready {
                "等待其他玩家"
            } else {
                "点击任意位置继续"
            },
            if final_standings {
                Some(UiAction::Mahjong(MahjongUiAction::ShowFinalSummary))
            } else if ready {
                None
            } else {
                Some(UiAction::Lobby(LobbyUiAction::PlayAgain))
            },
        );
    }
}
