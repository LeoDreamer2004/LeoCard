use crate::app::presentation::{CardVoice, CardVoices};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_doudizhu::{Action, Mode, PlayKind, classify};
use leocard_protocol::{DoudizhuEvent, GameEvent, ServerEvent};

pub(super) fn sync_doudizhu_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<CardVoices>,
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
            action,
        })) = &notification.event
        else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        let voice = match action {
            Action::Play(cards) => classify(cards, game.rules.mode)
                .and_then(|play| pattern_voice(play.kind, game.rules.mode)),
            Action::Pass => Some(CardVoice::Pass),
            Action::Bid(score) => match score {
                0 => Some(CardVoice::BidZero),
                1 => Some(CardVoice::BidOne),
                2 => Some(CardVoice::BidTwo),
                3 => Some(CardVoice::BidThree),
                _ => None,
            },
        };
        if let Some(pattern) = voice
            && let Some(profile) = game.players.iter().find(|profile| profile.id == *player)
        {
            voices.queue(context, profile.game_profiles.gender, pattern);
        }
    }
}

fn pattern_voice(kind: PlayKind, mode: Mode) -> Option<CardVoice> {
    match kind {
        PlayKind::TripleSingle => Some(CardVoice::TripleSingle),
        PlayKind::TriplePair => Some(CardVoice::TriplePair),
        PlayKind::FourSingles | PlayKind::FourPairs => Some(CardVoice::FourTwo),
        PlayKind::Straight => Some(CardVoice::Straight),
        PlayKind::Pairs => Some(CardVoice::ConsecutivePairs),
        PlayKind::Airplane | PlayKind::AirplaneSingles | PlayKind::AirplanePairs => {
            Some(CardVoice::Airplane)
        }
        PlayKind::Bomb => Some(CardVoice::Bomb),
        PlayKind::Rocket if mode == Mode::Four => Some(CardVoice::HeavenBomb),
        PlayKind::Rocket => Some(CardVoice::Rocket),
        _ => None,
    }
}
