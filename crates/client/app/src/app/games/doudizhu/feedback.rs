use super::DoudizhuUiState;
use crate::app::presentation::{PlayEffectKind, PlayEffectState};
use crate::app::runtime::{ClientResource, ServerNotification, UiAssets};
use bevy::prelude::*;
use leocard_doudizhu::{Action, PlayKind, classify};
use leocard_protocol::{DoudizhuEvent, GameEvent, MatchId, ServerEvent};

#[derive(Resource, Default)]
pub(super) struct DoudizhuFeedback {
    match_id: Option<MatchId>,
    sequence: Option<u128>,
}

pub(super) fn play_feedback(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    assets: Res<UiAssets>,
    mut feedback: ResMut<DoudizhuFeedback>,
    mut ui: ResMut<DoudizhuUiState>,
    mut effects: ResMut<PlayEffectState>,
) {
    let Some(game) = client.as_deref().and_then(|c| c.0.model().doudizhu_game()) else {
        notifications.clear();
        *feedback = DoudizhuFeedback::default();
        return;
    };
    if feedback.match_id != Some(game.match_id) {
        *feedback = DoudizhuFeedback {
            match_id: Some(game.match_id),
            ..default()
        };
    }
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::Doudizhu(event)) = &notification.event else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if context.match_id != game.match_id
            || feedback.sequence.is_some_and(|s| s >= context.sequence)
        {
            continue;
        }
        feedback.sequence = Some(context.sequence);
        let variants = match event {
            DoudizhuEvent::ActionApplied { player, action } => match action {
                Action::Play(cards) => {
                    let kind =
                        classify(cards, game.rules).and_then(|play| pattern_effect(play.kind));
                    effects.active = None;
                    if let Some(kind) = kind {
                        effects.start(game.match_id, *player, kind, cards.len());
                    }
                    if kind.is_some_and(PlayEffectKind::is_sequence) {
                        &assets.audio.shove_sounds
                    } else {
                        &assets.audio.place_sounds
                    }
                }
                Action::Bid(_) | Action::Pass => continue,
            },
            DoudizhuEvent::Redealt => {
                ui.pending_deal = Some(game.match_id);
                continue;
            }
            DoudizhuEvent::HandFinished { .. } | DoudizhuEvent::ActionAnalyzed { .. } => continue,
        };
        if let Some(sound) = variants.get(fastrand::usize(..variants.len().max(1))) {
            commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
}

fn pattern_effect(kind: PlayKind) -> Option<PlayEffectKind> {
    match kind {
        PlayKind::Straight => Some(PlayEffectKind::Straight),
        PlayKind::Pairs => Some(PlayEffectKind::ConsecutivePairs),
        PlayKind::Airplane | PlayKind::AirplaneSingles | PlayKind::AirplanePairs => {
            Some(PlayEffectKind::Airplane)
        }
        PlayKind::Bomb | PlayKind::Rocket => Some(PlayEffectKind::Bomb),
        _ => None,
    }
}
