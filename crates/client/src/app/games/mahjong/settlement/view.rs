//! 国标麻将结算：逐位报番，然后在牌桌座位上结算分数。

use super::super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongUiAction, MahjongWinTileSizes,
    mahjong_settlement_timing, render_mahjong_win_tile_row, render_round_status_with_scores,
};
use super::continuation::render_settlement_continue;
use super::final_summary::FinalSummaryView;
use super::*;
use crate::app::presentation::{
    ACCENT, DANGER, GameSummaryAnimation, MUTED, PanelSkin, READY, TEXT, add_avatar, add_text,
    decorate_panel_skin, spawn_node, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot};

pub(super) const PAGE_FADE_DURATION: f32 = 0.28;
pub(super) const SCORE_FADE_DURATION: f32 = 0.36;
pub(super) const DELTA_FLIGHT_DURATION: f32 = 0.62;
pub(super) const SCORE_ROLL_DURATION: f32 = 0.80;
pub(super) const DELTA_APPEAR_DELAY: f32 = 0.14;
pub(super) const DELTA_HOLD_DURATION: f32 = 0.75;
pub(super) const SCORE_END_HOLD_DURATION: f32 = 0.65;

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
    let score_start = timing.score_rows_delay + if result.winners.is_empty() { 0.0 } else { 0.12 };
    let (flight_start, roll_start, continue_start) = score_timing(score_start);
    let final_standings = result.match_complete;
    let final_summary_open = final_standings && final_summary_opened_at.is_some();
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.seat.0);

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
        FocusPolicy::Block,
    ));

    for (index, (winner, winner_timing)) in result.winners.iter().zip(&timing.winners).enumerate() {
        let end = timing
            .winners
            .get(index + 1)
            .map_or(score_start, |next| next.outcome_delay);
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
            FocusPolicy::Block,
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
        commands.entity(spacer).insert(FocusPolicy::Pass);
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
        commands.entity(rule).insert(FocusPolicy::Pass);
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
                .insert((settlement_row_texture(assets), FocusPolicy::Pass));
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
            FocusPolicy::Block,
        ));
    }

    if !fan_summary_continued && let Some(pause_at) = mahjong_fan_pause_at(result) {
        render_settlement_continue(
            commands,
            content,
            assets,
            animation.elapsed,
            pause_at,
            score_start,
            "点击任意位置继续",
            Some(UiAction::Mahjong(MahjongUiAction::ContinueFanSummary)),
        );
    }

    let score_stage = spawn_node(
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
    commands.entity(score_stage).insert((
        MahjongScoreStage {
            start: score_start,
            end: if final_summary_open {
                continue_start
            } else {
                f32::INFINITY
            },
        },
        GlobalZIndex(1200),
        if animation.elapsed >= score_start && !final_summary_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));

    // 复刻牌桌中央的方位牌，让遮罩只留下这一块亮区。四面数值从结算前
    // 的分数开始，沿各自座位的朝向接收增减，再滚到结算后的分数。
    let layout_scores = std::array::from_fn(|slot| {
        let to = result.match_scores[slot];
        let from = to.saturating_sub(result.deltas[slot]);
        if from.to_string().len() > to.to_string().len() {
            from
        } else {
            to
        }
    });
    let score_texts = render_round_status_with_scores(
        commands,
        score_stage,
        game,
        own_seat,
        &layout_scores,
        assets,
        game_assets,
    );
    for (relative, score_text) in score_texts.into_iter().enumerate() {
        let Some(player) = game
            .players
            .iter()
            .find(|player| usize::from((player.seat.0 + 4 - own_seat) % 4) == relative)
        else {
            continue;
        };
        let slot = usize::from(player.id.0);
        let total = result.match_scores[slot];
        let delta = result.deltas[slot];
        let from = total.saturating_sub(delta);
        commands.entity(score_text).insert((
            Text(score_at(animation.elapsed, from, total, roll_start).to_string()),
            MahjongScoreValue {
                from,
                to: total,
                start: roll_start,
            },
            TextColor(TEXT),
        ));
        let (left, top, origin, rotation) = match relative {
            0 => (637.0, 359.0, Vec2::new(0.0, 38.0), 0.0),
            1 => (
                692.0,
                317.0,
                Vec2::new(38.0, 0.0),
                -std::f32::consts::FRAC_PI_2,
            ),
            2 => (637.0, 281.0, Vec2::new(0.0, -38.0), std::f32::consts::PI),
            _ => (
                575.0,
                317.0,
                Vec2::new(-38.0, 0.0),
                std::f32::consts::FRAC_PI_2,
            ),
        };
        let color = if delta > 0 {
            READY
        } else if delta < 0 {
            DANGER
        } else {
            MUTED
        };
        let flight_progress = delta_progress(animation.elapsed, flight_start);
        let flying = add_text(
            commands,
            score_stage,
            format!("{delta:+}"),
            18.0,
            color.with_alpha(1.0 - flight_progress.powi(3)),
            assets,
        );
        commands.entity(flying).insert((
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                ..default()
            },
            MahjongFlyingDelta {
                appear_at: score_start + DELTA_APPEAR_DELAY,
                start: flight_start,
                color,
                origin,
                rotation,
            },
            delta_transform(flight_progress, origin, rotation),
            if animation.elapsed >= score_start + DELTA_APPEAR_DELAY && flight_progress < 1.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            FocusPolicy::Pass,
        ));
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
            continue_start,
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
