use super::*;

impl GameState {
    /// 结算界面内全员准备后开始下一盘。单局模式重置累计分，但继续轮庄和轮圈风。
    pub fn start_next_hand(&mut self, deck: Vec<MahjongTile>) -> Result<(), GameError> {
        let Phase::Finished(result) = &self.phase else {
            return Err(GameError::WrongPhase);
        };
        validate_deck(&deck)?;
        let completed_match = result.match_complete;
        if self.rules.match_length == MahjongMatchLength::SingleHand {
            self.match_scores = [0; MahjongRuleSet::PLAYER_COUNT];
            self.sequence_index = (self.sequence_index + 1) % 16;
            self.hands_in_match = 0;
        } else if completed_match {
            self.match_scores = [0; MahjongRuleSet::PLAYER_COUNT];
            self.sequence_index = 0;
            self.hands_in_match = 0;
        } else {
            self.sequence_index += 1;
        }
        self.dealer = MahjongPlayerId(
            (self.initial_dealer.0 + usize::from(self.sequence_index))
                % MahjongRuleSet::PLAYER_COUNT,
        );
        self.prevalent_wind = MahjongWind::ALL[usize::from(self.sequence_index / 4)];
        self.wall = VecDeque::from(deck);
        self.discards.clear();
        self.hand_deltas = [0; MahjongRuleSet::PLAYER_COUNT];
        self.current_player = self.dealer;
        self.last_drawn = None;
        self.draw_origin = MahjongDrawOrigin::Normal;
        self.phase = Phase::Dealing { batch: 0 };
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
    pub fn advance_deal(&mut self) -> Result<bool, GameError> {
        let Phase::Dealing { batch } = self.phase else {
            return Err(GameError::WrongPhase);
        };
        let (player, count) = match batch {
            0..=11 => {
                let offset = usize::from(batch % MahjongRuleSet::PLAYER_COUNT as u8);
                (
                    MahjongPlayerId((self.dealer.0 + offset) % MahjongRuleSet::PLAYER_COUNT),
                    4,
                )
            }
            12..=15 => {
                let offset = usize::from(batch - 12);
                (
                    MahjongPlayerId((self.dealer.0 + offset) % MahjongRuleSet::PLAYER_COUNT),
                    1,
                )
            }
            16 => (self.dealer, 1),
            17 => return self.advance_initial_flower_replacement(),
            _ => return Err(GameError::WrongPhase),
        };
        let mut last = None;
        for _ in 0..count {
            let tile = self
                .wall
                .pop_front()
                .ok_or(GameError::InvalidDeckContents)?;
            self.players[player.0].hand.push(tile);
            last = Some(tile);
        }
        if batch == 16 {
            let drawn = last.expect("庄家跳张批次必定发出一张牌");
            self.current_player = self.dealer;
            self.last_drawn = Some(drawn);
            self.draw_origin = MahjongDrawOrigin::Normal;
        }
        self.phase = Phase::Dealing { batch: batch + 1 };
        Ok(false)
    }

    fn advance_initial_flower_replacement(&mut self) -> Result<bool, GameError> {
        for offset in 0..MahjongRuleSet::PLAYER_COUNT {
            let player = MahjongPlayerId((self.dealer.0 + offset) % MahjongRuleSet::PLAYER_COUNT);
            let Some(position) = self.players[player.0]
                .hand
                .iter()
                .position(|tile| tile.kind().is_flower())
            else {
                continue;
            };
            let flower = self.players[player.0].hand.remove(position);
            self.players[player.0].flowers.push(flower);
            let replacement = self.wall.pop_back().ok_or(GameError::InvalidDeckContents)?;
            self.players[player.0].hand.push(replacement);
            if self.last_drawn == Some(flower) {
                self.last_drawn = Some(replacement);
                self.draw_origin = MahjongDrawOrigin::FlowerReplacement;
            }
            return Ok(false);
        }
        self.sort_hands();
        self.phase = Phase::Playing;
        Ok(true)
    }

    pub fn advance_flower_replacement(&mut self) -> Result<MahjongPlayerId, GameError> {
        let Phase::ReplacingFlower { player } = self.phase else {
            return Err(GameError::WrongPhase);
        };
        let flower = self.last_drawn.ok_or(GameError::InvalidDeckContents)?;
        if !flower.kind().is_flower() {
            return Err(GameError::InvalidDeckContents);
        }
        let position = self.players[player.0]
            .hand
            .iter()
            .position(|tile| *tile == flower)
            .ok_or(GameError::InvalidDeckContents)?;
        self.players[player.0].hand.remove(position);
        self.players[player.0].flowers.push(flower);
        let replacement = self.wall.pop_back().ok_or(GameError::InvalidDeckContents)?;
        self.players[player.0].hand.push(replacement);
        self.last_drawn = Some(replacement);
        self.draw_origin = MahjongDrawOrigin::FlowerReplacement;
        if replacement.kind().is_flower() {
            self.phase = Phase::ReplacingFlower { player };
        } else {
            self.phase = Phase::Playing;
            self.sort_hands();
        }
        Ok(player)
    }

    pub(super) fn draw_replacement(
        &mut self,
        player: MahjongPlayerId,
        origin: MahjongDrawOrigin,
    ) -> Result<(MahjongTile, MahjongDrawOrigin), GameError> {
        let tile = self.wall.pop_back().ok_or(GameError::CannotKong)?;
        self.players[player.0].hand.push(tile);
        self.current_player = player;
        self.last_drawn = Some(tile);
        self.draw_origin = origin;
        if tile.kind().is_flower() {
            self.phase = Phase::ReplacingFlower { player };
        } else {
            self.phase = Phase::Playing;
            self.sort_hands();
        }
        Ok((tile, origin))
    }

    pub(super) fn draw_normal(
        &mut self,
        player: MahjongPlayerId,
    ) -> Result<ActionOutcome, GameError> {
        let Some(tile) = self.wall.pop_front() else {
            return self.finish_exhaustive_draw();
        };
        self.players[player.0].hand.push(tile);
        self.current_player = player;
        self.last_drawn = Some(tile);
        self.draw_origin = MahjongDrawOrigin::Normal;
        if tile.kind().is_flower() {
            self.phase = Phase::ReplacingFlower { player };
        } else {
            self.phase = Phase::Playing;
            self.sort_hands();
        }
        Ok(ActionOutcome::Drew {
            player,
            tile,
            origin: MahjongDrawOrigin::Normal,
        })
    }
}
