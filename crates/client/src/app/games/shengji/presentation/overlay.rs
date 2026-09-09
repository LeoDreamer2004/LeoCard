use super::{
    ShengjiAssets, ShengjiBottomFlipVisual, ShengjiBottomFlipVisualKind, ShengjiPowerOutageVisual,
    ShengjiPowerOutageVisualKind, ShengjiPresentationDivider, ShengjiPresentationKind,
    ShengjiPresentationPacket, ShengjiPresentationRoot, ShengjiPresentationState,
    ShengjiPresentationText, ShengjiPresentationVeil, ShengjiTrumpKillVisual,
    ShengjiTrumpKillVisualKind, presentation_color, presentation_text, rank_label,
    shengji_player_panel_anchor, shengji_power_outage_anchors, shengji_presentation_routes,
};
use crate::app::presentation::{ACCENT, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::ShengjiSnapshot;

pub(crate) fn add_shengji_presentation_overlay(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    state: &ShengjiPresentationState,
    assets: &UiAssets,
    game_assets: &ShengjiAssets,
) {
    let Some(active) = state.active.as_ref() else {
        return;
    };
    let is_play = matches!(
        &active.kind,
        ShengjiPresentationKind::Play { .. } | ShengjiPresentationKind::TrumpKill { .. }
    );
    let is_trump_kill = matches!(&active.kind, ShengjiPresentationKind::TrumpKill { .. });
    let bottom_flip_matches = match &active.kind {
        ShengjiPresentationKind::BottomFlip { matches, .. } => Some(matches.as_slice()),
        _ => None,
    };
    let is_bottom_flip = bottom_flip_matches.is_some();
    let power_outage_anchors = shengji_power_outage_anchors(&active.kind, game);
    let is_power_outage = power_outage_anchors.is_some();
    let routes = shengji_presentation_routes(&active.kind, game);
    let is_routed = !routes.is_empty();
    let mut root_node = Node {
        position_type: PositionType::Absolute,
        height: px(132),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    };
    let mut base_translation = Vec2::ZERO;
    let anchored_player = match &active.kind {
        ShengjiPresentationKind::Play { player, .. }
        | ShengjiPresentationKind::TrumpKill { player, .. } => Some(*player),
        _ => None,
    };
    if let Some(player) = anchored_player {
        root_node.width = px(if is_trump_kill { 190 } else { 180 });
        root_node.height = px(if is_trump_kill { 92 } else { 64 });
        let half_width = if is_trump_kill { 95.0 } else { 90.0 };
        if is_trump_kill {
            root_node.justify_content = JustifyContent::FlexEnd;
            root_node.row_gap = px(2);
            root_node.padding = UiRect::bottom(px(3));
        }
        let own_seat = game
            .players
            .iter()
            .find(|candidate| candidate.id == game.you)
            .map_or(0, |candidate| candidate.seat.0);
        let player_seat = game
            .players
            .iter()
            .find(|candidate| candidate.id == player)
            .map_or(own_seat, |candidate| candidate.seat.0);
        match (player_seat + 4 - own_seat) % 4 {
            0 => {
                root_node.left = percent(50);
                root_node.bottom = px(116);
                base_translation.x = -half_width;
            }
            1 => {
                root_node.left = px(260);
                root_node.top = percent(50);
                base_translation.y = -92.0;
            }
            2 => {
                root_node.left = percent(50);
                root_node.top = px(202);
                base_translation.x = -half_width;
            }
            3 => {
                root_node.right = px(260);
                root_node.top = percent(50);
                base_translation.y = -92.0;
            }
            _ => unreachable!(),
        }
    } else if is_routed || is_power_outage || is_bottom_flip {
        // 交牌和换庄演出覆盖整张桌面，移动方向才能和真实座位一致。
        root_node.left = px(0);
        root_node.top = px(0);
        root_node.width = percent(100);
        root_node.height = percent(100);
        root_node.row_gap = px(5);
    } else {
        // 规则事件始终以牌桌的几何中心为锚点。
        root_node.left = percent(25);
        root_node.right = percent(25);
        root_node.top = percent(50);
        base_translation.y = -66.0;
    }
    let root = spawn_node(commands, table, root_node, None);
    commands.entity(root).insert((
        ShengjiPresentationRoot { base_translation },
        GlobalZIndex(if is_bottom_flip { 920 } else { 520 }),
        FocusPolicy::Pass,
    ));
    let veil = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(-500),
            right: px(-500),
            top: px(-420),
            bottom: px(-420),
            ..default()
        },
        Some(Color::NONE),
    );
    commands
        .entity(veil)
        .insert((ShengjiPresentationVeil, FocusPolicy::Pass));

    if let Some(matches) = bottom_flip_matches {
        const CENTER: Vec2 = Vec2::new(50.0, 30.5);
        const SCAN_SPARK_COUNT: usize = 6;
        const REPLY_SPARK_COUNT: usize = 5;
        for (player_index, matched) in matches.iter().enumerate() {
            let Some(target) = shengji_player_panel_anchor(game, matched.player) else {
                continue;
            };
            for spark_index in 0..SCAN_SPARK_COUNT {
                let spark = spawn_node(
                    commands,
                    root,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(CENTER.x),
                        top: percent(CENTER.y),
                        width: px(7),
                        height: px(7),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    Some(Color::NONE),
                );
                commands.entity(spark).insert((
                    ShengjiBottomFlipVisual {
                        kind: ShengjiBottomFlipVisualKind::ScanSpark {
                            player_index,
                            player_count: matches.len(),
                            spark_index,
                            spark_count: SCAN_SPARK_COUNT,
                            start: CENTER,
                            end: target,
                        },
                    },
                    UiTransform::from_translation(Val2::px(-3.5, -3.5)),
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                ));
            }

            let reply_end = CENTER.lerp(target, 0.30);
            for spark_index in 0..REPLY_SPARK_COUNT {
                let spark = spawn_node(
                    commands,
                    root,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(target.x),
                        top: percent(target.y),
                        width: px(8),
                        height: px(8),
                        border_radius: BorderRadius::all(px(2)),
                        ..default()
                    },
                    Some(Color::NONE),
                );
                commands.entity(spark).insert((
                    ShengjiBottomFlipVisual {
                        kind: ShengjiBottomFlipVisualKind::ReplySpark {
                            player_index,
                            player_count: matches.len(),
                            spark_index,
                            spark_count: REPLY_SPARK_COUNT,
                            start: target,
                            end: reply_end,
                        },
                    },
                    UiTransform::from_translation(Val2::px(-4.0, -4.0)),
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                ));
            }
        }
    }

    if let Some((start, end)) = power_outage_anchors {
        const SPARK_COUNT: usize = 7;
        for index in 0..SPARK_COUNT {
            let spark = spawn_node(
                commands,
                root,
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(start.x),
                    top: percent(start.y),
                    width: px(8),
                    height: px(8),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                Some(Color::NONE),
            );
            commands.entity(spark).insert((
                ShengjiPowerOutageVisual {
                    kind: ShengjiPowerOutageVisualKind::Spark {
                        index,
                        count: SPARK_COUNT,
                        start,
                        end,
                    },
                },
                UiTransform::from_translation(Val2::px(-4.0, -4.0)),
                Visibility::Hidden,
                FocusPolicy::Pass,
            ));
        }

        let ring = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(end.x),
                top: percent(end.y),
                width: px(50),
                height: px(50),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(ring).insert((
            ShengjiPowerOutageVisual {
                kind: ShengjiPowerOutageVisualKind::TargetRing { target: end },
            },
            BorderColor::all(Color::NONE),
            UiTransform::from_translation(Val2::px(-25.0, -25.0)),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));

        let level_target = end.lerp(Vec2::new(50.0, 50.0), 0.24);
        let level = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(level_target.x),
                top: percent(level_target.y),
                width: px(116),
                height: px(34),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(level).insert((
            ShengjiPowerOutageVisual {
                kind: ShengjiPowerOutageVisualKind::LevelFlash {
                    target: level_target,
                },
            },
            BorderColor::all(Color::NONE),
            UiTransform::from_translation(Val2::px(-58.0, -17.0)),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));
        let rank = match &active.kind {
            ShengjiPresentationKind::PowerOutage { level, .. } => rank_label(*level),
            _ => unreachable!(),
        };
        add_text(
            commands,
            level,
            format!("新庄 · 打{rank}"),
            15.0,
            ACCENT,
            assets,
        );
    }

    if is_trump_kill {
        let target = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(108),
                top: px(1),
                width: px(58),
                height: px(58),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(target).insert((
            ShengjiTrumpKillVisual {
                kind: ShengjiTrumpKillVisualKind::Target,
            },
            ImageNode::new(game_assets.target.clone()).with_color(Color::NONE),
            UiTransform::IDENTITY,
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));

        let dart = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(-42),
                top: px(10),
                width: px(52),
                height: px(52),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(dart).insert((
            ShengjiTrumpKillVisual {
                kind: ShengjiTrumpKillVisualKind::Dart,
            },
            ImageNode::new(game_assets.dart.clone()).with_color(Color::NONE),
            UiTransform {
                rotation: Rot2::radians((-45.0_f32).to_radians()),
                ..default()
            },
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));

        let ring = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(110),
                top: px(3),
                width: px(56),
                height: px(56),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(ring).insert((
            ShengjiTrumpKillVisual {
                kind: ShengjiTrumpKillVisualKind::ImpactRing,
            },
            BorderColor::all(Color::NONE),
            UiTransform::IDENTITY,
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));
    }

    for (route_index, route) in routes.iter().enumerate() {
        for index in 0..route.packet_count {
            let packet = spawn_node(
                commands,
                root,
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(route.start.x),
                    top: percent(route.start.y),
                    width: px(34),
                    height: px(48),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                None,
            );
            commands.entity(packet).insert((
                ShengjiPresentationPacket {
                    index,
                    count: route.packet_count,
                    route_index,
                    start: route.start,
                    end: route.end,
                },
                ImageNode::new(assets.playing_cards.card_back.clone())
                    .with_mode(NodeImageMode::Stretch),
                UiTransform::from_translation(Val2::px(-17.0, -24.0)),
                FocusPolicy::Pass,
            ));
        }
    }

    // 扣底的标题、底牌和匹配牌已经在中央面板内完整展示，
    // 此处只叠加玩家定向的交互动画，避免再显示一套重复文字。
    if is_bottom_flip {
        return;
    }

    let (title, subtitle) = presentation_text(&active.kind, game);
    let title_color = presentation_color(&active.kind);
    let title = add_text(
        commands,
        root,
        title,
        if is_play {
            if is_trump_kill { 17.0 } else { 19.0 }
        } else if is_routed {
            22.0
        } else {
            25.0
        },
        title_color,
        assets,
    );
    commands.entity(title).insert(ShengjiPresentationText {
        base_color: title_color,
    });
    let divider = spawn_node(
        commands,
        root,
        Node {
            width: px(if is_trump_kill {
                42.0
            } else if is_play {
                54.0
            } else {
                88.0
            }),
            height: px(2),
            ..default()
        },
        Some(title_color.with_alpha(0.72)),
    );
    commands
        .entity(divider)
        .insert((ShengjiPresentationDivider, FocusPolicy::Pass));
    if !subtitle.is_empty() {
        let subtitle = add_text(commands, root, subtitle, 15.0, TEXT, assets);
        commands
            .entity(subtitle)
            .insert(ShengjiPresentationText { base_color: TEXT });
    }
}
