use super::{MahjongGameError, MahjongGameState, MahjongPhase};
use crate::{MahjongRuleSet, MahjongTile};
use leocard_game_common::PlayerId;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "[u8; 2]", into = "[u8; 2]"))]
pub struct MahjongWallBreak {
    dice: [u8; 2],
}

impl MahjongWallBreak {
    pub const STACKS_PER_SIDE: usize = 18;
    pub const TILE_COUNT: usize = Self::STACKS_PER_SIDE * MahjongRuleSet::PLAYER_COUNT as usize * 2;

    pub fn new(dice: [u8; 2]) -> Option<Self> {
        dice.iter()
            .all(|face| (1..=6).contains(face))
            .then_some(Self { dice })
    }

    pub const fn dice(self) -> [u8; 2] {
        self.dice
    }

    pub const fn total(self) -> u8 {
        self.dice[0] + self.dice[1]
    }

    /// Count seats from the dealer (one) in turn order, toward the lower seat.
    pub const fn side(self, dealer: PlayerId) -> PlayerId {
        let count = MahjongRuleSet::PLAYER_COUNT;
        PlayerId((dealer.0 % count + self.total() - 1) % count)
    }

    /// The wall runs clockwise from seat zero, opposite to player turn order.
    pub const fn first_tile_index(self, dealer: PlayerId) -> usize {
        let side =
            (MahjongRuleSet::PLAYER_COUNT - self.side(dealer).0) % MahjongRuleSet::PLAYER_COUNT;
        (side as usize * Self::STACKS_PER_SIDE + self.total() as usize) * 2
    }
}

#[cfg(feature = "serde")]
impl TryFrom<[u8; 2]> for MahjongWallBreak {
    type Error = &'static str;

    fn try_from(dice: [u8; 2]) -> Result<Self, Self::Error> {
        Self::new(dice).ok_or("Mahjong dice must have faces from one to six")
    }
}

#[cfg(feature = "serde")]
impl From<MahjongWallBreak> for [u8; 2] {
    fn from(opening: MahjongWallBreak) -> Self {
        opening.dice()
    }
}

impl MahjongGameState {
    pub fn break_wall(&mut self, opening: MahjongWallBreak) -> Result<(), MahjongGameError> {
        if self.wall_break.is_some() || !matches!(self.phase, MahjongPhase::Dealing { batch: 0 }) {
            return Err(MahjongGameError::WrongPhase);
        }
        self.wall.rotate_left(opening.first_tile_index(self.dealer));
        self.wall_break = Some(opening);
        Ok(())
    }

    pub const fn wall_break(&self) -> Option<MahjongWallBreak> {
        self.wall_break
    }

    pub const fn wall_tail_drawn(&self) -> u8 {
        self.wall_tail_drawn
    }

    pub(super) fn pop_wall_tail(&mut self) -> Option<MahjongTile> {
        let tile = self.wall.pop_back()?;
        self.wall_tail_drawn += 1;
        Some(tile)
    }
}
