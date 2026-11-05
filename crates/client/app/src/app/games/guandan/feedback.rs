use super::hand::sorted_hand;
use super::state::GuandanUiState;
use crate::app::presentation::{
    CardAnimationState, PendingDealSound, PlayEffectKind, PlayEffectState,
};
use crate::app::runtime::{ClientResource, ServerNotification, UiAssets};
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_guandan::{GuandanPlayKind, classify};
use leocard_protocol::{GameEvent, GameSnapshot, GuandanEvent, GuandanPublicAction, ServerEvent};
use std::mem;

pub(super) fn sync_play_feedback(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    assets: Res<UiAssets>,
    mut ui: ResMut<GuandanUiState>,
    mut effects: ResMut<PlayEffectState>,
    mut shell_ui: ResMut<UiState>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::guandan)
    }) else {
        notifications.clear();
        ui.clear();
        return;
    };
    if ui.reconcile(game) {
        effects.active = None;
    }
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::Guandan(event)) = &notification.event else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if context.match_id != game.match_id
            || context.hand_index != Some(game.hand_number)
            || ui
                .effect_sequence
                .is_some_and(|sequence| sequence >= context.sequence)
        {
            continue;
        }
        ui.effect_sequence = Some(context.sequence);
        shell_ui.dirty = true;
        match event {
            GuandanEvent::ActionApplied {
                player,
                action: GuandanPublicAction::Play(cards),
            } => {
                let kind = classify(cards, game.level)
                    .ok()
                    .and_then(|play| pattern_effect(play.kind));
                effects.active = None;
                if let Some(kind) = kind {
                    effects.start(game.match_id, *player, kind, cards.len());
                }
                let sounds = if kind.is_some_and(PlayEffectKind::is_sequence) {
                    &assets.audio.shove_sounds
                } else {
                    &assets.audio.place_sounds
                };
                if !sounds.is_empty() {
                    commands.spawn((
                        AudioPlayer::new(sounds[fastrand::usize(..sounds.len())].clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }
            GuandanEvent::TributeSubmitted { player } => {
                if let Some(target) = game.tribute_pairs.iter().find_map(|&(giver, receiver)| {
                    if *player == giver {
                        Some(receiver)
                    } else if *player == receiver {
                        Some(giver)
                    } else {
                        None
                    }
                }) {
                    ui.tribute_flights.push((*player, target));
                }
            }
            GuandanEvent::ActionApplied { .. } | GuandanEvent::HandFinished { .. } => {}
        }
    }
}

fn pattern_effect(kind: GuandanPlayKind) -> Option<PlayEffectKind> {
    match kind {
        GuandanPlayKind::Straight => Some(PlayEffectKind::Straight),
        GuandanPlayKind::ConsecutivePairs => Some(PlayEffectKind::ThreePairs),
        GuandanPlayKind::ConsecutiveTriples => Some(PlayEffectKind::ConsecutiveTriples),
        GuandanPlayKind::StraightFlush => Some(PlayEffectKind::StraightFlush),
        GuandanPlayKind::Bomb(_) => Some(PlayEffectKind::Bomb),
        GuandanPlayKind::JokerBomb => Some(PlayEffectKind::FourJokers),
        _ => None,
    }
}

pub(super) fn queue_deal(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut state: ResMut<GuandanUiState>,
    mut ui: ResMut<UiState>,
) {
    if !mem::take(&mut state.pending_deal) {
        return;
    }
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::guandan)
    }) else {
        return;
    };
    for (index, card) in sorted_hand(game).into_iter().enumerate() {
        let delay = index as f32 * 0.035;
        state.card_animations.insert(
            card,
            CardAnimationState {
                deal_elapsed: -delay,
                dealing: true,
                ..default()
            },
        );
        if !assets.audio.deal_sounds.is_empty() {
            commands.spawn(PendingDealSound {
                remaining: delay,
                variant: fastrand::usize(..assets.audio.deal_sounds.len()),
            });
        }
    }
    ui.dirty = true;
}

pub(super) fn advance_previous_trick(
    time: Res<Time>,
    mut state: ResMut<GuandanUiState>,
    mut ui: ResMut<UiState>,
) {
    if state.previous_reveal > 0.0 {
        state.previous_reveal = (state.previous_reveal - time.delta_secs()).max(0.0);
        ui.dirty |= state.previous_reveal == 0.0;
    }
}
