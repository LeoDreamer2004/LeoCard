//! 玩家框、头像、准备标记和房主标记。

use super::super::{BORDER, MUTED, READY, TEXT};
use super::{add_ready_badge, add_text, text_scene};
use crate::app::presentation::SeatSide;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    InteractionCooldownMask, InteractionMenuPanel, NavigationUiAction, OpponentBadge,
    PlayerAvatarAnchor, PlayerProfilePage, SocialUiAction, UiAction, add_cozy_button,
    add_cozy_panel, reference_level,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_game_common::PlayerId;
use leocard_protocol::{PlayerGameProfiles, PlayerInteractionKind};
use leocard_ui::{
    AnchoredPopup, TextureButton, UiComposer, UiSceneKey, sliced_image, spawn_node, ui_component,
};

#[track_caller]
pub(crate) fn add_avatar(
    commands: &mut UiComposer,
    parent: Entity,
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> Entity {
    commands.node(
        parent,
        bsn! {
            Node {
                width: px(size), height: px(size), min_width: px(size),
                border: UiRect::all(px(1)), border_radius: BorderRadius::all(percent(50)),
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            }
            BorderColor::all(BORDER)
            @avatar_content(name, image, size, assets)
        },
    )
}

pub(super) fn avatar_content(
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> impl Scene {
    let picture = image.map(|image| ui_component(ImageNode::new(image.clone())));
    let initial = image.is_none().then(|| {
        let text = text_scene(
            assets,
            name.chars().next().unwrap_or('玩').to_string(),
            size * 0.42,
            TEXT,
        );
        bsn! {
            Children [ UiSceneKey("initial") @text Pickable::IGNORE ]
        }
    });
    bsn! {
        BackgroundColor({avatar_color(name)})
        @picture
        @initial
    }
}

/// 结算窗口中的下一局准备状态。保持德州扑克原有的绿色头像环和右下角对钩，
/// 让所有游戏都能在玩家点击“再来一局”后直接看到彼此的准备进度。
pub(crate) fn add_ready_avatar(
    commands: &mut UiComposer,
    parent: Entity,
    player: PlayerId,
    profile: PlayerMenuProfile<'_>,
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
    let avatar = add_avatar(
        commands,
        frame,
        profile.name,
        profile.avatar,
        avatar_size,
        assets,
    );
    commands.entity(avatar).insert(Pickable::IGNORE);
    if ready {
        add_ready_badge(commands, frame, avatar_size, assets);
    }
    attach_player_interactions(commands, frame, frame, player, profile, assets);
    frame
}

pub(crate) fn attach_player_interactions(
    commands: &mut UiComposer,
    button: Entity,
    avatar: Entity,
    player: PlayerId,
    profile: PlayerMenuProfile<'_>,
    assets: &UiAssets,
) {
    let menu = add_interaction_menu(commands, button, player, profile, assets);
    commands
        .entity(menu)
        .insert((Visibility::Hidden, AnchoredPopup(avatar)));
    commands.entity(avatar).insert(PlayerAvatarAnchor(player));
    commands.entity(button).insert((
        Button,
        UiAction::Social(SocialUiAction::ToggleInteractionMenu {
            player,
            anchor: button,
        }),
        GlobalZIndex(1301),
        OpponentBadge {
            player,
            score_popup: None,
            interaction_menu: menu,
        },
    ));
}

pub(crate) fn add_host_crown(
    commands: &mut UiComposer,
    avatar: Entity,
    assets: &UiAssets,
) -> Entity {
    commands.child(
        avatar,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(-2),
                top: px(-8),
                width: px(19),
                height: px(17),
                ..default()
            },
            ImageNode::new(assets.controls.host_crown.clone()),
            UiTransform::from_rotation(Rot2::degrees(-35.0)),
            ZIndex(40),
            Pickable::IGNORE,
        ),
    )
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

#[derive(Clone, Copy)]
pub(crate) struct PlayerMenuProfile<'a> {
    pub name: &'a str,
    pub avatar: Option<&'a Handle<Image>>,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: &'a PlayerGameProfiles,
}

pub(crate) fn add_interaction_menu(
    commands: &mut UiComposer,
    parent: Entity,
    target: PlayerId,
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
    let node = Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: percent(100),
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
    let menu = add_cozy_panel(commands, parent, node, assets);
    commands.entity(menu).insert((
        InteractionMenuPanel(target),
        AnchoredPopup(parent),
        GlobalZIndex(1500),
        Pickable::default(),
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
                id: target,
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
        let icon = assets
            .social
            .interaction_images
            .get(&(kind, false))
            .expect("every interaction has a flight image")
            .clone();
        let button = TextureButton {
            node: Node {
                width: px(69),
                height: px(64),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(2),
                ..default()
            },
            normal: sliced_image(assets.home.button.clone(), 32.0, 0.5),
            hovered: sliced_image(assets.home.purple_button_compact.clone(), 32.0, 0.5),
            action: Some(UiAction::Social(SocialUiAction::SendInteraction {
                target,
                kind,
            })),
            label: Some(label),
            font_size: 12.0,
            color: TEXT,
            icon: Some((icon, 36.0)),
        }
        .mount(commands, actions, &assets.font)
        .0;
        if matches!(
            kind,
            PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe
        ) {
            commands.child(
                button,
                (
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
                    Pickable::IGNORE,
                ),
            );
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
            node.left = percent(50);
            node.margin.left = node.width * -0.5;
            node.top = px(76);
        }
        SeatSide::Right => {
            node.right = px(0);
            node.top = px(76);
        }
    }
}
