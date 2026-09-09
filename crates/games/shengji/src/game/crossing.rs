use super::{
    ActionOutcome, FiveTrumpCrossingStage, GameError, GameState, Phase, partner, remove_cards,
    validate_cards_owned,
};
use crate::{ShengjiCard, ShengjiPlayerId};

impl GameState {
    /// 符合资格的玩家提交五张过江牌；`None` 表示明确放弃。本阶段的资格
    /// 固定取自埋底后的原始手牌，交换产生的新主牌数量不会再次触发过江。
    pub fn choose_five_trump_crossing(
        &mut self,
        player: ShengjiPlayerId,
        cards: Option<&[ShengjiCard]>,
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::FiveTrumpCrossing
            || self
                .five_trump_crossing
                .as_ref()
                .is_none_or(|state| state.stage != FiveTrumpCrossingStage::Deciding)
        {
            return Err(GameError::WrongPhase);
        }
        self.player(player)?;
        let state = self.five_trump_crossing.as_ref().unwrap();
        if !state.eligible(player) {
            return Err(GameError::CrossingNotEligible);
        }
        if state.decision_made(player) {
            return Err(GameError::CrossingAlreadyDecided);
        }

        let crossing = cards.is_some();
        if let Some(cards) = cards {
            if cards.len() != 5 {
                return Err(GameError::WrongCrossingCount {
                    expected: 5,
                    actual: cards.len(),
                });
            }
            let hand = &self.players[usize::from(player.0)].hand;
            validate_cards_owned(cards, hand)?;
            let trump = self.trump.unwrap();
            if hand
                .iter()
                .filter(|card| trump.is_trump(**card))
                .any(|card| !cards.contains(card))
            {
                return Err(GameError::CrossingMustIncludeAllTrumps);
            }
        }

        let state = self.five_trump_crossing.as_mut().unwrap();
        let index = usize::from(player.0);
        if let Some(cards) = cards {
            state.outgoing[index] = Some(cards.to_vec());
        } else {
            state.declined[index] = true;
        }
        let decisions_complete = state.pending_players().is_empty();
        if decisions_complete {
            self.apply_five_trump_outgoing();
        }
        Ok(ActionOutcome::FiveTrumpCrossingDecision {
            player,
            crossing,
            decisions_complete,
        })
    }

    /// 对家看过收到的过江牌后选择任意五张回交。所有回牌先分别锁定，待双方
    /// 队伍全部完成后再同时移动，因此同队两人可以彼此过江。
    pub fn return_five_trump_crossing(
        &mut self,
        player: ShengjiPlayerId,
        cards: &[ShengjiCard],
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::FiveTrumpCrossing
            || self
                .five_trump_crossing
                .as_ref()
                .is_none_or(|state| state.stage != FiveTrumpCrossingStage::Returning)
        {
            return Err(GameError::WrongPhase);
        }
        self.player(player)?;
        let state = self.five_trump_crossing.as_ref().unwrap();
        if !state.return_required(player) {
            return Err(GameError::CrossingReturnNotRequired);
        }
        if state.return_made(player) {
            return Err(GameError::CrossingAlreadyReturned);
        }
        if cards.len() != 5 {
            return Err(GameError::WrongCrossingCount {
                expected: 5,
                actual: cards.len(),
            });
        }
        validate_cards_owned(cards, &self.players[usize::from(player.0)].hand)?;
        self.five_trump_crossing.as_mut().unwrap().returned[usize::from(player.0)] =
            Some(cards.to_vec());
        let crossing_complete = self
            .five_trump_crossing
            .as_ref()
            .unwrap()
            .pending_players()
            .is_empty();
        if crossing_complete {
            self.apply_five_trump_returns();
            self.start_playing();
        }
        Ok(ActionOutcome::FiveTrumpCrossingReturn {
            player,
            crossing_complete,
        })
    }

    fn apply_five_trump_outgoing(&mut self) {
        let outgoing = self.five_trump_crossing.as_ref().unwrap().outgoing.clone();
        for (index, cards) in outgoing.iter().enumerate() {
            if let Some(cards) = cards {
                remove_cards(&mut self.players[index].hand, cards);
            }
        }
        for (index, cards) in outgoing.iter().enumerate() {
            if let Some(cards) = cards {
                self.players[usize::from(partner(ShengjiPlayerId(index as u8)).0)]
                    .hand
                    .extend(cards.iter().copied());
            }
        }
        let state = self.five_trump_crossing.as_mut().unwrap();
        if state.outgoing.iter().any(Option::is_some) {
            state.stage = FiveTrumpCrossingStage::Returning;
        } else {
            self.start_playing();
        }
    }

    fn apply_five_trump_returns(&mut self) {
        let returned = self.five_trump_crossing.as_ref().unwrap().returned.clone();
        for (index, cards) in returned.iter().enumerate() {
            if let Some(cards) = cards {
                remove_cards(&mut self.players[index].hand, cards);
            }
        }
        for (index, cards) in returned.iter().enumerate() {
            if let Some(cards) = cards {
                self.players[usize::from(partner(ShengjiPlayerId(index as u8)).0)]
                    .hand
                    .extend(cards.iter().copied());
            }
        }
    }
}
