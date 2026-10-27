use crate::app::presentation::{CardPatternVoice, CardPatternVoices};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_doudizhu::{Action, Mode, PlayKind, classify};
use leocard_protocol::{DoudizhuEvent, GameEvent, ServerEvent};

pub(super) fn sync_doudizhu_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<CardPatternVoices>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().doudizhu_game())
    else {
        notifications.clear();
        return;
    };
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::Doudizhu(DoudizhuEvent::ActionApplied {
            player,
            action: Action::Play(cards),
        })) = &notification.event
        else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if let Some(pattern) = classify(cards, game.rules.mode)
            .and_then(|play| pattern_voice(play.kind, game.rules.mode))
            && let Some(profile) = game.players.iter().find(|profile| profile.id == *player)
        {
            voices.queue(context, profile.game_profiles.gender, pattern);
        }
    }
}

fn pattern_voice(kind: PlayKind, mode: Mode) -> Option<CardPatternVoice> {
    match kind {
        PlayKind::Straight => Some(CardPatternVoice::Straight),
        PlayKind::Pairs => Some(CardPatternVoice::ConsecutivePairs),
        PlayKind::Airplane | PlayKind::AirplaneSingles | PlayKind::AirplanePairs => {
            Some(CardPatternVoice::Airplane)
        }
        PlayKind::Bomb => Some(CardPatternVoice::Bomb),
        PlayKind::Rocket if mode == Mode::Four => Some(CardPatternVoice::HeavenBomb),
        PlayKind::Rocket => Some(CardPatternVoice::Rocket),
        _ => None,
    }
}
