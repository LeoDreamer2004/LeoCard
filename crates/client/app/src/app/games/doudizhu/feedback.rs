use super::DoudizhuUiState;
use crate::app::presentation::{PlayEffectKind, PlayEffectState};
use crate::app::runtime::{ClientResource, ServerNotification, UiAssets};
use bevy::prelude::*;
use leocard_doudizhu::{DoudizhuAction, DoudizhuPlayKind, classify};
use leocard_protocol::{DoudizhuEvent, GameEvent, GameSnapshot, MatchId, ServerEvent};

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
    let Some(game) = client
        .as_deref()
        .and_then(|c| c.0.model().game_snapshot().and_then(GameSnapshot::doudizhu))
    else {
        notifications.clear();
        *feedback = DoudizhuFeedback::default();
        return;
    };
    ui.reconcile(game);
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
            || context.hand_index != Some(u32::from(game.progress.hand_number))
            || feedback.sequence.is_some_and(|s| s >= context.sequence)
        {
            continue;
        }
        feedback.sequence = Some(context.sequence);
        let variants = match event {
            DoudizhuEvent::ActionApplied { player, action } => {
                ui.history
                    .observe(*player, action, game.rules.mode.players());
                match action {
                    DoudizhuAction::Play(cards) => {
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
                    DoudizhuAction::Bid(_) | DoudizhuAction::Pass => continue,
                }
            }
            DoudizhuEvent::Redealt => {
                ui.pending_deal = Some(game.match_id);
                ui.history.clear();
                continue;
            }
            DoudizhuEvent::HandFinished { .. } | DoudizhuEvent::ActionAnalyzed { .. } => continue,
        };
        if let Some(sound) = variants.get(fastrand::usize(..variants.len().max(1))) {
            commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
}

fn pattern_effect(kind: DoudizhuPlayKind) -> Option<PlayEffectKind> {
    match kind {
        DoudizhuPlayKind::Straight => Some(PlayEffectKind::Straight),
        DoudizhuPlayKind::Pairs => Some(PlayEffectKind::ConsecutivePairs),
        DoudizhuPlayKind::Airplane
        | DoudizhuPlayKind::AirplaneSingles
        | DoudizhuPlayKind::AirplanePairs => Some(PlayEffectKind::Airplane),
        DoudizhuPlayKind::Bomb | DoudizhuPlayKind::Rocket => Some(PlayEffectKind::Bomb),
        _ => None,
    }
}
