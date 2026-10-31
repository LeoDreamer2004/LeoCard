//! Gender-specific tile announcements use only public discard events.

use super::MahjongAssets;
use crate::app::runtime::AppearancePreferences;
use bevy::audio::Volume;
use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongClaim, MahjongTileKind};
use leocard_protocol::{MahjongEvent, MahjongSnapshot};
use leocard_ui::UiComposer;

use bevy::prelude::*;

#[derive(Component)]
pub(super) struct MahjongTileVoicePlayback;

pub(super) fn mahjong_tile_voice_key(kind: MahjongTileKind) -> Option<String> {
    Some(match kind {
        MahjongTileKind::Suited { suit, rank } => format!("{suit:?}_{rank}").to_ascii_lowercase(),
        MahjongTileKind::Wind(wind) => format!("{wind:?}").to_ascii_lowercase(),
        MahjongTileKind::Dragon(dragon) => format!("{dragon:?}").to_ascii_lowercase(),
        MahjongTileKind::Flower(_) => return None,
    })
}

enum TileVoiceUpdate {
    Discard(PlayerId, MahjongTileKind),
    Stop,
}

pub(super) fn sync_mahjong_tile_voice(
    commands: &mut UiComposer,
    assets: &MahjongAssets,
    preferences: &AppearancePreferences,
    game: &MahjongSnapshot,
    events: &[MahjongEvent],
    new_match: bool,
    playing: &Query<Entity, With<MahjongTileVoicePlayback>>,
) {
    // Only the latest public action matters when multiple events arrive together.
    let update = events.iter().rev().find_map(|event| match event {
        MahjongEvent::TileDiscarded { player, tile } => {
            Some(TileVoiceUpdate::Discard(*player, tile.kind()))
        }
        MahjongEvent::ClaimResolved { claim, .. } if !matches!(claim, MahjongClaim::Pass) => {
            Some(TileVoiceUpdate::Stop)
        }
        MahjongEvent::KongDeclared { .. }
        | MahjongEvent::HandFinished { .. }
        | MahjongEvent::FlowerReplaced { .. } => Some(TileVoiceUpdate::Stop),
        _ => None,
    });
    if update.is_none() && !new_match && preferences.mahjong_tile_voices {
        return;
    }
    for entity in playing {
        commands.entity(entity).despawn();
    }
    if !preferences.mahjong_tile_voices {
        return;
    }
    let Some(TileVoiceUpdate::Discard(player, tile)) = update else {
        return;
    };
    let Some(profile) = game
        .players
        .iter()
        .find(|profile| profile.player.id == player)
    else {
        return;
    };
    let Some(sound) = assets.tile_voice(profile.player.game_profiles.gender, tile) else {
        return;
    };
    commands.effect((
        AudioPlayer::new(sound.clone()),
        PlaybackSettings {
            volume: Volume::Linear(0.88),
            ..PlaybackSettings::DESPAWN
        },
        MahjongTileVoicePlayback,
    ));
}
