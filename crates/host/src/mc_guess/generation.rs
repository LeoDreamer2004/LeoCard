use super::McGuessSession;
use crate::Delivery;
use crate::lifecycle::HostedGameLifecycle;
use leocard_game_common::PlayerId;
use leocard_mc_guess::{McGuessGameError, McGuessGameState, McGuessRuleSet};
use leocard_protocol::{RejectReason, ServerEvent};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Debug)]
pub(super) struct GraphGeneration {
    players: Option<Vec<PlayerId>>,
    rules: McGuessRuleSet,
    result: Arc<Mutex<Option<Result<McGuessGameState, McGuessGameError>>>>,
}

impl GraphGeneration {
    pub(super) fn new(players: Vec<PlayerId>, rules: McGuessRuleSet) -> Self {
        Self {
            players: Some(players),
            rules,
            result: Arc::default(),
        }
    }

    fn poll(&mut self) -> Option<Result<McGuessGameState, McGuessGameError>> {
        if let Some(players) = self.players.take() {
            let result = self.result.clone();
            let rules = self.rules;
            thread::spawn(move || {
                let game = McGuessGameState::new_graph(players, rules, fastrand::u64(..));
                *result.lock().expect("graph generation lock") = Some(game);
            });
        }
        self.result.lock().expect("graph generation lock").take()
    }
}

impl McGuessSession {
    pub(super) fn advance_generation(&mut self) -> Vec<Delivery> {
        let Some(result) = self.generation.as_mut().and_then(GraphGeneration::poll) else {
            return Vec::new();
        };
        self.generation = None;
        match result {
            Ok(game) => {
                self.game = Some(game);
                self.bump_revision();
                self.broadcast_game(None)
            }
            Err(error) => {
                self.game = None;
                self.match_id = None;
                self.match_state = None;
                self.room.reset_ready_after_rules_change();
                self.bump_revision();
                let mut deliveries = self.broadcast_lobby(None);
                deliveries.extend(self.room.broadcast_event(
                    None,
                    ServerEvent::Rejected {
                        reason: RejectReason::Game(error.into()),
                    },
                ));
                deliveries
            }
        }
    }
}
