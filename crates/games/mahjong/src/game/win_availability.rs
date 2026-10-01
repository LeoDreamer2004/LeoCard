use super::{GameError, GameState, MahjongWinAvailability, validate_player};
use crate::{MahjongPlayerId, MahjongScoreResult, MahjongTileKind, ScoreError, WinSource};
impl GameState {
    pub fn self_draw_win_availability(
        &self,
        player: MahjongPlayerId,
    ) -> Result<MahjongWinAvailability, GameError> {
        Ok(self
            .self_draw_score(player)?
            .map_or(MahjongWinAvailability::Unavailable, |score| {
                self.score_availability(&score)
            }))
    }

    pub fn claim_win_availability(
        &self,
        player: MahjongPlayerId,
        tile: MahjongTileKind,
        source: WinSource,
    ) -> Result<MahjongWinAvailability, GameError> {
        validate_player(player)?;
        match source {
            WinSource::Discard(from) | WinSource::RobbingKong(from) => {
                validate_player(from)?;
                if from == player {
                    return Ok(MahjongWinAvailability::Unavailable);
                }
            }
            _ => return Err(GameError::InvalidClaim),
        }
        if self.players[player.0].dead_hand {
            return Ok(MahjongWinAvailability::Unavailable);
        }
        match self.score_for(player, tile, source) {
            Ok(score) => Ok(self.score_availability(&score)),
            Err(GameError::Score(ScoreError::NotComplete)) => {
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
