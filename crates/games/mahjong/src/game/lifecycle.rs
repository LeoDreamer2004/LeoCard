use super::{
    MahjongActionOutcome, MahjongDrawOrigin, MahjongGameError, MahjongGameState,
    MahjongMatchProgress, MahjongPhase, validate_deck, validate_player,
};
use crate::{MahjongMatchLength, MahjongRuleSet, MahjongTile, MahjongWind};
use leocard_game_common::PlayerId;
use std::collections::VecDeque;

const PACK_SIZE: usize = 4;
const PACK_BATCHES: u8 = 3 * MahjongRuleSet::PLAYER_COUNT;
const DEAL_BATCHES: u8 = PACK_BATCHES + MahjongRuleSet::PLAYER_COUNT;
const JUMP_OFFSET: usize = 4;

/// Index from the original wall head of the jumped tile while it leaves a hole ahead of the draw.
pub const fn mahjong_deal_wall_gap(next_batch: u8) -> Option<usize> {
    if next_batch > PACK_BATCHES && next_batch < DEAL_BATCHES {
        Some(PACK_BATCHES as usize * PACK_SIZE + JUMP_OFFSET)
    } else {
        None
    }
}

impl MahjongGameState {
    /// 结算界面内全员准备后开始下一盘。单局模式重置累计分，但继续轮庄和轮圈风。
    pub fn start_next_hand(&mut self, deck: Vec<MahjongTile>) -> Result<(), MahjongGameError> {
        let MahjongPhase::Finished(result) = &self.phase else {
            return Err(MahjongGameError::WrongPhase);
        };
        validate_deck(&deck)?;
        let completed_match = result.match_complete;
        if self.rules.match_length == MahjongMatchLength::SingleHand {
            self.match_scores = [0; MahjongRuleSet::PLAYER_COUNT as usize];
            self.sequence_index = (self.sequence_index + 1) % 16;
            self.match_progress = MahjongMatchProgress::default();
        } else if completed_match {
            self.match_scores = [0; MahjongRuleSet::PLAYER_COUNT as usize];
            self.sequence_index = 0;
            self.match_progress = MahjongMatchProgress::default();
        } else {
            self.sequence_index += 1;
        }
        self.dealer = PlayerId::from_index(
            (self.initial_dealer.index() + usize::from(self.sequence_index))
                % usize::from(MahjongRuleSet::PLAYER_COUNT),
        );
        self.prevalent_wind = MahjongWind::ALL[usize::from(self.sequence_index / 4)];
        self.wall = VecDeque::from(deck);
        self.wall_break = None;
        self.wall_tail_drawn = 0;
        self.discards.clear();
        self.hand_deltas = [0; MahjongRuleSet::PLAYER_COUNT as usize];
        self.current_player = self.dealer;
        self.last_drawn = None;
        self.draw_origin = MahjongDrawOrigin::Normal;
        self.phase = MahjongPhase::Dealing { batch: 0 };
        for player in &mut self.players {
            player.hand.clear();
            player.melds.clear();
            player.flowers.clear();
            player.dead_hand = false;
            player.hand_revealed = false;
        }
        Ok(())
    }

    /// 前三轮每家四张，之后庄家同时摸第一、第三垛的上层，另外三家依次摸墙头一张。
    /// 返回 `true` 表示本次发完后已经进入出牌阶段。
    pub fn advance_deal(&mut self) -> Result<bool, MahjongGameError> {
        let MahjongPhase::Dealing { batch } = self.phase else {
            return Err(MahjongGameError::WrongPhase);
        };
        let (offset, count) = match batch {
            0..PACK_BATCHES => (batch % MahjongRuleSet::PLAYER_COUNT, PACK_SIZE),
            PACK_BATCHES..DEAL_BATCHES => (batch - PACK_BATCHES, 1),
            DEAL_BATCHES => return self.advance_initial_flower_replacement(),
            _ => return Err(MahjongGameError::WrongPhase),
        };
        let player = PlayerId((self.dealer.0 + offset) % MahjongRuleSet::PLAYER_COUNT);
        let jumped = if batch == PACK_BATCHES {
            // The lower tile in the first stack and both tiles in the second stay at the front.
            Some(
                self.wall
                    .remove(JUMP_OFFSET)
                    .ok_or(MahjongGameError::InvalidDeckContents)?,
            )
        } else {
            None
        };
        let hand = &mut self.players[player.index()].hand;
        for _ in 0..count {
            let tile = self
                .wall
                .pop_front()
                .ok_or(MahjongGameError::InvalidDeckContents)?;
            hand.push(tile);
        }
        if let Some(tile) = jumped {
            hand.push(tile);
            self.last_drawn = Some(tile);
            self.draw_origin = MahjongDrawOrigin::Normal;
        }
        self.phase = MahjongPhase::Dealing { batch: batch + 1 };
        Ok(false)
    }

    fn advance_initial_flower_replacement(&mut self) -> Result<bool, MahjongGameError> {
        for offset in 0..usize::from(MahjongRuleSet::PLAYER_COUNT) {
            let player = PlayerId::from_index(
                (self.dealer.index() + offset) % usize::from(MahjongRuleSet::PLAYER_COUNT),
            );
            if !self.auto_flower_enabled(player) {
                continue;
            }
            let Some(position) = self.flower_position(player) else {
                continue;
            };
            let flower = self.players[player.index()].hand.remove(position);
            self.players[player.index()].flowers.push(flower);
            let replacement = self
                .pop_wall_tail()
                .ok_or(MahjongGameError::InvalidDeckContents)?;
            self.players[player.index()].hand.push(replacement);
            if self.last_drawn == Some(flower) {
                self.last_drawn = Some(replacement);
                self.draw_origin = MahjongDrawOrigin::FlowerReplacement;
            }
            return Ok(false);
        }
        self.sort_hands();
        self.phase = MahjongPhase::Playing;
        Ok(true)
    }

    pub fn auto_flower_enabled(&self, player: PlayerId) -> bool {
        self.auto_flower
            .get(player.index())
            .copied()
            .unwrap_or(false)
    }

    pub fn set_auto_flower(
        &mut self,
        player: PlayerId,
        enabled: bool,
    ) -> Result<bool, MahjongGameError> {
        validate_player(player)?;
        if matches!(self.phase, MahjongPhase::Finished(_)) {
            return Err(MahjongGameError::WrongPhase);
        }
        if self.auto_flower[player.index()] == enabled {
            return Ok(false);
        }
        self.auto_flower[player.index()] = enabled;
        if self.current_player == player
            && matches!(
                self.phase,
                MahjongPhase::Playing | MahjongPhase::ReplacingFlower { .. }
            )
        {
            self.prepare_player_turn(player);
        }
        Ok(true)
    }

    pub(super) fn flower_position(&self, player: PlayerId) -> Option<usize> {
        self.players[player.index()]
            .hand
            .iter()
            .position(|tile| tile.kind().is_flower())
    }

    pub(super) fn prepare_player_turn(&mut self, player: PlayerId) {
        self.current_player = player;
        if self.auto_flower_enabled(player) && self.flower_position(player).is_some() {
            self.phase = MahjongPhase::ReplacingFlower { player };
        } else {
            self.phase = MahjongPhase::Playing;
            self.sort_hands();
        }
    }

    pub fn advance_flower_replacement(&mut self) -> Result<PlayerId, MahjongGameError> {
        let MahjongPhase::ReplacingFlower { player } = self.phase else {
            return Err(MahjongGameError::WrongPhase);
        };
        let position = self
            .flower_position(player)
            .ok_or(MahjongGameError::InvalidDeckContents)?;
        let flower = self.players[player.index()].hand.remove(position);
        self.players[player.index()].flowers.push(flower);
        let Some(replacement) = self.pop_wall_tail() else {
            self.last_drawn = None;
            self.finish_exhaustive_draw()?;
            return Ok(player);
        };
        self.players[player.index()].hand.push(replacement);
        self.last_drawn = Some(replacement);
        self.draw_origin = MahjongDrawOrigin::FlowerReplacement;
        self.prepare_player_turn(player);
        Ok(player)
    }

    pub(super) fn draw_replacement(
        &mut self,
        player: PlayerId,
        origin: MahjongDrawOrigin,
    ) -> Result<(MahjongTile, MahjongDrawOrigin), MahjongGameError> {
        let tile = self.pop_wall_tail().ok_or(MahjongGameError::CannotKong)?;
        self.players[player.index()].hand.push(tile);
        self.last_drawn = Some(tile);
        self.draw_origin = origin;
        self.prepare_player_turn(player);
        Ok((tile, origin))
    }

    pub(super) fn draw_normal(
        &mut self,
        player: PlayerId,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        let Some(tile) = self.wall.pop_front() else {
            return self.finish_exhaustive_draw();
        };
        self.players[player.index()].hand.push(tile);
        self.last_drawn = Some(tile);
        self.draw_origin = MahjongDrawOrigin::Normal;
        self.prepare_player_turn(player);
        Ok(MahjongActionOutcome::Drew {
            player,
            tile,
            origin: MahjongDrawOrigin::Normal,
        })
    }
}
