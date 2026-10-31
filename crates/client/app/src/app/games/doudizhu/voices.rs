use crate::app::presentation::{CardVoice, CardVoices};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_doudizhu::{DoudizhuAction, DoudizhuMode, DoudizhuPlay, DoudizhuPlayKind, classify};
use leocard_protocol::{DoudizhuEvent, GameEvent, GameSnapshot, ServerEvent};

pub(super) fn sync_doudizhu_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<CardVoices>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::doudizhu)
    }) else {
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
            DoudizhuAction::Play(cards) => {
                classify(cards, game.rules).and_then(|play| pattern_voice(&play, game.rules.mode))
            }
            DoudizhuAction::Pass => Some(CardVoice::Pass),
            DoudizhuAction::Bid(score) => match score {
                0 => Some(CardVoice::BidZero),
                1 => Some(CardVoice::BidOne),
                2 => Some(CardVoice::BidTwo),
                3 => Some(CardVoice::BidThree),
                _ => None,
            },
        };
        if let Some(pattern) = voice
            && let Some(profile) = game
                .players
                .iter()
                .find(|profile| profile.player.id == *player)
        {
            voices.announce(context, profile.player.game_profiles.gender, pattern);
        }
    }
}

fn pattern_voice(play: &DoudizhuPlay, mode: DoudizhuMode) -> Option<CardVoice> {
    match play.kind {
        DoudizhuPlayKind::Single => play
            .cards
            .first()
            .map(|card| CardVoice::Single((*card).into())),
        DoudizhuPlayKind::Pair => play
            .cards
            .first()
            .map(|card| CardVoice::Pair((*card).into())),
        DoudizhuPlayKind::Triple => Some(CardVoice::Triple),
        DoudizhuPlayKind::TripleSingle => Some(CardVoice::TripleSingle),
        DoudizhuPlayKind::TriplePair => Some(CardVoice::TriplePair),
        DoudizhuPlayKind::FourSingles | DoudizhuPlayKind::FourPairs => Some(CardVoice::FourTwo),
        DoudizhuPlayKind::Straight => Some(CardVoice::Straight),
        DoudizhuPlayKind::Pairs => Some(CardVoice::ConsecutivePairs),
        DoudizhuPlayKind::Airplane => Some(CardVoice::TripleSequence),
        DoudizhuPlayKind::AirplaneSingles | DoudizhuPlayKind::AirplanePairs => {
            Some(CardVoice::Airplane)
        }
        DoudizhuPlayKind::Bomb => Some(CardVoice::Bomb),
        DoudizhuPlayKind::Rocket if mode == DoudizhuMode::Four => Some(CardVoice::HeavenBomb),
        DoudizhuPlayKind::Rocket => Some(CardVoice::Rocket),
    }
}
