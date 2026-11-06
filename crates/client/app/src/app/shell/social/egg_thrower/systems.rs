use super::{AutoEggThrower, EggThrowerCount};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_client::{PlayerEconomy, PropId};

pub(crate) fn tick_auto_egg_thrower(
    time: Res<Time<Real>>,
    mut thrower: ResMut<AutoEggThrower>,
    client: Option<Res<ClientResource>>,
) {
    if thrower.active.as_ref().is_some_and(|burst| {
        burst.finished(time.elapsed())
            || client
                .as_deref()
                .is_none_or(|client| !burst.still_valid(&client.0))
    }) {
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
