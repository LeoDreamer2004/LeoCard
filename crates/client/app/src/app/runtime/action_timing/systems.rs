use super::clock::{ActionTimingClock, ActionTimingLabel};
use crate::app::runtime::{ClientResource, ClientUpdateSet};
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;

pub(in crate::app::runtime) struct ActionTimingPlugin;

impl Plugin for ActionTimingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionTimingClock>()
            .add_systems(Update, synchronize_clocks.in_set(ClientUpdateSet::Sync))
            .add_systems(Update, update_labels.in_set(ClientUpdateSet::Animate));
    }
}

fn synchronize_clocks(
    client: Option<Res<ClientResource>>,
    time: Res<Time<Real>>,
    mut clock: ResMut<ActionTimingClock>,
) {
    let Some(client) = client else {
        clock.clear();
        return;
    };
    if client.is_added() {
        clock.clear();
    }
    let model = client.0.model();
    let snapshot = model.action_timing().filter(|timing| {
        matches!(model.phase(), ClientPhaseRef::Playing(game)
            if game.kind() == timing.game && game.match_id() == timing.match_id)
    });
    let Some(snapshot) = snapshot else {
        clock.clear();
        return;
    };
    clock.synchronize(
        model.room_id(),
        model.action_timing_serial(),
        snapshot,
        time.elapsed(),
    );
}

fn update_labels(
    time: Res<Time<Real>>,
    clock: Res<ActionTimingClock>,
    mut labels: Query<(&ActionTimingLabel, &mut Text, &mut Visibility)>,
) {
    let now = time.elapsed();
    for (label, mut text, mut visibility) in &mut labels {
        let elapsed = clock.elapsed(label.0, now);
        let next_visibility = if elapsed.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next_visibility {
            *visibility = next_visibility;
        }
        if let Some(elapsed) = elapsed {
            let tenths = elapsed.as_millis() / 100;
            let value = format!("用时 {}.{}秒", tenths / 10, tenths % 10);
            if text.0 != value {
                text.0 = value;
            }
        } else if !text.0.is_empty() {
            text.0.clear();
        }
    }
}
