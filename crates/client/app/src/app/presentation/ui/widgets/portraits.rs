//! 可复用的无底板玩家头像：姓名、行动耗时、静态托管遮罩和社交交互。

use super::{PlayerMenuProfile, add_framed_avatar, add_interaction_menu, text_scene};
use crate::app::runtime::{ActionTimingLabel, UiAssets};
use crate::app::shell::{OpponentBadge, PlayerAvatarAnchor, SocialUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::LineHeight;
use bevy::ui_widgets::Button;
use leocard_game_common::PlayerId;
use leocard_ui::{UiComposer, UiSceneKey, ui_component};

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
    pub name: Entity,
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
    let name = text_scene(assets, spec.profile.name, 12.0, spec.name_color);
    let portrait = commands.mount(
        parent,
        format!("portrait:{}", spec.player.0),
        bsn! { node Button },
    );
    let framed = add_framed_avatar(
        commands,
        portrait,
        "ring",
        spec.profile.name,
        spec.profile.avatar,
        assets
            .avatar_frames
            .frame(spec.profile.game_profiles.avatar_frame, size),
        assets,
    );
    let avatar_ring = framed.frame;
    let avatar = framed.avatar;
    commands
        .entity(avatar)
        .insert(PlayerAvatarAnchor(spec.player));
    let name_area = commands.mount(
        portrait,
        "name-area",
        bsn! {
            Node {
                // Keep both rows inside the original 23.4px name area.
                width: px(96.0 * 1.17), height: px(20.0 * 1.17),
                flex_shrink: 0.0, flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
            }
            BackgroundColor({Color::BLACK.with_alpha(0.48)})
            Pickable::IGNORE
            Children [
                UiSceneKey("name")
                Node { width: percent(100), height: px(13), flex_shrink: 0.0 }
                @name
                TextLayout::justify(Justify::Center).with_no_wrap()
                LineHeight::Px(13.0)
                Pickable::IGNORE
            ]
        },
    );
    let name = commands.entity_for(name_area, "name");
    add_portrait_action_timing(commands, name_area, &spec, assets);
    if spec.auto_play {
        add_avatar_auto_play_overlay(commands, avatar, size, assets);
    }
    let menu = add_interaction_menu(commands, portrait, spec.player, spec.profile, assets);
    commands.entity(menu).insert(if spec.menu_open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    });
    commands.entity(portrait).insert((
        UiAction::Social(SocialUiAction::ToggleInteractionMenu {
            player: spec.player,
            anchor: portrait,
        }),
        OpponentBadge {
            player: spec.player,
            score_popup: None,
            interaction_menu: menu,
        },
    ));
    PlayerPortraitEntities {
        portrait,
        avatar_ring,
        name,
    }
}

/// Reserve the second name-area row even when the runtime hides the timer.
fn add_portrait_action_timing(
    commands: &mut UiComposer,
    name_area: Entity,
    spec: &PlayerPortraitSpec<'_>,
    assets: &UiAssets,
) {
    // Retain the runtime-owned Text/Visibility across page reconciliation.
    let (row, cached) = commands.cached_child(
        name_area,
        "action-timing",
        (spec.player, spec.name_color, assets.font.clone()),
        (
            Node {
                width: percent(100),
                height: px(20.0 * 1.17 - 13.0),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ),
    );
    if cached {
        return;
    }
    let text = text_scene(assets, "", 10.0, spec.name_color);
    let layout = TextLayout::justify(Justify::Center).with_no_wrap();
    let label = commands.mount(
        row,
        "label",
        bsn! {
            Node { width: percent(100), height: percent(100) }
            @text @ui_component(layout)
            LineHeight::Px(10.4)
            Visibility::Hidden Pickable::IGNORE
        },
    );
    commands
        .entity(label)
        .insert(ActionTimingLabel(spec.player));
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
