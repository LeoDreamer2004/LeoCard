use crate::app::presentation::{CardPatternVoice, CardPatternVoices};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_protocol::{GameEvent, QiGui523Event, ServerEvent};
use leocard_qigui523::{BombKind, QiGuiPlayKind, QiGuiRank};

pub(super) fn sync_qigui_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<CardPatternVoices>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game());
    let Some(game) = game else {
        notifications.clear();
        return;
    };
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect { player, play })) =
            &notification.event
        else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if let Some(pattern) = pattern_voice(&play.kind)
            && let Some(profile) = game.players.iter().find(|profile| profile.id == *player)
        {
            voices.queue(context, profile.game_profiles.gender, pattern);
        }
    }
}

fn pattern_voice(kind: &QiGuiPlayKind) -> Option<CardPatternVoice> {
    match kind {
        QiGuiPlayKind::Straight { .. } => Some(CardPatternVoice::Straight),
        QiGuiPlayKind::ConsecutivePairs { .. } => Some(CardPatternVoice::ConsecutivePairs),
        QiGuiPlayKind::Airplane { .. } => Some(CardPatternVoice::Airplane),
        QiGuiPlayKind::Bomb(BombKind::OfAKind {
            rank: QiGuiRank::Joker,
            ..
        }) => Some(CardPatternVoice::Rocket),
        QiGuiPlayKind::Bomb(_) => Some(CardPatternVoice::Bomb),
        QiGuiPlayKind::HeavenBomb => Some(CardPatternVoice::HeavenBomb),
        _ => None,
    }
}
