use super::super::MahjongBotObservation;
use super::features::encode_view;
use crate::{MahjongGameState, MahjongMeld, MahjongTile, MahjongWind};
use leocard_game_common::PlayerId;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct FeatureSnapshot {
    hand: Vec<MahjongTile>,
    melds: Vec<MahjongMeld>,
    winds: [MahjongWind; 2],
    features: Arc<[f32]>,
}

/// Preserve the published feature agent's refresh points: own-hand changes.
/// Claims use cached feature planes together with a current legal-action mask.
#[derive(Clone, Debug, Default)]
pub struct MahjongModelHistory {
    snapshots: [Option<FeatureSnapshot>; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct MahjongModelObservation {
    pub position: MahjongBotObservation,
    pub(super) features: Arc<[f32]>,
}

impl MahjongModelHistory {
    pub fn refresh(&mut self, game: &MahjongGameState) {
        for own in game.players() {
            let player = own.id();
            let winds = [
                game.seat_wind(player).expect("seated player"),
                game.prevalent_wind(),
            ];
            let slot = &mut self.snapshots[player.index()];
            if slot.as_ref().is_some_and(|cached| {
                cached.hand == own.hand() && cached.melds == own.melds() && cached.winds == winds
            }) {
                continue;
            }
            let public: Vec<_> = game
                .players()
                .iter()
                .filter_map(|other| game.public_player(player, other.id()))
                .collect();
            let features = encode_view(player, own.hand(), game.discards(), &public, winds).into();
            *slot = Some(FeatureSnapshot {
                hand: own.hand().to_vec(),
                melds: own.melds().to_vec(),
                winds,
                features,
            });
        }
    }

    pub fn observation(
        &mut self,
        game: &MahjongGameState,
        player: PlayerId,
    ) -> Option<MahjongModelObservation> {
        self.refresh(game);
        Some(MahjongModelObservation {
            position: MahjongBotObservation::from_game(game, player)?,
            features: self.snapshots[player.index()].as_ref()?.features.clone(),
        })
    }
}
