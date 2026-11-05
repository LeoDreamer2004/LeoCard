use crate::app::presentation::{CardVoice, CardVoices};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_guandan::{GuandanPlay, GuandanPlayKind, classify};
use leocard_protocol::{GameEvent, GameSnapshot, GuandanEvent, GuandanPublicAction, ServerEvent};

pub(super) fn sync_guandan_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<CardVoices>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::guandan)
    }) else {
        notifications.clear();
        return;
    };
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::Guandan(GuandanEvent::ActionApplied {
            player,
            action,
        })) = &notification.event
        else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if context.match_id != game.match_id || context.hand_index != Some(game.hand_number) {
            continue;
        }
        let voice = match action {
            GuandanPublicAction::Play(cards) => classify(cards, game.level)
                .ok()
                .and_then(|play| pattern_voice(&play)),
            GuandanPublicAction::Pass => Some(CardVoice::Pass),
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

fn pattern_voice(play: &GuandanPlay) -> Option<CardVoice> {
    match play.kind {
        GuandanPlayKind::Single | GuandanPlayKind::Pair => {
            // The classified rank may differ from a heart-level wildcard's printed rank.
            let face = play.cards.iter().find(|card| card.rank == play.value)?;
            Some(if play.kind == GuandanPlayKind::Single {
                CardVoice::Single((*face).into())
            } else {
                CardVoice::Pair((*face).into())
            })
        }
        GuandanPlayKind::Triple => Some(CardVoice::Triple),
        GuandanPlayKind::FullHouse => Some(CardVoice::TriplePair),
        GuandanPlayKind::Straight => Some(CardVoice::Straight),
        GuandanPlayKind::ConsecutivePairs => Some(CardVoice::ThreePairs),
        GuandanPlayKind::ConsecutiveTriples => Some(CardVoice::SteelPlate),
        GuandanPlayKind::Bomb(_) => Some(CardVoice::Bomb),
        GuandanPlayKind::StraightFlush => Some(CardVoice::StraightFlush),
        GuandanPlayKind::JokerBomb => Some(CardVoice::FourJokers),
    }
}
