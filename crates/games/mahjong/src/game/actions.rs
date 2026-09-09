use super::{
    ActionOutcome, ClaimPriority, ClaimTrigger, Discard, GameError, GameState, MahjongClaim,
    MahjongClaimOption, MahjongDrawOrigin, PendingClaim, Phase, WinRecord, claim_option_priority,
    claim_priority, players_after, validate_player,
};
use crate::{
    MahjongMeldKind, MahjongPlayerId, MahjongRuleSet, MahjongScoreResult, MahjongTile,
    MahjongTileKind, Meld, WinSource, is_complete_hand,
};
use std::array;

impl GameState {
    pub fn discard(
        &mut self,
        player: MahjongPlayerId,
        tile: MahjongTile,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        let position = self.players[player.0]
            .hand
            .iter()
            .position(|held| *held == tile)
            .ok_or(GameError::TileNotInHand(tile))?;
        self.players[player.0].hand.remove(position);
        self.last_drawn = None;
        let discard_index = self.discards.len();
        self.discards.push(Discard {
            player,
            tile,
            claimed_by: None,
        });
        let pending = self.pending_for_discard(discard_index, player, tile)?;
        if pending.waiting_for().is_empty() {
            return self.advance_after_unclaimed_discard(player);
        }
        self.phase = Phase::WaitingForClaims(pending);
        Ok(ActionOutcome::Discarded { player, tile })
    }

    pub fn respond_to_claim(
        &mut self,
        player: MahjongPlayerId,
        claim: MahjongClaim,
    ) -> Result<ActionOutcome, GameError> {
        validate_player(player)?;
        {
            let Phase::WaitingForClaims(pending) = &mut self.phase else {
                return Err(GameError::WrongPhase);
            };
            if pending.options[player.0].is_empty() {
                return Err(GameError::InvalidClaim);
            }
            if pending.responses[player.0].is_some() {
                return Err(GameError::AlreadyResponded);
            }
            if let Some(option) = claim.option()
                && !pending.options[player.0].contains(&option)
            {
                return Err(GameError::InvalidClaim);
            }
            pending.responses[player.0] = Some(claim);
        }
        self.cancel_lower_priority_claims()?;
        let Phase::WaitingForClaims(pending) = &self.phase else {
            unreachable!("priority cancellation does not leave the claim phase");
        };
        if !pending.waiting_for().is_empty() {
            return Ok(ActionOutcome::ClaimRecorded { player });
        }
        self.resolve_pending_claim()
    }

    fn cancel_lower_priority_claims(&mut self) -> Result<(), GameError> {
        let Phase::WaitingForClaims(pending) = &self.phase else {
            return Err(GameError::WrongPhase);
        };
        let pending = pending.clone();
        let source = pending.source_player();
        let mut selected: Option<(ClaimPriority, bool)> = None;
        for player in players_after(source) {
            let Some(claim) = pending.responses[player.0] else {
                continue;
            };
            if claim == MahjongClaim::Pass
                || (claim == MahjongClaim::Win && !self.legal_claim_win_available(player)?)
            {
                continue;
            }
            let priority = claim_priority(source, player, claim);
            if selected.is_none_or(|(current, _)| priority > current) {
                selected = Some((priority, claim == MahjongClaim::Win));
            }
        }
        let Some((selected, selected_is_win)) = selected else {
            return Ok(());
        };
        let mut cancel = [false; MahjongRuleSet::PLAYER_COUNT];
        for player in players_after(source) {
            if pending.responses[player.0].is_some() || pending.options[player.0].is_empty() {
                continue;
            }
            let can_still_matter = pending.options[player.0].iter().copied().any(|option| {
                if self.rules.multiple_winners
                    && selected_is_win
                    && option == MahjongClaimOption::Win
                {
                    return true;
                }
                claim_option_priority(source, player, option) > selected
            });
            cancel[player.0] = !can_still_matter;
        }
        let Phase::WaitingForClaims(pending) = &mut self.phase else {
            unreachable!("claim phase was only read while priorities were calculated");
        };
        for (index, cancel) in cancel.into_iter().enumerate() {
            if cancel {
                pending.responses[index] = Some(MahjongClaim::Pass);
            }
        }
        Ok(())
    }

    pub fn declare_self_draw(
        &mut self,
        player: MahjongPlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        if self.players[player.0].dead_hand {
            return Err(GameError::CannotWin);
        }
        let winning = self.last_drawn.ok_or(GameError::CannotWin)?;
        let source = match self.draw_origin {
            MahjongDrawOrigin::Normal => WinSource::SelfDraw,
            MahjongDrawOrigin::KongReplacement => WinSource::KongReplacement,
            MahjongDrawOrigin::FlowerReplacement => WinSource::FlowerReplacement,
        };
        let score = self.score_for(player, winning.kind(), source)?;
        if self.is_legal_score(&score) {
            return self.finish_with_winners(vec![WinRecord {
                player,
                from: None,
                winning_tile: winning,
                score,
            }]);
        }
        if self.rules.false_win {
            let penalty = self.apply_false_win(player);
            return Ok(ActionOutcome::FalseWin {
                player,
                deltas: penalty,
            });
        }
        Err(GameError::CannotWin)
    }

    pub fn self_draw_available(&self, player: MahjongPlayerId) -> Result<bool, GameError> {
        Ok(self
            .self_draw_score(player)?
            .is_some_and(|score| self.is_legal_score(&score) || self.rules.false_win))
    }

    /// 供自动玩家判断真正合法的自摸，避免“允许错和”开启时主动报错和。
    pub fn legal_self_draw_available(&self, player: MahjongPlayerId) -> Result<bool, GameError> {
        Ok(self
            .self_draw_score(player)?
            .is_some_and(|score| self.is_legal_score(&score)))
    }

    /// 判断当前响应窗口中的和牌是否真正达到起和要求。
    pub fn legal_claim_win_available(&self, player: MahjongPlayerId) -> Result<bool, GameError> {
        validate_player(player)?;
        let Phase::WaitingForClaims(pending) = &self.phase else {
            return Ok(false);
        };
        if !pending.options[player.0].contains(&MahjongClaimOption::Win) {
            return Ok(false);
        }
        let (tile, source) = match pending.trigger {
            ClaimTrigger::Discard { from, tile, .. } => (tile, WinSource::Discard(from)),
            ClaimTrigger::AddedKong { player, tile, .. } => (tile, WinSource::RobbingKong(player)),
        };
        let score = self.score_for(player, tile.kind(), source)?;
        Ok(self.is_legal_score(&score))
    }

    fn self_draw_score(
        &self,
        player: MahjongPlayerId,
    ) -> Result<Option<MahjongScoreResult>, GameError> {
        validate_player(player)?;
        if !matches!(self.phase, Phase::Playing)
            || self.current_player != player
            || self.players[player.0].dead_hand
        {
            return Ok(None);
        }
        let Some(winning) = self.last_drawn else {
            return Ok(None);
        };
        let concealed: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .map(|tile| tile.kind())
            .collect();
        if !is_complete_hand(&concealed, &self.players[player.0].melds) {
            return Ok(None);
        }
        let source = match self.draw_origin {
            MahjongDrawOrigin::Normal => WinSource::SelfDraw,
            MahjongDrawOrigin::KongReplacement => WinSource::KongReplacement,
            MahjongDrawOrigin::FlowerReplacement => WinSource::FlowerReplacement,
        };
        let score = self.score_for(player, winning.kind(), source)?;
        Ok(Some(score))
    }

    pub fn declare_concealed_kong(
        &mut self,
        player: MahjongPlayerId,
        tile: MahjongTileKind,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        if self.wall.is_empty() || tile.is_flower() {
            return Err(GameError::CannotKong);
        }
        let positions: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .enumerate()
            .filter(|(_, held)| held.kind() == tile)
            .map(|(index, _)| index)
            .collect();
        if positions.len() != 4 {
            return Err(GameError::CannotKong);
        }
        for position in positions.into_iter().rev() {
            self.players[player.0].hand.remove(position);
        }
        self.players[player.0]
            .melds
            .push(Meld::concealed_kong(tile));
        self.draw_replacement(player, MahjongDrawOrigin::KongReplacement)?;
        Ok(ActionOutcome::KongDeclared {
            player,
            tile,
            added: false,
        })
    }

    pub fn declare_added_kong(
        &mut self,
        player: MahjongPlayerId,
        tile: MahjongTile,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        if self.wall.is_empty() {
            return Err(GameError::CannotKong);
        }
        if !self.players[player.0].hand.contains(&tile) {
            return Err(GameError::TileNotInHand(tile));
        }
        let Some(meld_index) = self.players[player.0]
            .melds
            .iter()
            .position(|meld| meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind())
        else {
            return Err(GameError::CannotKong);
        };
        let mut pending = PendingClaim {
            trigger: ClaimTrigger::AddedKong {
                player,
                tile,
                meld_index,
            },
            options: array::from_fn(|_| Vec::new()),
            responses: [None; MahjongRuleSet::PLAYER_COUNT],
        };
        for target in 0..MahjongRuleSet::PLAYER_COUNT {
            let target = MahjongPlayerId(target);
            if target != player
                && self.win_button_available(target, tile.kind(), WinSource::RobbingKong(player))?
            {
                pending.options[target.0].push(MahjongClaimOption::Win);
            }
        }
        if pending.waiting_for().is_empty() {
            self.finalize_added_kong(player, tile, meld_index)?;
            return Ok(ActionOutcome::KongDeclared {
                player,
                tile: tile.kind(),
                added: true,
            });
        }
        self.phase = Phase::WaitingForClaims(pending);
        Ok(ActionOutcome::ClaimRecorded { player })
    }
}
