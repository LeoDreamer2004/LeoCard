use super::MahjongMatchProgress;
use super::{
    MahjongActionOutcome, MahjongDrawOrigin, MahjongGameError, MahjongGameState, MahjongPhase,
    validate_deck,
};
use crate::{MahjongMatchLength, MahjongRuleSet, MahjongTile, MahjongWind};
use leocard_game_common::PlayerId;

use std::collections::VecDeque;

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

    /// 推进一次真实发牌：前三轮依次给一家四张，随后每家一张，最后庄家跳一张。
    /// 返回 `true` 表示本次发完后已经进入出牌阶段。
    pub fn advance_deal(&mut self) -> Result<bool, MahjongGameError> {
        let MahjongPhase::Dealing { batch } = self.phase else {
            return Err(MahjongGameError::WrongPhase);
        };
        let (player, count) = match batch {
            0..=11 => {
                let offset = usize::from(batch % MahjongRuleSet::PLAYER_COUNT);
                (
                    PlayerId::from_index(
                        (self.dealer.index() + offset) % usize::from(MahjongRuleSet::PLAYER_COUNT),
                    ),
                    4,
                )
            }
            12..=15 => {
                let offset = usize::from(batch - 12);
                (
                    PlayerId::from_index(
                        (self.dealer.index() + offset) % usize::from(MahjongRuleSet::PLAYER_COUNT),
                    ),
                    1,
                )
            }
            16 => (self.dealer, 1),
            17 => return self.advance_initial_flower_replacement(),
            _ => return Err(MahjongGameError::WrongPhase),
        };
        let mut last = None;
        for _ in 0..count {
            let tile = self
                .wall
                .pop_front()
                .ok_or(MahjongGameError::InvalidDeckContents)?;
            self.players[player.index()].hand.push(tile);
            last = Some(tile);
        }
        if batch == 16 {
            let drawn = last.expect("庄家跳张批次必定发出一张牌");
            self.current_player = self.dealer;
            self.last_drawn = Some(drawn);
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
            let Some(position) = self.players[player.index()]
                .hand
                .iter()
                .position(|tile| tile.kind().is_flower())
            else {
                continue;
            };
            let flower = self.players[player.index()].hand.remove(position);
            self.players[player.index()].flowers.push(flower);
            let replacement = self
                .wall
                .pop_back()
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

    pub fn advance_flower_replacement(&mut self) -> Result<PlayerId, MahjongGameError> {
        let MahjongPhase::ReplacingFlower { player } = self.phase else {
            return Err(MahjongGameError::WrongPhase);
        };
        let flower = self
            .last_drawn
            .ok_or(MahjongGameError::InvalidDeckContents)?;
        if !flower.kind().is_flower() {
            return Err(MahjongGameError::InvalidDeckContents);
        }
        let position = self.players[player.index()]
            .hand
            .iter()
            .position(|tile| *tile == flower)
            .ok_or(MahjongGameError::InvalidDeckContents)?;
        self.players[player.index()].hand.remove(position);
        self.players[player.index()].flowers.push(flower);
        let Some(replacement) = self.wall.pop_back() else {
            self.last_drawn = None;
            self.finish_exhaustive_draw()?;
            return Ok(player);
        };
        self.players[player.index()].hand.push(replacement);
        self.last_drawn = Some(replacement);
        self.draw_origin = MahjongDrawOrigin::FlowerReplacement;
        if replacement.kind().is_flower() {
            self.phase = MahjongPhase::ReplacingFlower { player };
        } else {
            self.phase = MahjongPhase::Playing;
            self.sort_hands();
        }
        Ok(player)
    }

    pub(super) fn draw_replacement(
        &mut self,
        player: PlayerId,
        origin: MahjongDrawOrigin,
    ) -> Result<(MahjongTile, MahjongDrawOrigin), MahjongGameError> {
        let tile = self.wall.pop_back().ok_or(MahjongGameError::CannotKong)?;
        self.players[player.index()].hand.push(tile);
        self.current_player = player;
        self.last_drawn = Some(tile);
        self.draw_origin = origin;
        if tile.kind().is_flower() {
            self.phase = MahjongPhase::ReplacingFlower { player };
        } else {
            self.phase = MahjongPhase::Playing;
            self.sort_hands();
        }
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
        self.current_player = player;
        self.last_drawn = Some(tile);
        self.draw_origin = MahjongDrawOrigin::Normal;
        if tile.kind().is_flower() {
            self.phase = MahjongPhase::ReplacingFlower { player };
        } else {
            self.phase = MahjongPhase::Playing;
            self.sort_hands();
        }
        Ok(MahjongActionOutcome::Drew {
            player,
            tile,
            origin: MahjongDrawOrigin::Normal,
        })
    }
}
