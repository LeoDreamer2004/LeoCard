use super::{
    ClaimPriority, ClaimTrigger, MahjongActionOutcome, MahjongClaim, MahjongClaimOption,
    MahjongDiscard, MahjongDrawOrigin, MahjongGameError, MahjongGameState, MahjongPendingClaim,
    MahjongPhase, MahjongWinRecord, claim_option_priority, claim_priority, players_after,
    validate_player,
};
use crate::{
    MahjongMeld, MahjongMeldKind, MahjongRuleSet, MahjongScoreResult, MahjongTile, MahjongTileKind,
    MahjongWinSource, is_complete_hand,
};
use leocard_game_common::PlayerId;

use std::array;

impl MahjongGameState {
    pub fn discard(
        &mut self,
        player: PlayerId,
        tile: MahjongTile,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        self.ensure_playing_turn(player)?;
        let position = self.players[player.index()]
            .hand
            .iter()
            .position(|held| *held == tile)
            .ok_or(MahjongGameError::TileNotInHand(tile))?;
        let from_drawn = self.last_drawn == Some(tile);
        self.players[player.index()].hand.remove(position);
        self.last_drawn = None;
        let discard_index = self.discards.len();
        self.discards.push(MahjongDiscard {
            player,
            tile,
            claimed_by: None,
            from_drawn,
        });
        let pending = self.pending_for_discard(discard_index, player, tile)?;
        if pending.waiting_for().is_empty() {
            return self.advance_after_unclaimed_discard(player);
        }
        self.phase = MahjongPhase::WaitingForClaims(pending);
        Ok(MahjongActionOutcome::Discarded { player, tile })
    }

    pub fn respond_to_claim(
        &mut self,
        player: PlayerId,
        claim: MahjongClaim,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        validate_player(player)?;
        {
            let MahjongPhase::WaitingForClaims(pending) = &mut self.phase else {
                return Err(MahjongGameError::WrongPhase);
            };
            if pending.options[player.index()].is_empty() {
                return Err(MahjongGameError::InvalidClaim);
            }
            if pending.responses[player.index()].is_some() {
                return Err(MahjongGameError::AlreadyResponded);
            }
            if let Some(option) = claim.option()
                && !pending.options[player.index()].contains(&option)
            {
                return Err(MahjongGameError::InvalidClaim);
            }
            pending.responses[player.index()] = Some(claim);
        }
        self.cancel_lower_priority_claims()?;
        let MahjongPhase::WaitingForClaims(pending) = &self.phase else {
            unreachable!("priority cancellation does not leave the claim phase");
        };
        if !pending.waiting_for().is_empty() {
            return Ok(MahjongActionOutcome::ClaimRecorded { player });
        }
        self.resolve_pending_claim()
    }

    fn cancel_lower_priority_claims(&mut self) -> Result<(), MahjongGameError> {
        let MahjongPhase::WaitingForClaims(pending) = &self.phase else {
            return Err(MahjongGameError::WrongPhase);
        };
        let pending = pending.clone();
        let source = pending.source_player();
        let mut selected: Option<(ClaimPriority, bool)> = None;
        for player in players_after(source) {
            let Some(claim) = pending.responses[player.index()] else {
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
        let mut cancel = [false; MahjongRuleSet::PLAYER_COUNT as usize];
        for player in players_after(source) {
            if pending.responses[player.index()].is_some()
                || pending.options[player.index()].is_empty()
            {
                continue;
            }
            let can_still_matter = pending.options[player.index()]
                .iter()
                .copied()
                .any(|option| {
                    if self.rules.multiple_winners
                        && selected_is_win
                        && option == MahjongClaimOption::Win
                    {
                        return true;
                    }
                    claim_option_priority(source, player, option) > selected
                });
            cancel[player.index()] = !can_still_matter;
        }
        let MahjongPhase::WaitingForClaims(pending) = &mut self.phase else {
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
        player: PlayerId,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        self.ensure_playing_turn(player)?;
        if self.players[player.index()].dead_hand {
            return Err(MahjongGameError::CannotWin);
        }
        let winning = self.last_drawn.ok_or(MahjongGameError::CannotWin)?;
        let source = match self.draw_origin {
            MahjongDrawOrigin::Normal => MahjongWinSource::SelfDraw,
            MahjongDrawOrigin::KongReplacement => MahjongWinSource::KongReplacement,
            MahjongDrawOrigin::FlowerReplacement => MahjongWinSource::FlowerReplacement,
        };
        let score = self.score_for(player, winning.kind(), source)?;
        if self.is_legal_score(&score) {
            return self.finish_with_winners(vec![MahjongWinRecord {
                player,
                from: None,
                concealed_hand: self.players[player.index()].is_concealed_hand(),
                winning_tile: winning,
                score,
                wait_kind_count: self.wait_kind_count(player, winning, true),
            }]);
        }
        if self.rules.false_win {
            let penalty = self.apply_false_win(player);
            return Ok(MahjongActionOutcome::FalseWin {
                player,
                deltas: penalty,
            });
        }
        Err(MahjongGameError::CannotWin)
    }

    pub fn self_draw_available(&self, player: PlayerId) -> Result<bool, MahjongGameError> {
        Ok(self
            .self_draw_score(player)?
            .is_some_and(|score| self.is_legal_score(&score) || self.rules.false_win))
    }

    /// 供自动玩家判断真正合法的自摸，避免“允许错和”开启时主动报错和。
    pub fn legal_self_draw_available(&self, player: PlayerId) -> Result<bool, MahjongGameError> {
        Ok(self
            .self_draw_score(player)?
            .is_some_and(|score| self.is_legal_score(&score)))
    }

    /// 判断当前响应窗口中的和牌是否真正达到起和要求。
    pub fn legal_claim_win_available(&self, player: PlayerId) -> Result<bool, MahjongGameError> {
        validate_player(player)?;
        let MahjongPhase::WaitingForClaims(pending) = &self.phase else {
            return Ok(false);
        };
        if !pending.options[player.index()].contains(&MahjongClaimOption::Win) {
            return Ok(false);
        }
        let (tile, source) = match pending.trigger {
            ClaimTrigger::Discard { from, tile, .. } => (tile, MahjongWinSource::Discard(from)),
            ClaimTrigger::AddedKong { player, tile, .. } => {
                (tile, MahjongWinSource::RobbingKong(player))
            }
        };
        let score = self.score_for(player, tile.kind(), source)?;
        Ok(self.is_legal_score(&score))
    }

    pub(super) fn self_draw_score(
        &self,
        player: PlayerId,
    ) -> Result<Option<MahjongScoreResult>, MahjongGameError> {
        validate_player(player)?;
        if !matches!(self.phase, MahjongPhase::Playing)
            || self.current_player != player
            || self.players[player.index()].dead_hand
        {
            return Ok(None);
        }
        let Some(winning) = self.last_drawn else {
            return Ok(None);
        };
        let concealed: Vec<_> = self.players[player.index()]
            .hand
            .iter()
            .map(|tile| tile.kind())
            .collect();
        if !is_complete_hand(&concealed, &self.players[player.index()].melds) {
            return Ok(None);
        }
        let source = match self.draw_origin {
            MahjongDrawOrigin::Normal => MahjongWinSource::SelfDraw,
            MahjongDrawOrigin::KongReplacement => MahjongWinSource::KongReplacement,
            MahjongDrawOrigin::FlowerReplacement => MahjongWinSource::FlowerReplacement,
        };
        let score = self.score_for(player, winning.kind(), source)?;
        Ok(Some(score))
    }

    pub fn declare_concealed_kong(
        &mut self,
        player: PlayerId,
        tile: MahjongTileKind,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        self.ensure_playing_turn(player)?;
        if self.wall.is_empty() || tile.is_flower() {
            return Err(MahjongGameError::CannotKong);
        }
        let positions: Vec<_> = self.players[player.index()]
            .hand
            .iter()
            .enumerate()
            .filter(|(_, held)| held.kind() == tile)
            .map(|(index, _)| index)
            .collect();
        if positions.len() != 4 {
            return Err(MahjongGameError::CannotKong);
        }
        for position in positions.into_iter().rev() {
            self.players[player.index()].hand.remove(position);
        }
        self.players[player.index()]
            .melds
            .push(MahjongMeld::concealed_kong(tile));
        self.draw_replacement(player, MahjongDrawOrigin::KongReplacement)?;
        Ok(MahjongActionOutcome::KongDeclared {
            player,
            tile,
            added: false,
        })
    }

    pub fn declare_added_kong(
        &mut self,
        player: PlayerId,
        tile: MahjongTile,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        self.ensure_playing_turn(player)?;
        if self.wall.is_empty() {
            return Err(MahjongGameError::CannotKong);
        }
        if !self.players[player.index()].hand.contains(&tile) {
            return Err(MahjongGameError::TileNotInHand(tile));
        }
        let Some(meld_index) = self.players[player.index()]
            .melds
            .iter()
            .position(|meld| meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind())
        else {
            return Err(MahjongGameError::CannotKong);
        };
        let mut pending = MahjongPendingClaim {
            trigger: ClaimTrigger::AddedKong {
                player,
                tile,
                meld_index,
            },
            options: array::from_fn(|_| Vec::new()),
            responses: [None; MahjongRuleSet::PLAYER_COUNT as usize],
        };
        for target in 0..usize::from(MahjongRuleSet::PLAYER_COUNT) {
            let target = PlayerId::from_index(target);
            if target != player
                && self.win_button_available(
                    target,
                    tile.kind(),
                    MahjongWinSource::RobbingKong(player),
                )?
            {
                pending.options[target.index()].push(MahjongClaimOption::Win);
            }
        }
        if pending.waiting_for().is_empty() {
            self.finalize_added_kong(player, tile, meld_index)?;
            return Ok(MahjongActionOutcome::KongDeclared {
                player,
                tile: tile.kind(),
                added: true,
            });
        }
        self.phase = MahjongPhase::WaitingForClaims(pending);
        Ok(MahjongActionOutcome::ClaimRecorded { player })
    }
}
