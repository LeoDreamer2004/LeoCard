use crate::app::games::GameLobbyContext;
use crate::app::shell::{LobbyPage, LobbyPageConfig};
use bevy::prelude::*;
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

pub(super) fn render(
    commands: &mut UiComposer,
    root: Entity,
    lobby: &LobbySnapshot,
    context: GameLobbyContext<'_>,
) {
    LobbyPage::spawn(
        commands,
        root,
        context.client,
        lobby,
        context.assets,
        context.avatars,
        LobbyPageConfig::new("麻将连连看"),
    );
}
