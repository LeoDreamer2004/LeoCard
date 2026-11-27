use super::discard::DISCARD_DURATION;
use super::{MahjongAssets, MahjongDealTile, MahjongDiscardHandShift, MahjongHandTile};
use crate::app::runtime::ClientResource;
use bevy::audio::Volume;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use leocard_mahjong::MahjongTile;
use leocard_platform::is_mobile;
use leocard_protocol::{GameSnapshot, MahjongEvent, MahjongSnapshot, MatchId};
use leocard_ui::{UiComposer, UiPress};

pub(super) struct MahjongTableSounds {
    deal: Handle<AudioSource>,
    draw: Handle<AudioSource>,
    discard: Handle<AudioSource>,
    select: Handle<AudioSource>,
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
            deal: server.load("audio/mahjong/table/deal.ogg"),
            draw: server.load("audio/mahjong/table/draw.ogg"),
            discard: server.load("audio/mahjong/table/discard.ogg"),
            select: server.load("vendor/kenney/interface-sounds/Audio/glass_002.ogg"),
        }
    }

    pub fn queue_opening(
        &self,
        commands: &mut UiComposer,
        game: &MahjongSnapshot,
        tile_count: u16,
    ) {
        match tile_count {
            0 => {}
            1 => self.queue_draw(commands, game),
            _ => self.queue(commands, game, &self.deal, 0.0, 0.50),
        }
    }

    fn queue_draw(&self, commands: &mut UiComposer, game: &MahjongSnapshot) {
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

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query limits selection feedback to enabled, stationary hand tiles"
)]
pub(super) fn play_mahjong_hand_selection_sounds(
    mut presses: MessageReader<UiPress>,
    hands: Query<
        (&MahjongHandTile, &PickingInteraction),
        (
            Without<InteractionDisabled>,
            Without<MahjongDealTile>,
            Without<MahjongDiscardHandShift>,
        ),
    >,
    assets: Res<MahjongAssets>,
    mut previous_hover: Local<Option<MahjongTile>>,
    mut commands: Commands,
) {
    let mobile = is_mobile();
    let mut selected = false;
    for press in presses.read() {
        selected |= mobile && hands.contains(press.0);
    }
    let hovered = if mobile {
        None
    } else {
        hands.iter().find_map(|(tile, interaction)| {
            matches!(
                interaction,
                PickingInteraction::Hovered | PickingInteraction::Pressed
            )
            .then_some(tile.tile)
        })
    };
    selected |= hovered.is_some() && hovered != *previous_hover;
    *previous_hover = hovered;
    if selected {
        commands.spawn((
            AudioPlayer::new(assets.table_sounds.select.clone()),
            PlaybackSettings {
                volume: Volume::Linear(0.24),
                speed: 1.1,
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
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
