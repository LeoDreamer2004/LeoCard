//! 将当前游戏快照投影为公共结算演出描述。

use super::{mahjong, qigui523, texas_holdem, uno};
use crate::app::presentation::{SummaryDescriptor, SummaryPlayback};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::GameSnapshot;

pub(crate) fn game_summary_descriptor(game: &GameSnapshot) -> Option<SummaryDescriptor> {
    match game {
        GameSnapshot::QiGui523(game) => qigui523::summary::qigui523_summary_descriptor(game),
        GameSnapshot::TexasHoldem(game) => {
            texas_holdem::settlement::texas_holdem_summary_descriptor(game)
        }
        GameSnapshot::Shengji(_) => None,
        GameSnapshot::Uno(game) => uno::settlement::uno_summary_descriptor(game),
        GameSnapshot::Mahjong(game) => mahjong::settlement::mahjong_summary_descriptor(game),
    }
}

/// Project gameplay decisions before the common summary renderer advances its clock.
pub(super) fn sync_summary_playback(
    client: Option<Res<ClientResource>>,
    mahjong_ui: Res<mahjong::MahjongUiState>,
    mut playback: ResMut<SummaryPlayback>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot());
    playback.descriptor = game.and_then(game_summary_descriptor);
    playback.pause_at = match game {
        Some(GameSnapshot::Mahjong(game)) => {
            mahjong::settlement::mahjong_summary_pause(game, &mahjong_ui)
        }
        _ => None,
    };
}
