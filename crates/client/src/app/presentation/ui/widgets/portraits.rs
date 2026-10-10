//! 可复用的无底板玩家头像：姓名、静态托管遮罩和原有社交交互。

use super::super::{MUTED, TEXT};
use super::{PlayerMenuProfile, add_interaction_menu, add_text, avatar_color, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{OpponentBadge, PlayerAvatarAnchor, SeatSide, SocialUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_protocol::PlayerId;

pub(crate) struct PlayerPortraitSpec<'a> {
    pub player: PlayerId,
    pub profile: PlayerMenuProfile<'a>,
    pub side: SeatSide,
    pub avatar_size: f32,
    pub auto_play: bool,
    pub menu_open: bool,
    pub menu_above: bool,
    pub name_color: Color,
}

pub(crate) struct PlayerPortraitEntities {
    pub portrait: Entity,
    pub avatar_ring: Entity,
}

pub(crate) struct PlayerSeatValue<'a> {
    pub label: &'a str,
    pub value: i32,
    pub width: f32,
    pub rotation: f32,
    pub active: bool,
}

/// 在父容器中心绘制可旋转的方位/分数行，供牌桌中央的各座位复用。
pub(crate) fn add_player_seat_value(
    commands: &mut Commands,
    parent: Entity,
    value: PlayerSeatValue<'_>,
    assets: &UiAssets,
) -> Entity {
    let score = value.value.to_string();
    let units = value.label.chars().count() as f32 + score.len() as f32 * 0.65;
    let font_size = ((value.width - 10.0) / units.max(1.0)).clamp(9.0, 16.0);
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: px(value.width),
            min_width: px(value.width),
            height: px(20),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    commands.entity(row).insert((
        UiTransform::from_rotation(Rot2::radians(value.rotation)),
        Pickable::IGNORE,
    ));
    let color = if value.active { TEXT } else { MUTED };
    let mut score_entity = row;
    for text in [value.label, score.as_str()] {
        let text = add_text(commands, row, text, font_size, color, assets);
        commands
            .entity(text)
            .insert((TextLayout::default().with_no_wrap(), Pickable::IGNORE));
        score_entity = text;
    }
    score_entity
}

pub(crate) fn add_player_portrait(
    commands: &mut Commands,
    parent: Entity,
    mut node: Node,
    spec: PlayerPortraitSpec<'_>,
    assets: &UiAssets,
) -> PlayerPortraitEntities {
    node.flex_direction = FlexDirection::Column;
    node.align_items = AlignItems::Center;
    node.row_gap = px(4.0 * 1.17);
    let portrait = spawn_node(commands, parent, node, None);
    commands.entity(portrait).insert((
        Button,
        UiAction::Social(SocialUiAction::ToggleInteractionMenu(spec.player)),
    ));
    let size = spec.avatar_size;
    let avatar_ring = spawn_node(
        commands,
        portrait,
        Node {
            width: px(size),
            height: px(size),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    commands.entity(avatar_ring).insert(Pickable::IGNORE);
    let avatar = spawn_node(
        commands,
        avatar_ring,
        Node {
            width: px(size),
            height: px(size),
            min_width: px(size),
            min_height: px(size),
            border: UiRect::all(px(1.25)),
            border_radius: BorderRadius::all(px(size * 0.2)),
            overflow: Overflow::clip(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(avatar_color(spec.profile.name)),
    );
    commands.entity(avatar).insert((
        PlayerAvatarAnchor(spec.player),
        BorderColor::all(Color::srgb(0.82, 0.88, 0.84).with_alpha(0.78)),
        Pickable::IGNORE,
    ));
    if let Some(image) = spec.profile.avatar {
        commands
            .entity(avatar)
            .insert(ImageNode::new(image.clone()));
    } else {
        let initial = spec.profile.name.chars().next().unwrap_or('玩').to_string();
        let initial = add_text(commands, avatar, initial, size * 0.42, TEXT, assets);
        commands.entity(initial).insert(Pickable::IGNORE);
    }
    if spec.auto_play {
        add_avatar_auto_play_overlay(commands, avatar, size, assets);
    }
    let name_area = spawn_node(
        commands,
        portrait,
        Node {
            width: px(96.0 * 1.17),
            height: px(20.0 * 1.17),
            border_radius: BorderRadius::all(px(5)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.48)),
    );
    commands.entity(name_area).insert((
        UiTransform::from_translation(Val2::px(0.0, 2.0)),
        Pickable::IGNORE,
    ));
    let name = add_text(
        commands,
        name_area,
        spec.profile.name,
        14.0 * 1.17,
        spec.name_color,
        assets,
    );
    commands.entity(name).insert((
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.8),
        },
        Pickable::IGNORE,
    ));
    let menu = add_interaction_menu(
        commands,
        portrait,
        spec.player,
        spec.side,
        spec.profile,
        assets,
    );
    let menu_above = spec.menu_above;
    commands
        .entity(menu)
        .entry::<Node>()
        .and_modify(move |mut node| {
            if menu_above {
                node.top = Val::Auto;
                node.bottom = percent(100);
                node.margin.bottom = px(6);
            } else {
                node.top = percent(100);
                node.margin.top = px(6);
            }
        });
    commands.entity(menu).insert(if spec.menu_open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    });
    commands.entity(portrait).insert(OpponentBadge {
        player: spec.player,
        score_popup: None,
        interaction_menu: menu,
    });
    PlayerPortraitEntities {
        portrait,
        avatar_ring,
    }
}

/// 托管图标直接覆盖头像，保持与头像相同的圆角。
pub(crate) fn add_avatar_auto_play_overlay(
    commands: &mut Commands,
    avatar: Entity,
    avatar_size: f32,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            left: px(1),
            right: px(1),
            top: px(1),
            bottom: px(1),
            border_radius: BorderRadius::all(px(avatar_size * 0.2 - 1.0)),
            overflow: Overflow::clip(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.62)),
    );
    commands
        .entity(overlay)
        .insert((ZIndex(1), Pickable::IGNORE));
    let robot = commands
        .spawn((
            Node {
                width: percent(78),
                height: percent(78),
                ..default()
            },
            ImageNode::new(assets.controls.robot_icon.clone()),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(overlay).add_child(robot);
}
