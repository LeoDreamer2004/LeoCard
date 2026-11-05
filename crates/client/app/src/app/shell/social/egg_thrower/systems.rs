use super::{AutoEggThrower, EggThrowerCount};
use crate::app::runtime::{ClientResource, PageErrorState};
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, PropId};
use leocard_protocol::{ClientCommand, PlayerInteractionKind};

pub(crate) fn tick_auto_egg_thrower(
    time: Res<Time<Real>>,
    mut thrower: ResMut<AutoEggThrower>,
    mut client: Option<ResMut<ClientResource>>,
    mut error: ResMut<PageErrorState>,
) {
    let Some(burst) = thrower.active.as_mut() else {
        return;
    };
    let Some(client) = client
        .as_deref_mut()
        .filter(|client| burst.still_valid(&client.0))
    else {
        thrower.active = None;
        return;
    };
    let now = time.elapsed();
    if burst.ready(now) {
        // Never catch up with a batch after a slow frame. Send ordinary requests
        // through the existing host validation and anti-replay path.
        if !client.0.send(ClientCommand::Interact {
            target: burst.target,
            kind: PlayerInteractionKind::Egg,
        }) {
            thrower.active = None;
            error.error = Some("自动砸蛋已停止：无法发送互动消息".to_owned());
            return;
        }
        burst.record_sent(now);
    }
    if burst.finished() {
        thrower.active = None;
    }
}

pub(crate) fn sync_egg_thrower_counts(
    economy: Res<PlayerEconomy>,
    mut counts: Query<&mut Text, With<EggThrowerCount>>,
) {
    let label = format!("砸蛋×{}", economy.prop_count(PropId::AutoEggThrower));
    for mut text in &mut counts {
        if text.0 != label {
            text.0.clone_from(&label);
        }
    }
}
