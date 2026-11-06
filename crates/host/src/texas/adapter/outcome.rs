use super::state::{AdapterError, TexasAdapter, map_action_error};
use leocard_game_common::PlayerId;
use leocard_protocol::TexasEvent;
use leocard_texas_holdem::{TexasAction, TexasActionOutcome};

impl TexasAdapter {
    pub fn act(
        &mut self,
        player: PlayerId,
        action: TexasAction,
    ) -> Result<Vec<TexasEvent>, AdapterError> {
        let core_player = self.core_player(player)?;
        let community_before = self.game.community().len();
        let committed_before = self
            .game
            .players()
            .iter()
            .map(|state| state.committed_total())
            .collect::<Vec<_>>();
        let outcome = self
            .game
            .act(core_player, action)
            .map_err(map_action_error)?;
        if matches!(action, TexasAction::Fold) {
            self.spectator
                .record_fold(core_player.index(), community_before);
        }
        if self.game.community().len() != community_before || matches!(action, TexasAction::Fold) {
            self.equity_cache.clear();
        }
        let amount = self
            .game
            .betting_history()
            .last()
            .expect("an accepted action has a betting record")
            .payment;
        let mut events = vec![TexasEvent::ActionApplied {
            player,
            action,
            amount,
            statistics: self
                .game
                .last_action_statistics()
                .expect("an accepted action has statistics"),
        }];
        for (index, (before, state)) in committed_before.iter().zip(self.game.players()).enumerate()
        {
            let invested = before
                + if index == core_player.index() {
                    amount
                } else {
                    0
                };
            let returned = invested.saturating_sub(state.committed_total());
            if returned > 0 {
                events.push(TexasEvent::ChipsReturned {
                    player: self.protocol_player(PlayerId::from_index(index)),
                    amount: returned,
                });
            }
        }
        events.extend(self.outcome_events(outcome, community_before));
        Ok(events)
    }

    pub fn choose_runout(
        &mut self,
        player: PlayerId,
        twice: bool,
    ) -> Result<Vec<TexasEvent>, AdapterError> {
        let core_player = self.core_player(player)?;
        let community_before = self.game.community().len();
        let outcome = self
            .game
            .choose_runout(core_player, twice)
            .map_err(map_action_error)?;
        let complete = matches!(outcome, TexasActionOutcome::HandComplete(_));
        let mut events = self.outcome_events(outcome, community_before);
        // The final core outcome is HandComplete, so publish its vote explicitly too.
        if complete {
            self.equity_cache.clear();
            events.insert(
                0,
                TexasEvent::RunoutChosen {
                    player,
                    twice,
                    next_player: None,
                },
            );
        }
        Ok(events)
    }

    fn outcome_events(
        &self,
        outcome: TexasActionOutcome,
        community_before: usize,
    ) -> Vec<TexasEvent> {
        let mut events = Vec::new();
        match outcome {
            TexasActionOutcome::RunoutVoteStarted { current_player } => {
                events.push(TexasEvent::RunoutVoteStarted {
                    current_player: self.protocol_player(current_player),
                });
            }
            TexasActionOutcome::RunoutChosen {
                player,
                twice,
                next_player,
            } => {
                events.push(TexasEvent::RunoutChosen {
                    player: self.protocol_player(player),
                    twice,
                    next_player: Some(self.protocol_player(next_player)),
                });
            }
            TexasActionOutcome::BlindPosted { .. } => {}
            TexasActionOutcome::Acted { .. } => {}
            TexasActionOutcome::StreetAdvanced { street, .. } => {
                events.push(TexasEvent::StreetAdvanced {
                    street,
                    dealt: self.game.community()[community_before..].to_vec(),
                });
            }
            TexasActionOutcome::HandComplete(result) => {
                if let Some((player, statistics)) = self.game.squid().award_statistics() {
                    events.push(TexasEvent::SquidAwarded {
                        player: self.protocol_player(player),
                        statistics,
                    });
                }
                events.extend(self.players.iter().enumerate().map(|(index, participant)| {
                    TexasEvent::HandAnalyzed {
                        player: participant.id,
                        statistics: Box::new(
                            self.game
                                .hand_statistics(PlayerId::from_index(index))
                                .expect("a settled hand has statistics"),
                        ),
                    }
                }));
                events.push(TexasEvent::HandFinished {
                    showdown: result.showdown,
                });
            }
        }
        events
    }
}
