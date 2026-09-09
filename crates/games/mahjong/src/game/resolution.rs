use super::{
    ActionOutcome, ClaimTrigger, GameError, GameState, HandResult, MahjongClaim,
    MahjongClaimOption, MahjongDrawOrigin, PendingClaim, Phase, WinRecord, claim_priority,
    next_player, players_after,
};
use crate::{
    MahjongMatchLength, MahjongPlayerId, MahjongRuleSet, MahjongScoreResult, MahjongTile,
    MahjongTileKind, Meld, ScoreInput, WinContext, WinSource, is_complete_hand, score_hand,
};
use std::array;
use std::collections::HashSet;

impl GameState {
    pub(super) fn pending_for_discard(
        &self,
        discard_index: usize,
        from: MahjongPlayerId,
        tile: MahjongTile,
    ) -> Result<PendingClaim, GameError> {
        let mut pending = PendingClaim {
            trigger: ClaimTrigger::Discard {
                discard_index,
                from,
                tile,
            },
            options: array::from_fn(|_| Vec::new()),
            responses: [None; MahjongRuleSet::PLAYER_COUNT],
        };
        for target_index in 0..MahjongRuleSet::PLAYER_COUNT {
            let target = MahjongPlayerId(target_index);
            if target == from {
                continue;
            }
            if self.win_button_available(target, tile.kind(), WinSource::Discard(from))? {
                pending.options[target_index].push(MahjongClaimOption::Win);
            }
            let hand = &self.players[target_index].hand;
            let same = hand
                .iter()
                .filter(|held| held.kind() == tile.kind())
                .count();
            if !self.wall.is_empty() && same >= 2 {
                pending.options[target_index].push(MahjongClaimOption::Pung);
            }
            if !self.wall.is_empty() && same >= 3 {
                pending.options[target_index].push(MahjongClaimOption::Kong);
            }
            if target == next_player(from)
                && let MahjongTileKind::Suited { suit, rank } = tile.kind()
            {
                for start in rank.saturating_sub(2)..=rank {
                    if (1..=7).contains(&start)
                        && (start..=start + 2)
                            .filter(|value| *value != rank)
                            .all(|value| {
                                hand.iter()
                                    .any(|held| held.kind() == MahjongTileKind::suited(suit, value))
                            })
                    {
                        pending.options[target_index].push(MahjongClaimOption::Chow { start });
                    }
                }
            }
        }
        Ok(pending)
    }

    pub(super) fn resolve_pending_claim(&mut self) -> Result<ActionOutcome, GameError> {
        let Phase::WaitingForClaims(pending) = &self.phase else {
            return Err(GameError::WrongPhase);
        };
        let pending = pending.clone();
        let (from, tile, source) = match pending.trigger {
            ClaimTrigger::Discard { from, tile, .. } => (from, tile, WinSource::Discard(from)),
            ClaimTrigger::AddedKong { player, tile, .. } => {
                (player, tile, WinSource::RobbingKong(player))
            }
        };
        let mut valid_winners = Vec::new();
        for target in players_after(from) {
            if pending.responses[target.0] != Some(MahjongClaim::Win) {
                continue;
            }
            let score = self.score_for(target, tile.kind(), source)?;
            if self.is_legal_score(&score) {
                valid_winners.push(WinRecord {
                    player: target,
                    from: Some(from),
                    winning_tile: tile,
                    score,
                });
            } else {
                self.apply_false_win(target);
            }
        }
        if !valid_winners.is_empty() {
            if !self.rules.multiple_winners {
                valid_winners.truncate(1);
            }
            return self.finish_with_winners(valid_winners);
        }

        match pending.trigger {
            ClaimTrigger::AddedKong {
                player,
                tile,
                meld_index,
            } => {
                self.finalize_added_kong(player, tile, meld_index)?;
                Ok(ActionOutcome::KongDeclared {
                    player,
                    tile: tile.kind(),
                    added: true,
                })
            }
            ClaimTrigger::Discard {
                discard_index,
                from,
                tile,
            } => {
                let best_claim = players_after(from)
                    .filter_map(|player| {
                        let claim = pending.responses[player.0]?;
                        matches!(
                            claim,
                            MahjongClaim::Chow { .. } | MahjongClaim::Pung | MahjongClaim::Kong
                        )
                        .then_some((
                            player,
                            claim,
                            claim_priority(from, player, claim),
                        ))
                    })
                    .max_by_key(|(_, _, priority)| *priority);
                if let Some((player, claim, _)) = best_claim {
                    return self.apply_discard_claim(player, claim, tile, discard_index, from);
                }
                self.phase = Phase::Playing;
                self.advance_after_unclaimed_discard(from)
            }
        }
    }

    fn apply_discard_claim(
        &mut self,
        player: MahjongPlayerId,
        claim: MahjongClaim,
        tile: MahjongTile,
        discard_index: usize,
        from: MahjongPlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.discards[discard_index].claimed_by = Some(player);
        match claim {
            MahjongClaim::Pung => {
                self.remove_kind_from_hand(player, tile.kind(), 2)?;
                self.players[player.0]
                    .melds
                    .push(Meld::pung(tile.kind(), from));
                self.current_player = player;
                self.last_drawn = None;
                self.phase = Phase::Playing;
            }
            MahjongClaim::Kong => {
                self.remove_kind_from_hand(player, tile.kind(), 3)?;
                self.players[player.0]
                    .melds
                    .push(Meld::melded_kong(tile.kind(), from));
                self.current_player = player;
                self.phase = Phase::Playing;
                self.draw_replacement(player, MahjongDrawOrigin::KongReplacement)?;
            }
            MahjongClaim::Chow { start } => {
                let MahjongTileKind::Suited { suit, rank } = tile.kind() else {
                    return Err(GameError::InvalidClaim);
                };
                for value in start..=start + 2 {
                    if value != rank {
                        self.remove_kind_from_hand(
                            player,
                            MahjongTileKind::suited(suit, value),
                            1,
                        )?;
                    }
                }
                self.players[player.0]
                    .melds
                    .push(Meld::chow(suit, start, from));
                self.current_player = player;
                self.last_drawn = None;
                self.phase = Phase::Playing;
            }
            MahjongClaim::Pass | MahjongClaim::Win => return Err(GameError::InvalidClaim),
        }
        if !matches!(self.phase, Phase::ReplacingFlower { .. }) {
            self.sort_hands();
        }
        Ok(ActionOutcome::Claimed {
            player,
            source: from,
            tile,
            claim,
        })
    }

    pub(super) fn finalize_added_kong(
        &mut self,
        player: MahjongPlayerId,
        tile: MahjongTile,
        meld_index: usize,
    ) -> Result<(), GameError> {
        let original = self.players[player.0].melds[meld_index];
        let from = original.claimed_from().ok_or(GameError::CannotKong)?;
        let position = self.players[player.0]
            .hand
            .iter()
            .position(|held| *held == tile)
            .ok_or(GameError::TileNotInHand(tile))?;
        self.players[player.0].hand.remove(position);
        self.players[player.0].melds[meld_index] = Meld::melded_kong(tile.kind(), from);
        self.phase = Phase::Playing;
        self.current_player = player;
        self.draw_replacement(player, MahjongDrawOrigin::KongReplacement)?;
        Ok(())
    }

    pub(super) fn advance_after_unclaimed_discard(
        &mut self,
        from: MahjongPlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.phase = Phase::Playing;
        self.draw_normal(next_player(from))
    }

    pub(super) fn win_button_available(
        &self,
        player: MahjongPlayerId,
        tile: MahjongTileKind,
        source: WinSource,
    ) -> Result<bool, GameError> {
        if self.players[player.0].dead_hand {
            return Ok(false);
        }
        let mut concealed: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .map(|held| held.kind())
            .collect();
        concealed.push(tile);
        if !is_complete_hand(&concealed, &self.players[player.0].melds) {
            return Ok(false);
        }
        let score = self.score_for_with_concealed(player, tile, source, concealed)?;
        Ok(self.is_legal_score(&score) || self.rules.false_win)
    }

    pub(super) fn score_for(
        &self,
        player: MahjongPlayerId,
        winning_tile: MahjongTileKind,
        source: WinSource,
    ) -> Result<MahjongScoreResult, GameError> {
        let mut concealed: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .map(|tile| tile.kind())
            .collect();
        if !source.is_self_draw() {
            concealed.push(winning_tile);
        }
        self.score_for_with_concealed(player, winning_tile, source, concealed)
    }

    fn score_for_with_concealed(
        &self,
        player: MahjongPlayerId,
        winning_tile: MahjongTileKind,
        source: WinSource,
        concealed: Vec<MahjongTileKind>,
    ) -> Result<MahjongScoreResult, GameError> {
        let score = score_hand(&ScoreInput {
            concealed,
            melds: self.players[player.0].melds.clone(),
            winning_tile,
            context: WinContext {
                source,
                seat_wind: self.seat_wind(player).expect("player was validated"),
                prevalent_wind: self.prevalent_wind,
                last_wall_tile: self.wall.is_empty(),
                last_of_kind: self.is_last_of_kind(winning_tile, source),
                flower_count: self.players[player.0].flowers.len() as u8,
            },
        })?;
        Ok(score)
    }

    pub(super) fn is_legal_score(&self, score: &MahjongScoreResult) -> bool {
        !self.rules.minimum_eight_points || score.points_without_flowers >= 8
    }

    fn is_last_of_kind(&self, tile: MahjongTileKind, source: WinSource) -> bool {
        let mut visible = self
            .discards
            .iter()
            .filter(|discard| discard.claimed_by.is_none() && discard.tile.kind() == tile)
            .count();
        for player in &self.players {
            for meld in &player.melds {
                if meld.is_open() {
                    visible += meld
                        .tile_kinds()
                        .iter()
                        .filter(|kind| **kind == tile)
                        .count();
                }
            }
        }
        match source {
            WinSource::Discard(_) => visible >= 4,
            WinSource::RobbingKong(_) => false,
            _ => visible >= 3,
        }
    }

    pub(super) fn apply_false_win(
        &mut self,
        player: MahjongPlayerId,
    ) -> [i32; MahjongRuleSet::PLAYER_COUNT] {
        let mut delta = [10; MahjongRuleSet::PLAYER_COUNT];
        delta[player.0] = -30;
        for (hand_delta, penalty) in self.hand_deltas.iter_mut().zip(delta) {
            *hand_delta += penalty;
        }
        self.players[player.0].dead_hand = true;
        self.players[player.0].hand_revealed = true;
        delta
    }

    pub(super) fn finish_with_winners(
        &mut self,
        winners: Vec<WinRecord>,
    ) -> Result<ActionOutcome, GameError> {
        let claimed_tile = match &self.phase {
            Phase::WaitingForClaims(pending) => Some(pending.tile()),
            _ => None,
        };
        let reveal_all_hands = winners
            .iter()
            .any(|winner| winner.score.fans.iter().any(|fan| fan.fan.points() >= 48));
        for winner in &winners {
            let hand = &mut self.players[winner.player.0];
            hand.hand_revealed = true;
            if winner.from.is_some()
                && let Some(tile) = claimed_tile
            {
                hand.hand.push(tile);
                hand.hand.sort_by_key(|tile| (tile.kind(), tile.copy()));
            }
        }
        if reveal_all_hands {
            for player in &mut self.players {
                player.hand_revealed = true;
            }
        }
        let winner_ids: HashSet<_> = winners.iter().map(|winner| winner.player).collect();
        for winner in &winners {
            let points = i32::from(winner.score.total_points);
            match winner.from {
                None => {
                    let payment = points + i32::from(self.rules.minimum_eight_points) * 8;
                    for payer in 0..MahjongRuleSet::PLAYER_COUNT {
                        if payer != winner.player.0 {
                            self.hand_deltas[payer] -= payment;
                            self.hand_deltas[winner.player.0] += payment;
                        }
                    }
                }
                Some(discarder) => {
                    let base = i32::from(self.rules.minimum_eight_points) * 8;
                    let payment = points + base;
                    self.hand_deltas[discarder.0] -= payment;
                    self.hand_deltas[winner.player.0] += payment;
                    if self.rules.minimum_eight_points {
                        for payer in 0..MahjongRuleSet::PLAYER_COUNT {
                            let payer = MahjongPlayerId(payer);
                            if payer != discarder && !winner_ids.contains(&payer) {
                                self.hand_deltas[payer.0] -= 8;
                                self.hand_deltas[winner.player.0] += 8;
                            }
                        }
                    }
                }
            }
        }
        self.finish_hand(winners, false)
    }

    pub(super) fn finish_exhaustive_draw(&mut self) -> Result<ActionOutcome, GameError> {
        self.finish_hand(Vec::new(), true)
    }

    fn finish_hand(
        &mut self,
        winners: Vec<WinRecord>,
        exhaustive_draw: bool,
    ) -> Result<ActionOutcome, GameError> {
        for index in 0..MahjongRuleSet::PLAYER_COUNT {
            self.match_scores[index] += self.hand_deltas[index];
        }
        self.hands_in_match += 1;
        let match_complete = self.rules.match_length == MahjongMatchLength::SingleHand
            || self.hands_in_match >= self.rules.match_length.hand_count();
        let result = HandResult {
            winners,
            exhaustive_draw,
            deltas: self.hand_deltas,
            match_scores: self.match_scores,
            match_complete,
            sequence_index: self.sequence_index,
        };
        self.phase = Phase::Finished(result.clone());
        Ok(ActionOutcome::HandFinished(result))
    }

    fn remove_kind_from_hand(
        &mut self,
        player: MahjongPlayerId,
        kind: MahjongTileKind,
        count: usize,
    ) -> Result<(), GameError> {
        for _ in 0..count {
            let Some(position) = self.players[player.0]
                .hand
                .iter()
                .position(|tile| tile.kind() == kind)
            else {
                return Err(GameError::InvalidClaim);
            };
            self.players[player.0].hand.remove(position);
        }
        Ok(())
    }
}
