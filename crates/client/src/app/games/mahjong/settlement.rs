//! 国标麻将结算：逐位报番，然后在牌桌座位上结算分数。

use super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongUiAction, MahjongWinTileSizes,
    mahjong_settlement_timing, mahjong_win_reveal_duration, render_mahjong_win_tile_row,
    render_round_status_with_scores,
};
use crate::app::presentation::{
    ACCENT, DANGER, GameSummaryAnimation, MUTED, PanelSkin, READY, SUMMARY_ACTIONS_EXTRA_DELAY,
    SUMMARY_ROW_INTERVAL, SUMMARY_ROW_START_DELAY, SummaryDescriptor, TEXT, add_avatar, add_text,
    decorate_panel_skin, spawn_node, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{
    CozyButtonVariant, LobbyUiAction, UiAction, add_cozy_button, add_cozy_button_variant,
    add_cozy_disabled_button,
};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::MahjongMatchLength;
use leocard_protocol::{MahjongHandResultView, MahjongPhaseView, MahjongSnapshot};

const PAGE_FADE_DURATION: f32 = 0.28;
const SCORE_FADE_DURATION: f32 = 0.36;
const DELTA_FLIGHT_DURATION: f32 = 0.62;
const SCORE_ROLL_DURATION: f32 = 0.80;
const DELTA_APPEAR_DELAY: f32 = 0.14;
const DELTA_HOLD_DURATION: f32 = 0.75;
const SCORE_END_HOLD_DURATION: f32 = 0.65;

pub(crate) fn mahjong_fan_pause_at(result: &MahjongHandResultView) -> Option<f32> {
    (!result.winners.is_empty()).then(|| {
        mahjong_settlement_timing(result).score_rows_delay + 0.12 - PAGE_FADE_DURATION - 0.04
    })
}

fn score_timing(score_start: f32) -> (f32, f32, f32) {
    let flight_start = score_start + DELTA_APPEAR_DELAY + DELTA_HOLD_DURATION;
    let roll_start = flight_start + DELTA_FLIGHT_DURATION * 0.72;
    let continue_start = roll_start + SCORE_ROLL_DURATION + SCORE_END_HOLD_DURATION;
    (flight_start, roll_start, continue_start)
}

fn score_at(elapsed: f32, from: i32, to: i32, start: f32) -> i32 {
    let progress = ((elapsed - start) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    (from as f32 + (to as f32 - from as f32) * eased).round() as i32
}

fn delta_progress(elapsed: f32, start: f32) -> f32 {
    ((elapsed - start) / DELTA_FLIGHT_DURATION).clamp(0.0, 1.0)
}

fn delta_transform(progress: f32, origin: Vec2, rotation: f32) -> UiTransform {
    let eased = progress * progress * (3.0 - 2.0 * progress);
    UiTransform {
        translation: Val2::px(origin.x * (1.0 - eased), origin.y * (1.0 - eased)),
        scale: Vec2::splat(1.0 - 0.38 * eased),
        rotation: Rot2::radians(rotation),
    }
}

fn settlement_row_texture(assets: &UiAssets) -> ImageNode {
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

#[derive(Component)]
pub(super) struct MahjongFanPage {
    start: f32,
    end: f32,
}

#[derive(Component)]
pub(super) struct MahjongFanPageCurtain {
    end: f32,
}

#[derive(Component)]
pub(super) struct MahjongFanEntry {
    delay: f32,
    end: f32,
}

#[derive(Component)]
pub(super) struct MahjongScoreStage {
    start: f32,
    end: f32,
}

#[derive(Component)]
pub(super) struct MahjongScoreShade {
    start: f32,
}

#[derive(Component)]
pub(super) struct MahjongScoreValue {
    from: i32,
    to: i32,
    start: f32,
}

#[derive(Component)]
pub(super) struct MahjongFlyingDelta {
    appear_at: f32,
    start: f32,
    color: Color,
    origin: Vec2,
    rotation: f32,
}

#[derive(Component)]
pub(super) struct MahjongFinalRow {
    opened_at: f32,
    index: usize,
}

#[derive(Component)]
pub(super) struct MahjongFinalActions {
    opened_at: f32,
    row_count: usize,
}

pub(super) fn animate_mahjong_final_summary(
    time: Res<Time>,
    mut rows: Query<
        (
            &MahjongFinalRow,
            &mut Visibility,
            &mut UiTransform,
            &mut ImageNode,
        ),
        Without<MahjongFinalActions>,
    >,
    mut actions: Query<(&MahjongFinalActions, &mut Visibility), Without<MahjongFinalRow>>,
) {
    let now = time.elapsed_secs();
    for (row, mut visible, mut transform, mut image) in &mut rows {
        let progress = summary_row_progress(
            now - row.opened_at,
            SUMMARY_ROW_START_DELAY + row.index as f32 * SUMMARY_ROW_INTERVAL,
        );
        *visible = if progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
        image.color = Color::WHITE.with_alpha(progress);
    }
    for (state, mut visible) in &mut actions {
        let delay = SUMMARY_ROW_START_DELAY
            + state.row_count as f32 * SUMMARY_ROW_INTERVAL
            + SUMMARY_ACTIONS_EXTRA_DELAY;
        *visible = if now - state.opened_at >= delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[expect(
    clippy::type_complexity,
    reason = "ParamSet keeps simultaneous page and score-stage animation queries disjoint"
)]
pub(super) fn animate_mahjong_settlement(
    animation: Res<GameSummaryAnimation>,
    mut visibility: ParamSet<(
        Query<(&MahjongFanPage, &mut Visibility)>,
        Query<(&MahjongFanEntry, &mut Visibility, &mut UiTransform)>,
        Query<(&MahjongScoreStage, &mut Visibility)>,
        Query<(
            &MahjongFlyingDelta,
            &mut Visibility,
            &mut UiTransform,
            &mut TextColor,
        )>,
    )>,
    mut backgrounds: ParamSet<(
        Query<(&MahjongFanPageCurtain, &mut BackgroundColor)>,
        Query<(&MahjongScoreShade, &mut BackgroundColor)>,
    )>,
    mut scores: Query<(&MahjongScoreValue, &mut Text)>,
) {
    if !animation.is_changed() {
        return;
    }
    let elapsed = animation.elapsed;
    for (page, mut visibility) in &mut visibility.p0() {
        *visibility = if elapsed >= page.start && elapsed < page.end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (curtain, mut color) in &mut backgrounds.p0() {
        let opacity =
            ((elapsed - (curtain.end - PAGE_FADE_DURATION)) / PAGE_FADE_DURATION).clamp(0.0, 1.0);
        color.0 = Color::srgb(0.035, 0.039, 0.055).with_alpha(opacity);
    }
    for (entry, mut visibility, mut transform) in &mut visibility.p1() {
        let progress = summary_row_progress(elapsed, entry.delay);
        *visibility = if progress > 0.0 && elapsed < entry.end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
    }
    for (stage, mut visibility) in &mut visibility.p2() {
        *visibility = if elapsed >= stage.start && elapsed < stage.end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (shade, mut color) in &mut backgrounds.p1() {
        let alpha = ((elapsed - shade.start) / SCORE_FADE_DURATION).clamp(0.0, 1.0);
        color.0 = Color::BLACK.with_alpha(0.80 * alpha);
    }
    for (score, mut text) in &mut scores {
        text.0 = score_at(elapsed, score.from, score.to, score.start).to_string();
    }
    for (delta, mut visible, mut transform, mut color) in &mut visibility.p3() {
        let progress = delta_progress(elapsed, delta.start);
        *visible = if elapsed >= delta.appear_at && progress < 1.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *transform = delta_transform(progress, delta.origin, delta.rotation);
        color.0 = delta.color.with_alpha(1.0 - progress.powi(3));
    }
}

pub(crate) fn mahjong_summary_descriptor(game: &MahjongSnapshot) -> Option<SummaryDescriptor> {
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return None;
    };
    let timing = mahjong_settlement_timing(result);
    let score_start = timing.score_rows_delay + if result.winners.is_empty() { 0.0 } else { 0.12 };
    let (_, _, continue_delay) = score_timing(score_start);
    let entry_count =
        ((continue_delay + 0.05 - SUMMARY_ROW_START_DELAY - SUMMARY_ACTIONS_EXTRA_DELAY)
            / SUMMARY_ROW_INTERVAL)
            .ceil() as usize;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        texas_hand_number: None,
        settlement_index: Some(u32::from(result.sequence_index)),
        entry_count: entry_count.max(game.players.len()),
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
    pub fan_summary_continued: bool,
    pub final_summary_opened_at: Option<f32>,
}

pub(super) fn render_mahjong_settlement(
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
        let summary = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(270),
                right: px(270),
                top: px(114),
                padding: UiRect::axes(px(24), px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands
            .entity(summary)
            .insert((GlobalZIndex(1201), FocusPolicy::Pass));
        decorate_panel_skin(commands, summary, PanelSkin::Window, assets);
        add_text(commands, summary, "整局结算", 26.0, ACCENT, assets);
        spawn_node(
            commands,
            summary,
            Node {
                width: percent(100),
                height: px(2),
                ..default()
            },
            Some(ACCENT.with_alpha(0.55)),
        );
        let mut ranked = game.players.iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            result.match_scores[right.id.0 as usize]
                .cmp(&result.match_scores[left.id.0 as usize])
                .then_with(|| left.seat.0.cmp(&right.seat.0))
        });
        for (place, player) in ranked.iter().enumerate() {
            let row = spawn_node(
                commands,
                summary,
                Node {
                    width: percent(100),
                    height: px(61),
                    padding: UiRect::axes(px(13), px(7)),
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    ..default()
                },
                None,
            );
            commands.entity(row).insert((
                settlement_row_texture(assets),
                MahjongFinalRow {
                    opened_at,
                    index: place,
                },
                UiTransform::default(),
                Visibility::Hidden,
                FocusPolicy::Pass,
            ));
            add_text(
                commands,
                row,
                format!("{}", place + 1),
                19.0,
                ACCENT,
                assets,
            );
            let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
            add_avatar(commands, row, &player.name, avatar, 39.0, assets);
            add_text(commands, row, &player.name, 17.0, TEXT, assets);
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
                format!("总分 {:+}", result.match_scores[usize::from(player.id.0)]),
                17.0,
                ACCENT,
                assets,
            );
            if let Some(change) = result
                .reference_changes
                .iter()
                .find(|change| change.player == player.id)
            {
                add_text(
                    commands,
                    row,
                    format!("积分 {:+}", change.delta),
                    17.0,
                    if change.delta >= 0 { READY } else { DANGER },
                    assets,
                );
            }
        }
        let actions = spawn_node(
            commands,
            summary,
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
            MahjongFinalActions {
                opened_at,
                row_count: ranked.len(),
            },
            Visibility::Hidden,
        ));
        if game.rules.match_length == MahjongMatchLength::SingleHand {
            add_cozy_button_variant(
                commands,
                actions,
                "退出游戏",
                UiAction::Lobby(LobbyUiAction::LeaveRoom),
                assets,
                px(150),
                48.0,
                CozyButtonVariant::Danger,
            );
            if game.you == game.host {
                add_cozy_button(
                    commands,
                    actions,
                    "返回房间",
                    UiAction::Lobby(LobbyUiAction::ReturnToLobby),
                    assets,
                    px(150),
                    48.0,
                );
            }
            let ready = game
                .players
                .iter()
                .find(|player| player.id == game.you)
                .is_some_and(|player| player.ready);
            if ready {
                add_cozy_disabled_button(commands, actions, "已准备", assets, px(150), 48.0);
            } else {
                add_cozy_button_variant(
                    commands,
                    actions,
                    "再来一局",
                    UiAction::Lobby(LobbyUiAction::PlayAgain),
                    assets,
                    px(150),
                    48.0,
                    CozyButtonVariant::Cool,
                );
            }
        } else {
            add_cozy_button_variant(
                commands,
                actions,
                "返回大厅",
                UiAction::Lobby(LobbyUiAction::ReturnToLobby),
                assets,
                percent(100),
                48.0,
                CozyButtonVariant::Primary,
            );
        }
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

#[expect(
    clippy::too_many_arguments,
    reason = "the prompt has explicit timing and action"
)]
fn render_settlement_continue(
    commands: &mut Commands,
    content: Entity,
    assets: &UiAssets,
    elapsed: f32,
    start: f32,
    end: f32,
    label: &str,
    action: Option<UiAction>,
) {
    let area = spawn_node(
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
        Some(Color::BLACK.with_alpha(0.001)),
    );
    commands.entity(area).insert((
        MahjongScoreStage { start, end },
        GlobalZIndex(1300),
        FocusPolicy::Block,
        if elapsed >= start && elapsed < end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    if let Some(action) = action {
        commands.entity(area).insert((Button, action));
    }
    let prompt = add_text(commands, area, label, 18.0, TEXT, assets);
    commands.entity(prompt).insert((
        Node {
            position_type: PositionType::Absolute,
            right: px(30),
            bottom: px(22),
            ..default()
        },
        TextShadow {
            offset: Vec2::new(1.0, 2.0),
            color: Color::BLACK.with_alpha(0.9),
        },
        FocusPolicy::Pass,
    ));
}
