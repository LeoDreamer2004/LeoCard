use super::state::{LanSearchPulse, RoomSignal};
use crate::app::presentation::MUTED;
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct JoinRoomAssets {
    pub signals: [Handle<Image>; 5],
    pub checking: [Handle<Image>; 5],
    pub unreachable: Handle<Image>,
}

pub(in super::super) fn load_room_assets(mut commands: Commands, server: Res<AssetServer>) {
    let load = |name: &str| server.load(format!("ui/rooms/{name}.png"));
    commands.insert_resource(JoinRoomAssets {
        signals: [1, 2, 3, 4, 5].map(|i| load(&format!("ping_{i}"))),
        checking: [1, 2, 3, 4, 5].map(|i| load(&format!("pinging_{i}"))),
        unreachable: load("unreachable"),
    });
}

impl JoinRoomAssets {
    pub fn signal(&self, signal: RoomSignal, time: f32) -> Handle<Image> {
        match signal {
            RoomSignal::Checking => self.checking[(time * 8.0) as usize % 5].clone(),
            RoomSignal::Offline => self.unreachable.clone(),
            RoomSignal::Online(ms) => self.signals[match ms {
                0..150 => 4,
                150..300 => 3,
                300..600 => 2,
                600..1000 => 1,
                _ => 0,
            }]
            .clone(),
        }
    }
}

pub(in super::super) fn animate_room_signals(
    time: Res<Time>,
    assets: Res<JoinRoomAssets>,
    mut signals: Query<(&RoomSignal, &mut ImageNode)>,
    mut searches: Query<(&LanSearchPulse, &mut BackgroundColor)>,
) {
    for (signal, mut image) in &mut signals {
        if matches!(signal, RoomSignal::Checking) {
            let frame = assets.signal(*signal, time.elapsed_secs());
            if image.image != frame {
                image.image = frame;
            }
        }
    }
    let active = [0, 1, 2, 1][(time.elapsed_secs() * 3.0) as usize % 4];
    for (pulse, mut color) in &mut searches {
        color.set_if_neq(BackgroundColor(MUTED.with_alpha(if pulse.0 == active {
            1.0
        } else {
            0.25
        })));
    }
}
