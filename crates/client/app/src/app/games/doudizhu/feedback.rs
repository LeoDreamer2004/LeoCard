use super::DoudizhuUiState;
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
            DoudizhuEvent::ActionApplied { action, .. } => match action {
                Action::Play(cards) => {
                    if classify(cards, game.rules.mode)
                        .is_some_and(|play| matches!(play.kind, PlayKind::Bomb | PlayKind::Rocket))
                    {
                        commands.spawn((
                            AudioPlayer::new(assets.audio.bomb_explosion_sound.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    }
                    &assets.audio.place_sounds
                }
                Action::Bid(_) | Action::Pass => continue,
            },
            DoudizhuEvent::Redealt => {
                ui.pending_deal = Some(game.match_id);
                continue;
            }
            DoudizhuEvent::HandFinished { .. } => continue,
        };
        if let Some(sound) = variants.get(fastrand::usize(..variants.len().max(1))) {
            commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
}
