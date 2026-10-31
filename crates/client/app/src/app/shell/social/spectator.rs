use super::{OpponentBadge, SocialUiAction};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{UiAction, UiState, add_cozy_button};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_game_common::PlayerId;
use leocard_ui::{AnchoredPopup, UiComposer};

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
        matches!(model.phase(), ClientPhaseRef::Playing(_)) && model.is_spectating()
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
            && model.is_some_and(|model| model.has_player(badge.player))
    }) else {
        return;
    };
    ui.social.interaction_menu_open = None;
    add_spectator_view_menu(&mut commands, anchor, badge.player, &assets);
}
