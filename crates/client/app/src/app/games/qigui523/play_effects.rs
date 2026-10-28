//! 七鬼五二三出牌特效适配与发牌音效调度。

use super::{QiGui523UiState, sort_cards_high_to_low};
use crate::app::presentation::{
    CardPlaySoundKind, PendingDealSound, PlayEffectKind, PlayEffectState,
};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, QiGui523Snapshot, TABLE_SEAT_COUNT};
use leocard_qigui523::QiGuiPlayKind;

pub(super) fn sync_play_effect(
    client: Option<Res<ClientResource>>,
    mut effect: ResMut<PlayEffectState>,
    mut ui: ResMut<UiState>,
    mut game_ui: ResMut<QiGui523UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let Some(model) = client.as_deref().map(|client| client.0.model()) else {
        return;
    };
    let Some(game) = model.qigui523_game() else {
        return;
    };
    if game_ui.play_effect_serial == model.play_effect_serial() {
        return;
    }
    game_ui.play_effect_serial = model.play_effect_serial();
    if let Some((_, play)) = model.last_play_effect() {
        let sounds = match card_play_sound_kind(&play.kind) {
            CardPlaySoundKind::Place => &assets.audio.place_sounds,
            CardPlaySoundKind::Shove => &assets.audio.shove_sounds,
        };
        if !sounds.is_empty() {
            let sound = sounds[fastrand::usize(..sounds.len())].clone();
            commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
        }
    }
    effect.active = None;
    if let Some((player, play)) = model.last_play_effect()
        && let Some(kind) = pattern_effect(&play.kind)
    {
        effect.start(game.match_id, *player, kind, play.cards.len());
    }
    ui.dirty = true;
}

fn card_play_sound_kind(kind: &QiGuiPlayKind) -> CardPlaySoundKind {
    match kind {
        QiGuiPlayKind::Straight { .. }
        | QiGuiPlayKind::ConsecutivePairs { .. }
        | QiGuiPlayKind::Airplane { .. } => CardPlaySoundKind::Shove,
        QiGuiPlayKind::Single
        | QiGuiPlayKind::Pair
        | QiGuiPlayKind::Triple
        | QiGuiPlayKind::TripleWithSingle
        | QiGuiPlayKind::TripleWithPair
        | QiGuiPlayKind::Bomb(_)
        | QiGuiPlayKind::HeavenBomb => CardPlaySoundKind::Place,
    }
}

fn pattern_effect(kind: &QiGuiPlayKind) -> Option<PlayEffectKind> {
    match kind {
        QiGuiPlayKind::Straight { .. } => Some(PlayEffectKind::Straight),
        QiGuiPlayKind::ConsecutivePairs { .. } => Some(PlayEffectKind::ConsecutivePairs),
        QiGuiPlayKind::Airplane { .. } => Some(PlayEffectKind::Airplane),
        QiGuiPlayKind::Bomb(_) => Some(PlayEffectKind::Bomb),
        QiGuiPlayKind::HeavenBomb => Some(PlayEffectKind::HeavenBomb),
        _ => None,
    }
}

pub(super) fn bomb_effect_source(game: &QiGui523Snapshot, player: PlayerId) -> Vec2 {
    let own_seat = game
        .players
        .iter()
        .find(|state| state.id == game.you)
        .map_or(0, |state| state.seat.0);
    let player_seat = game
        .players
        .iter()
        .find(|state| state.id == player)
        .map_or(own_seat, |state| state.seat.0);
    match (player_seat + TABLE_SEAT_COUNT - own_seat) % TABLE_SEAT_COUNT {
        0 => Vec2::new(0.0, 225.0),
        1 => Vec2::new(-385.0, 155.0),
        2 => Vec2::new(-385.0, -110.0),
        3 => Vec2::new(0.0, -145.0),
        4 => Vec2::new(385.0, -110.0),
        _ => Vec2::new(385.0, 155.0),
    }
}

pub(super) fn queue_deal_animations(
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<QiGui523UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let Some((match_id, mut hand)) = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game())
        .map(|game| (game.match_id, game.your_hand.clone()))
    else {
        if !ui.observed_hand.state.is_empty() {
            ui.observed_hand.clear();
        }
        return;
    };
    ui.observed_hand.observe(match_id);
    sort_cards_high_to_low(&mut hand);

    let new_cards = hand
        .iter()
        .copied()
        .filter(|card| !ui.observed_hand.state.contains(card))
        .collect::<Vec<_>>();
    for (index, card) in new_cards.into_iter().enumerate() {
        let animation = ui.card_animations.entry(card).or_default();
        animation.deal_elapsed = -(index as f32 * 0.045);
        animation.dealing = true;
        if !assets.audio.deal_sounds.is_empty() {
            commands.spawn(PendingDealSound {
                remaining: index as f32 * 0.045,
                variant: fastrand::usize(..assets.audio.deal_sounds.len()),
            });
        }
    }
    if ui.observed_hand.state != hand {
        ui.observed_hand.state = hand;
    }
}
