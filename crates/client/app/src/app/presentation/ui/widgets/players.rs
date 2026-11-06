//! 玩家框、头像、准备标记和房主标记。

use super::super::{GAME_SIDE_CONTROL_SCALE, MUTED, READY, TEXT};
use super::{add_avatar, add_ready_badge, add_text};
use crate::app::presentation::SeatSide;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    InteractionMenuPanel, NavigationUiAction, OpponentBadge, PlayerAvatarAnchor, PlayerProfilePage,
    SocialUiAction, UiAction, add_cozy_button, add_cozy_panel, add_interaction_buttons,
    reference_level,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_game_common::PlayerId;
use leocard_protocol::PlayerGameProfiles;
use leocard_ui::{AnchoredPopup, UiComposer, spawn_node};

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
        width: px(300.0 * GAME_SIDE_CONTROL_SCALE),
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
    add_interaction_buttons(commands, menu, target, assets);
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
