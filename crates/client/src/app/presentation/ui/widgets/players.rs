//! 玩家框、头像、准备标记和房主标记。

use super::super::{ACCENT, BORDER, ButtonTint, HEADER_BG, MUTED, PANEL_ALT, READY, TEXT};
use super::{add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    AutoPlayAntennaLight, AutoPlayAntennaLightPart, AutoPlayRobotIndicator,
    InteractionCooldownMask, InteractionMenuPanel, NavigationUiAction, PlayerProfilePage, SeatSide,
    SocialUiAction, UiAction, reference_level,
};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{PlayerGameProfiles, PlayerId, PlayerInteractionKind};

pub(crate) fn decorate_player_panel(
    commands: &mut Commands,
    panel: Entity,
    assets: &UiAssets,
    scale: f32,
) {
    let image = if scale < 1.0 {
        assets.controls.player_panel_compact.clone()
    } else {
        assets.controls.player_panel_wide.clone()
    };
    let texture = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            ImageNode::new(image)
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgba(1.0, 1.0, 1.0, 0.82)),
            ZIndex(-1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(panel).add_child(texture);
}

/// 人物框靠牌桌内侧的大号数值区域。七鬼五二三用于本局得分，德州用于剩余筹码。
pub(crate) fn add_player_panel_primary_value(
    commands: &mut Commands,
    badge: Entity,
    side: SeatSide,
    value: impl ToString,
    assets: &UiAssets,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        top: px(0),
        bottom: px(0),
        width: px(66),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    match side {
        SeatSide::Left | SeatSide::Top => node.right = px(2),
        SeatSide::Right => node.left = px(2),
    }
    let area = spawn_node(commands, badge, node, None);
    commands.entity(area).insert((ZIndex(2), FocusPolicy::Pass));
    let text = add_text(commands, area, value.to_string(), 28.0, ACCENT, assets);
    commands.entity(text).insert(TextShadow {
        offset: Vec2::new(1.5, 2.0),
        color: Color::BLACK.with_alpha(0.82),
    });
    text
}

pub(crate) fn add_avatar(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> Entity {
    let mut entity = commands.spawn(Node {
        width: px(size),
        height: px(size),
        min_width: px(size),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(percent(50)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
    entity.insert(BorderColor::all(BORDER));
    if let Some(image) = image {
        entity.insert(ImageNode::new(image.clone()));
    } else {
        entity.insert(BackgroundColor(avatar_color(name)));
    }
    let entity = entity.id();
    commands.entity(parent).add_child(entity);
    if image.is_none() {
        let initial = name.chars().next().unwrap_or('玩').to_string();
        add_text(commands, entity, initial, size * 0.42, Color::WHITE, assets);
    }
    entity
}

/// 结算窗口中的下一局准备状态。保持德州扑克原有的绿色头像环和右下角对钩，
/// 让所有游戏都能在玩家点击“再来一局”后直接看到彼此的准备进度。
pub(crate) fn add_ready_avatar(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    image: Option<&Handle<Image>>,
    avatar_size: f32,
    ready: bool,
    assets: &UiAssets,
) -> Entity {
    let frame_size = avatar_size + 6.0;
    let frame = spawn_node(
        commands,
        parent,
        Node {
            width: px(frame_size),
            height: px(frame_size),
            min_width: px(frame_size),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        None,
    );
    commands.entity(frame).insert(BorderColor::all(if ready {
        READY
    } else {
        MUTED.with_alpha(0.42)
    }));
    add_avatar(commands, frame, name, image, avatar_size, assets);
    if ready {
        let check_size = (avatar_size * 0.47).max(13.0);
        let check = spawn_node(
            commands,
            frame,
            Node {
                position_type: PositionType::Absolute,
                right: px(-3),
                bottom: px(-2),
                width: px(check_size),
                height: px(check_size),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(READY),
        );
        add_text(
            commands,
            check,
            "✓",
            check_size * 2.0 / 3.0,
            Color::WHITE,
            assets,
        );
    }
    frame
}

pub(crate) fn add_host_crown(commands: &mut Commands, avatar: Entity, assets: &UiAssets) -> Entity {
    let crown = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(-9),
                top: px(-11),
                width: px(19),
                height: px(17),
                ..default()
            },
            ImageNode::new(assets.controls.host_crown.clone()),
            UiTransform::from_rotation(Rot2::radians(-1.08)),
            ZIndex(40),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(avatar).add_child(crown);
    crown
}

pub(crate) fn avatar_color(name: &str) -> Color {
    const COLORS: [Color; 6] = [
        Color::srgb(0.20, 0.48, 0.76),
        Color::srgb(0.65, 0.29, 0.68),
        Color::srgb(0.82, 0.35, 0.25),
        Color::srgb(0.18, 0.62, 0.48),
        Color::srgb(0.76, 0.53, 0.16),
        Color::srgb(0.35, 0.42, 0.72),
    ];
    let hash = name
        .bytes()
        .fold(0_usize, |hash, byte| hash.wrapping_mul(31) + byte as usize);
    COLORS[hash % COLORS.len()]
}

pub(crate) fn add_auto_play_robot_indicator(
    commands: &mut Commands,
    badge: Entity,
    player: PlayerId,
    side: SeatSide,
    assets: &UiAssets,
) {
    let mut node = Node {
        position_type: PositionType::Absolute,
        top: px(19),
        width: px(34),
        height: px(34),
        ..default()
    };
    match side {
        SeatSide::Left | SeatSide::Top => node.right = px(-38),
        SeatSide::Right => node.left = px(-38),
    }
    let indicator = commands
        .spawn((
            node,
            ImageNode::new(assets.controls.robot_icon.clone()),
            UiTransform::IDENTITY,
            ZIndex(30),
            FocusPolicy::Pass,
            AutoPlayRobotIndicator,
        ))
        .id();
    commands.entity(badge).add_child(indicator);
    add_auto_play_antenna_lights(commands, indicator, player);
}

fn add_auto_play_antenna_lights(commands: &mut Commands, indicator: Entity, player: PlayerId) {
    let glow = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(10.5),
                top: px(-3),
                width: px(13),
                height: px(13),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.32, 1.0, 0.58, 0.0)),
            UiTransform::IDENTITY,
            ZIndex(2),
            FocusPolicy::Pass,
            AutoPlayAntennaLight {
                player,
                part: AutoPlayAntennaLightPart::Glow,
            },
        ))
        .id();
    commands.entity(indicator).add_child(glow);

    for (left, top, rotation) in [(16.0, -8.0, 0.0), (7.5, -4.5, -0.82), (24.5, -4.5, 0.82)] {
        let ray = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(2),
                    height: px(6),
                    border_radius: BorderRadius::all(px(1)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.46, 1.0, 0.68, 0.0)),
                UiTransform {
                    rotation: Rot2::radians(rotation),
                    ..UiTransform::IDENTITY
                },
                ZIndex(3),
                FocusPolicy::Pass,
                AutoPlayAntennaLight {
                    player,
                    part: AutoPlayAntennaLightPart::Ray,
                },
            ))
            .id();
        commands.entity(indicator).add_child(ray);
    }
}

pub(crate) struct PlayerMenuProfile<'a> {
    pub name: &'a str,
    pub avatar: Option<&'a Handle<Image>>,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: &'a PlayerGameProfiles,
}

pub(crate) fn add_interaction_menu(
    commands: &mut Commands,
    parent: Entity,
    target: PlayerId,
    side: SeatSide,
    profile: PlayerMenuProfile<'_>,
    assets: &UiAssets,
) -> Entity {
    let PlayerMenuProfile {
        name: player_name,
        avatar,
        reference_points,
        completed_games,
        game_profiles,
    } = profile;
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        min_height: px(118),
        padding: UiRect::all(px(6)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(4),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    };
    position_opponent_popup(&mut node, side);
    let menu = spawn_node(commands, parent, node, Some(HEADER_BG.with_alpha(0.98)));
    commands.entity(menu).insert((
        InteractionMenuPanel(target),
        BorderColor::all(ACCENT.with_alpha(0.72)),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));
    let profile = spawn_node(
        commands,
        menu,
        Node {
            width: percent(100),
            height: px(38),
            min_height: px(38),
            padding: UiRect::axes(px(5), px(3)),
            align_items: AlignItems::Center,
            column_gap: px(7),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(PANEL_ALT.with_alpha(0.82)),
    );
    add_avatar(commands, profile, player_name, avatar, 30.0, assets);
    let identity = spawn_node(
        commands,
        profile,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(1),
            ..default()
        },
        None,
    );
    add_text(commands, identity, player_name, 13.0, TEXT, assets);
    add_text(
        commands,
        identity,
        format!(
            "等级:{}  分数:{}  对局:{}",
            reference_level(reference_points),
            reference_points,
            completed_games
        ),
        9.5,
        MUTED,
        assets,
    );
    let normal = Color::srgb(0.34, 0.50, 0.62);
    let profile_button = commands
        .spawn((
            Button,
            UiAction::Navigation(NavigationUiAction::OpenPlayerProfile(Box::new(
                PlayerProfilePage {
                    name: player_name.to_owned(),
                    avatar: avatar.cloned(),
                    reference_points,
                    completed_games,
                    game_profiles: game_profiles.clone(),
                },
            ))),
            ButtonTint {
                normal,
                hovered: Color::srgb(0.50, 0.66, 0.78),
                pressed: Color::srgb(0.24, 0.38, 0.50),
            },
            Node {
                width: px(66),
                min_width: px(66),
                height: px(28),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(profile).add_child(profile_button);
    add_text(
        commands,
        profile_button,
        "完整资料",
        11.0,
        Color::WHITE,
        assets,
    );
    let actions = spawn_node(
        commands,
        menu,
        Node {
            width: percent(100),
            height: px(62),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    for (kind, label) in [
        (PlayerInteractionKind::Flower, "鲜花"),
        (PlayerInteractionKind::Egg, "鸡蛋"),
        (PlayerInteractionKind::Wine, "酒杯"),
        (PlayerInteractionKind::Shoe, "拖鞋"),
    ] {
        let normal = Color::srgb(0.18, 0.42, 0.34);
        let button = commands
            .spawn((
                Button,
                UiAction::Social(SocialUiAction::SendInteraction { target, kind }),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.27, 0.58, 0.46),
                    pressed: Color::srgb(0.12, 0.30, 0.24),
                },
                Node {
                    width: px(72),
                    height: px(62),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(1),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(assets.controls.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id();
        commands.entity(actions).add_child(button);
        let icon = commands
            .spawn((
                Node {
                    width: px(38),
                    height: px(38),
                    ..default()
                },
                ImageNode::new(
                    assets
                        .social
                        .interaction_images
                        .get(&(kind, false))
                        .expect("every interaction has a flight image")
                        .clone(),
                ),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(icon);
        add_text(commands, button, label, 11.0, TEXT, assets);
        if matches!(
            kind,
            PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe
        ) {
            let mask = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    ImageNode::new(
                        assets
                            .social
                            .interaction_cooldown_masks
                            .last()
                            .cloned()
                            .unwrap_or_default(),
                    ),
                    InteractionCooldownMask {
                        player: target,
                        kind,
                    },
                    Visibility::Hidden,
                    ZIndex(10),
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(button).add_child(mask);
        }
    }
    menu
}

pub(crate) fn position_opponent_popup(node: &mut Node, side: SeatSide) {
    match side {
        SeatSide::Left => {
            node.left = px(0);
            node.top = px(76);
        }
        SeatSide::Top => {
            node.left = px(-81);
            node.top = px(76);
        }
        SeatSide::Right => {
            node.right = px(0);
            node.top = px(76);
        }
    }
}
