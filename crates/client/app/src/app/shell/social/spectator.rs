use super::{OpponentBadge, SocialUiAction};
use crate::app::presentation::{
    GAME_SIDE_CONTROL_SCALE, PlayerMenuProfile, add_avatar, attach_player_interactions,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{UiAction, UiState, add_cozy_button};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_client::{ClientModel, ClientPhaseRef};
use leocard_game_common::PlayerId;
use leocard_ui::{AnchoredPopup, UiComposer, spawn_node};

pub(crate) fn add_spectator_avatars(
    commands: &mut UiComposer,
    parent: Entity,
    model: &ClientModel,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let Some(room) = model.room_moderation() else {
        return;
    };
    let size = 32.0 * GAME_SIDE_CONTROL_SCALE;
    let list = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            width: px(size),
            flex_direction: FlexDirection::Column,
            row_gap: px(6.0 * GAME_SIDE_CONTROL_SCALE),
            ..default()
        },
        None,
    );
    commands.entity(list).insert(Pickable::IGNORE);
    for participant in room.participants.iter().filter(|player| player.spectating) {
        let player = &participant.player;
        let row = commands.keyed_child(
            list,
            format!("spectator:{}", player.id.0),
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                ..default()
            },
        );
        let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
        let image = add_avatar(commands, row, &player.name, avatar, size, assets);
        attach_player_interactions(
            commands,
            image,
            image,
            player.id,
            PlayerMenuProfile {
                name: &player.name,
                avatar,
                reference_points: player.reference_points,
                completed_games: player.completed_games,
                game_profiles: &player.game_profiles,
            },
            assets,
        );
        commands.entity(image).insert(GlobalZIndex(1460));
    }
}

#[derive(Component)]
pub(crate) struct SpectatorViewMenu;

fn add_spectator_view_menu(
    commands: &mut UiComposer,
    parent: Entity,
    player: PlayerId,
    assets: &UiAssets,
) {
    let button = add_cozy_button(
        commands,
        parent,
        "切换视角",
        UiAction::Social(SocialUiAction::SpectatePlayer(player)),
        assets,
        px(120),
        42.0,
    );
    commands.entity(button).insert((
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(100),
            width: px(120),
            height: px(42),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        SpectatorViewMenu,
        AnchoredPopup(parent),
        GlobalZIndex(1600),
    ));
}

pub(crate) fn sync_spectator_view_menu(
    mut commands: UiComposer,
    mouse: Res<ButtonInput<MouseButton>>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut ui: ResMut<UiState>,
    badges: Query<(Entity, &OpponentBadge, &PickingInteraction)>,
    menus: Query<Entity, With<SpectatorViewMenu>>,
) {
    let model = client.as_deref().map(|client| client.0.model());
    let spectating = model.is_some_and(|model| {
        matches!(model.phase(), ClientPhaseRef::Playing(_)) && model.can_change_spectator_view()
    });
    let right_click = mouse.just_pressed(MouseButton::Right);
    if spectating && !right_click && !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for menu in &menus {
        commands.entity(menu).despawn();
    }
    if !spectating || !right_click {
        return;
    }
    let Some((anchor, badge, _)) = badges.iter().find(|(_, badge, interaction)| {
        **interaction != PickingInteraction::None
            && model.is_some_and(|model| {
                model
                    .game_snapshot()
                    .is_some_and(|game| game.player(badge.player).is_some())
            })
    }) else {
        return;
    };
    ui.social.interaction_menu_open = None;
    add_spectator_view_menu(&mut commands, anchor, badge.player, &assets);
}
