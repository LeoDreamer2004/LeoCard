//! 可复用的无底板玩家头像：姓名、静态托管遮罩和原有社交交互。

use super::super::{MUTED, TEXT};
use super::players::avatar_content;
use super::{PlayerMenuProfile, add_interaction_menu, add_text, spawn_node, text_scene};
use crate::app::presentation::{UiComposer, UiSceneKey, ui_component};
use crate::app::runtime::UiAssets;
use crate::app::shell::{OpponentBadge, PlayerAvatarAnchor, SocialUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_protocol::PlayerId;

pub(crate) struct PlayerPortraitSpec<'a> {
    pub player: PlayerId,
    pub profile: PlayerMenuProfile<'a>,
    pub avatar_size: f32,
    pub auto_play: bool,
    pub menu_open: bool,
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
    commands: &mut UiComposer,
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
    commands: &mut UiComposer,
    parent: Entity,
    mut node: Node,
    spec: PlayerPortraitSpec<'_>,
    assets: &UiAssets,
) -> PlayerPortraitEntities {
    node.flex_direction = FlexDirection::Column;
    node.align_items = AlignItems::Center;
    node.row_gap = px(4.0 * 1.17);
    let size = spec.avatar_size;
    let action = UiAction::Social(SocialUiAction::ToggleInteractionMenu(spec.player));
    let name = text_scene(assets, spec.profile.name, 14.0 * 1.17, spec.name_color);
    let portrait = commands.mount(parent, format!("portrait:{}", spec.player.0), bsn! {
        node
        Button
        @ui_component(action)
        Children [
            UiSceneKey("ring")
            Node { width: px(size), height: px(size), position_type: PositionType::Relative }
            Pickable::IGNORE
            Children [
                UiSceneKey("avatar")
                Node {
                    width: px(size), height: px(size), min_width: px(size), min_height: px(size),
                    border: UiRect::all(px(1.25)), border_radius: BorderRadius::all(px(size * 0.2)),
                    overflow: Overflow::clip(), align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                @ui_component(PlayerAvatarAnchor(spec.player))
                BorderColor::all(Color::srgb(0.82, 0.88, 0.84).with_alpha(0.78))
                Pickable::IGNORE
                @avatar_content(spec.profile.name, spec.profile.avatar, size, assets)
            ]
            --
            UiSceneKey("name-area")
            Node {
                width: px(96.0 * 1.17), height: px(20.0 * 1.17),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
            }
            BackgroundColor({Color::BLACK.with_alpha(0.48)})
            UiTransform::from_translation(Val2::px(0.0, 2.0))
            Pickable::IGNORE
            Children [
                @name
                TextLayout::no_wrap()
                TextShadow { offset: Vec2::new(1.0, 1.0), color: {Color::BLACK.with_alpha(0.8)} }
                Pickable::IGNORE
            ]
        ]
    });
    let avatar_ring = commands.entity_for(portrait, "ring");
    let avatar = commands.entity_for(avatar_ring, "avatar");
    if spec.auto_play {
        add_avatar_auto_play_overlay(commands, avatar, size, assets);
    }
    let menu = add_interaction_menu(commands, portrait, spec.player, spec.profile, assets);
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
    commands: &mut UiComposer,
    avatar: Entity,
    avatar_size: f32,
    assets: &UiAssets,
) {
    let image = ImageNode::new(assets.controls.robot_icon.clone());
    commands.mount(
        avatar,
        "auto-play",
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: px(1), right: px(1), top: px(1), bottom: px(1),
                border_radius: BorderRadius::all(px(avatar_size * 0.2 - 1.0)),
                overflow: Overflow::clip(), align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            BackgroundColor({Color::BLACK.with_alpha(0.62)})
            ZIndex(1) Pickable::IGNORE
            Children [
                Node { width: percent(78), height: percent(78) }
                @ui_component(image) Pickable::IGNORE
            ]
        },
    );
}
