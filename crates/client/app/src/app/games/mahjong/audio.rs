use super::discard::DISCARD_DURATION;
use crate::app::presentation::PendingDealSound;
use crate::app::runtime::{ClientResource, UiAssets};
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, MahjongEvent, MahjongSnapshot, MatchId};
use leocard_ui::UiComposer;

pub(super) struct MahjongTableSounds {
    draw: Handle<AudioSource>,
    discard: Handle<AudioSource>,
}

#[derive(Component)]
pub(super) struct PendingMahjongTileSound {
    hand: (MatchId, u8),
    sound: Handle<AudioSource>,
    remaining: f32,
    volume: f32,
}

impl MahjongTableSounds {
    pub fn load(server: &AssetServer) -> Self {
        Self {
            draw: server.load("audio/mahjong/table/draw.ogg"),
            discard: server.load("audio/mahjong/table/discard.ogg"),
        }
    }

    pub fn queue_draw(&self, commands: &mut UiComposer, game: &MahjongSnapshot) {
        self.queue(commands, game, &self.draw, 0.04, 0.50);
    }

    pub fn queue_event(
        &self,
        commands: &mut UiComposer,
        game: &MahjongSnapshot,
        event: &MahjongEvent,
    ) {
        match event {
            MahjongEvent::TileDrawn { .. } | MahjongEvent::FlowerReplaced { .. } => {
                self.queue_draw(commands, game);
            }
            MahjongEvent::TileDiscarded { .. } => {
                self.queue(commands, game, &self.discard, DISCARD_DURATION, 0.62);
            }
            _ => {}
        }
    }

    fn queue(
        &self,
        commands: &mut UiComposer,
        game: &MahjongSnapshot,
        sound: &Handle<AudioSource>,
        remaining: f32,
        volume: f32,
    ) {
        commands.effect(PendingMahjongTileSound {
            hand: (game.match_id, game.sequence_index),
            sound: sound.clone(),
            remaining,
            volume,
        });
    }
}

pub(super) fn queue_mahjong_deal_sound(commands: &mut UiComposer, assets: &UiAssets) {
    if assets.audio.deal_sounds.is_empty() {
        return;
    }
    commands.effect(PendingDealSound {
        remaining: 0.0,
        variant: fastrand::usize(..assets.audio.deal_sounds.len()),
    });
}

pub(super) fn play_mahjong_tile_sounds(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    mut pending: Query<(Entity, &mut PendingMahjongTileSound)>,
    mut commands: Commands,
) {
    let hand = client
        .as_deref()
        .and_then(|client| {
            client
                .0
                .model()
                .game_snapshot()
                .and_then(GameSnapshot::mahjong)
        })
        .map(|game| (game.match_id, game.sequence_index));
    for (entity, mut sound) in &mut pending {
        if hand != Some(sound.hand) {
            commands.entity(entity).despawn();
            continue;
        }
        sound.remaining -= time.delta_secs();
        if sound.remaining > 0.0 {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(sound.sound.clone()),
            PlaybackSettings {
                volume: Volume::Linear(sound.volume),
                ..PlaybackSettings::DESPAWN
            },
        ));
        commands.entity(entity).despawn();
    }
}
