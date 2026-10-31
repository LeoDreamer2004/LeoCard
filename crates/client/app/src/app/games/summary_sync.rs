//! 将当前游戏快照投影为公共结算演出描述。

use super::{blackjack, doudizhu, mahjong, mc_guess, qigui523, texas, uno};
use crate::app::presentation::{SummaryDescriptor, SummaryPlayback};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::GameSnapshot;

fn game_summary_descriptor(game: &GameSnapshot) -> Option<SummaryDescriptor> {
    match game {
        GameSnapshot::Doudizhu(game) => doudizhu::summary::descriptor(game),
        GameSnapshot::Blackjack(game) => blackjack::blackjack_summary_descriptor(game),
        GameSnapshot::QiGui523(game) => qigui523::summary::qigui523_summary_descriptor(game),
        GameSnapshot::Texas(game) => texas::settlement::texas_summary_descriptor(game),
        GameSnapshot::Shengji(_) => None,
        GameSnapshot::McGuess(game) => mc_guess::summary::guess_summary_descriptor(game),
        GameSnapshot::Uno(game) => uno::settlement::uno_summary_descriptor(game),
        GameSnapshot::Mahjong(game) => mahjong::settlement::mahjong_summary_descriptor(game),
    }
}

/// Project gameplay decisions before the common summary renderer advances its clock.
pub(crate) fn sync_summary_playback(
    client: Option<Res<ClientResource>>,
    mahjong_ui: Res<mahjong::MahjongUiState>,
    guess_scores: Res<mc_guess::GuessScorePlayback>,
    blackjack_playback: Res<blackjack::BlackjackPlayback>,
    guess_feedback: Res<mc_guess::GuessFeedback>,
    mut playback: ResMut<SummaryPlayback>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot());
    playback.descriptor = game.and_then(game_summary_descriptor);
    playback.pause_at = match game {
        Some(GameSnapshot::Blackjack(_)) if blackjack_playback.busy() => Some(-0.01),
        Some(GameSnapshot::Mahjong(game)) => {
            mahjong::settlement::mahjong_summary_pause(game, &mahjong_ui)
        }
        Some(GameSnapshot::McGuess(_)) if guess_scores.busy() || guess_feedback.busy() => {
            Some(-0.01)
        }
        _ => None,
    };
}
