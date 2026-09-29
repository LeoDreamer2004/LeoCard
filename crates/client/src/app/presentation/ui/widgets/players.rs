//! 玩家框、头像、准备标记和房主标记。

use super::super::{BORDER, MUTED, READY, TEXT};
use super::{add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    HomeHighlightKind, InteractionCooldownMask, InteractionMenuPanel, NavigationUiAction,
    PlayerProfilePage, SeatSide, SocialUiAction, UiAction, add_cozy_button, add_cozy_panel,
    reference_level,
};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_protocol::{PlayerGameProfiles, PlayerId, PlayerInteractionKind};

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
                left: px(-2),
                top: px(-8),
                width: px(19),
                height: px(17),
                ..default()
            },
            ImageNode::new(assets.controls.host_crown.clone()),
            UiTransform::from_rotation(Rot2::radians(-0.2)),
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
        width: px(304),
        min_height: px(145),
        padding: UiRect {
            left: px(12),
            right: px(12),
            top: px(16),
            bottom: px(16),
        },
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(7),
        ..default()
    };
    position_opponent_popup(&mut node, side);
    let menu = add_cozy_panel(commands, parent, node, assets);
    commands.entity(menu).insert((
        InteractionMenuPanel(target),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));
    let profile = spawn_node(
        commands,
        menu,
        Node {
            width: percent(100),
            height: px(42),
            min_height: px(42),
            padding: UiRect::axes(px(5), px(3)),
            align_items: AlignItems::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    add_avatar(commands, profile, player_name, avatar, 34.0, assets);
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
    add_text(commands, identity, player_name, 15.0, TEXT, assets);
    add_text(
        commands,
        identity,
        format!(
            "等级:{}  分数:{}",
            reference_level(reference_points),
            reference_points,
        ),
        10.5,
        MUTED,
        assets,
    );
    add_cozy_button(
        commands,
        profile,
        "完整资料",
        UiAction::Navigation(NavigationUiAction::OpenPlayerProfile(Box::new(
            PlayerProfilePage {
                name: player_name.to_owned(),
                avatar: avatar.cloned(),
                reference_points,
                completed_games,
                game_profiles: game_profiles.clone(),
            },
        ))),
        assets,
        px(78),
        30.0,
    );
    let actions = spawn_node(
        commands,
        menu,
        Node {
            width: percent(100),
            height: px(64),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(0),
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
        let mut base = ImageNode::new(assets.home.button.clone()).with_mode(NodeImageMode::Sliced(
            TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.5,
            },
        ));
        base.visual_box = VisualBox::BorderBox;
        let button = commands
            .spawn((
                Button,
                UiAction::Social(SocialUiAction::SendInteraction { target, kind }),
                Node {
                    width: px(69),
                    height: px(64),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    ..default()
                },
                base,
            ))
            .id();
        commands.entity(actions).add_child(button);
        let mut highlight = ImageNode::new(assets.home.purple_button_compact.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.5,
            }),
        );
        highlight.visual_box = VisualBox::BorderBox;
        let overlay = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                highlight,
                Visibility::Hidden,
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(overlay);
        commands.entity(button).insert(HomeHighlightKind::Button {
            overlay,
            arrows: None,
        });
        let icon = commands
            .spawn((
                Node {
                    width: px(36),
                    height: px(36),
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
        add_text(commands, button, label, 12.0, TEXT, assets);
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
