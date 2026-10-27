use crate::app::presentation::{CardVoice, CardVoices};
use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_protocol::{GameEvent, QiGui523Event, ServerEvent};
use leocard_qigui523::{BombKind, QiGuiPlayKind, QiGuiRank};

pub(super) fn sync_qigui_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<CardVoices>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game());
    let Some(game) = game else {
        notifications.clear();
        return;
    };
    for notification in notifications.read() {
        let (player, voice) = match &notification.event {
            ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect {
                player,
                play,
            })) => (*player, pattern_voice(&play.kind)),
            ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::Passed { player })) => {
                (*player, Some(CardVoice::Pass))
            }
            _ => continue,
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if let Some(pattern) = voice
            && let Some(profile) = game.players.iter().find(|profile| profile.id == player)
        {
            voices.queue(context, profile.game_profiles.gender, pattern);
        }
    }
}

fn pattern_voice(kind: &QiGuiPlayKind) -> Option<CardVoice> {
    match kind {
        QiGuiPlayKind::Straight { .. } => Some(CardVoice::Straight),
        QiGuiPlayKind::ConsecutivePairs { .. } => Some(CardVoice::ConsecutivePairs),
        QiGuiPlayKind::Airplane { .. } => Some(CardVoice::Airplane),
        QiGuiPlayKind::Bomb(BombKind::OfAKind {
            rank: QiGuiRank::Joker,
            ..
        }) => Some(CardVoice::Rocket),
        QiGuiPlayKind::Bomb(_) => Some(CardVoice::Bomb),
        QiGuiPlayKind::HeavenBomb => Some(CardVoice::HeavenBomb),
        _ => None,
    }
}
