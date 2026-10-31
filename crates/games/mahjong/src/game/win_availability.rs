use super::{MahjongGameError, MahjongGameState, MahjongWinAvailability, validate_player};
use crate::{MahjongScoreError, MahjongScoreResult, MahjongTileKind, MahjongWinSource};
use leocard_game_common::PlayerId;

impl MahjongGameState {
    pub fn self_draw_win_availability(
        &self,
        player: PlayerId,
    ) -> Result<MahjongWinAvailability, MahjongGameError> {
        Ok(self
            .self_draw_score(player)?
            .map_or(MahjongWinAvailability::Unavailable, |score| {
                self.score_availability(&score)
            }))
    }

    pub fn claim_win_availability(
        &self,
        player: PlayerId,
        tile: MahjongTileKind,
        source: MahjongWinSource,
    ) -> Result<MahjongWinAvailability, MahjongGameError> {
        validate_player(player)?;
        match source {
            MahjongWinSource::Discard(from) | MahjongWinSource::RobbingKong(from) => {
                validate_player(from)?;
                if from == player {
                    return Ok(MahjongWinAvailability::Unavailable);
                }
            }
            _ => return Err(MahjongGameError::InvalidClaim),
        }
        if self.players[player.index()].dead_hand {
            return Ok(MahjongWinAvailability::Unavailable);
        }
        match self.score_for(player, tile, source) {
            Ok(score) => Ok(self.score_availability(&score)),
            Err(MahjongGameError::Score(MahjongScoreError::NotComplete)) => {
                Ok(MahjongWinAvailability::Unavailable)
            }
            Err(error) => Err(error),
        }
    }

    fn score_availability(&self, score: &MahjongScoreResult) -> MahjongWinAvailability {
        if self.is_legal_score(score) {
            MahjongWinAvailability::Legal
        } else {
            MahjongWinAvailability::InsufficientFan {
                points: score.points_without_flowers,
            }
        }
    }
}
