use super::{TexasActionOutcome, TexasGameError, TexasGameState, TexasPhase};
use leocard_game_common::PlayerId;
use std::mem;

impl TexasGameState {
    /// Refund only live players, using the smallest surviving all-in's TOTAL investment.
    /// The all-in marker stays latched until the next hand; refunded chips do not reopen betting.
    pub(super) fn refund_no_side_pot_excess(&mut self) {
        if self.rules.side_pots {
            return;
        }
        let Some(cap) = self
            .players
            .iter()
            .filter(|player| !player.folded && player.all_in)
            .map(|player| player.committed_total)
            .min()
        else {
            return;
        };
        for player in self.players.iter_mut().filter(|player| !player.folded) {
            let refund = player.committed_total.saturating_sub(cap);
            player.committed_total -= refund;
            player.committed_street = player.committed_street.saturating_sub(refund);
            player.stack += refund;
        }
    }

    pub(super) fn close_betting(&mut self) -> Result<(), TexasGameError> {
        if self.rules.run_it_twice && self.community.len() < 5 && self.contenders().len() >= 2 {
            self.runout_voters = self.contenders();
            let dealer = self.dealer;
            let count = self.players.len();
            self.runout_voters
                .sort_by_key(|player| (player.index() + count - dealer.index() - 1) % count);
            self.current_player = self.runout_voters.first().copied();
            self.phase = TexasPhase::ChoosingRunout;
            self.needs_action.fill(false);
            self.raise_allowed.fill(false);
            return Ok(());
        }
        self.draw_runout();
        self.settle_showdown(None).map(|_| ())
    }

    /// Every live contender votes clockwise after the dealer, even after a vote for once.
    /// No voting action is added to betting history or action statistics.
    pub fn choose_runout(
        &mut self,
        player: PlayerId,
        twice: bool,
    ) -> Result<TexasActionOutcome, TexasGameError> {
        let previous = self.clone();
        match self.choose_runout_inner(player, twice) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                *self = previous;
                Err(error)
            }
        }
    }

    fn choose_runout_inner(
        &mut self,
        player: PlayerId,
        twice: bool,
    ) -> Result<TexasActionOutcome, TexasGameError> {
        if !matches!(self.phase, TexasPhase::ChoosingRunout) {
            return Err(TexasGameError::NotChoosingRunout);
        }
        if player.index() >= self.players.len() {
            return Err(TexasGameError::InvalidPlayer(player));
        }
        if self.runout_votes.iter().any(|(voter, _)| *voter == player) {
            return Err(TexasGameError::RunoutAlreadyChosen(player));
        }
        let expected = self.current_player.expect("runout phase has a voter");
        if player != expected {
            return Err(TexasGameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        self.runout_votes.push((player, twice));
        if let Some(&next_player) = self.runout_voters.get(self.runout_votes.len()) {
            self.current_player = Some(next_player);
            return Ok(TexasActionOutcome::RunoutChosen {
                player,
                twice,
                next_player,
            });
        }
        let prefix = self.community.clone();
        self.draw_runout();
        let second_board = if self.runout_votes.iter().all(|(_, vote)| *vote) {
            let first_board = self.community.clone();
            self.community = prefix;
            self.draw_runout();
            Some(mem::replace(&mut self.community, first_board))
        } else {
            None
        };
        let result = self.settle_showdown(second_board)?;
        self.record_hand_statistics();
        Ok(TexasActionOutcome::HandComplete(result))
    }
}
