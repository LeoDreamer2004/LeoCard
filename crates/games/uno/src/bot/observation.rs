use crate::{UnoCard, UnoFlipSide, UnoGameState, UnoTurnState};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Opponent {
    pub id: PlayerId,
    pub hand_len: usize,
}

/// Only the acting player's hand, legal plays and public opponent hand sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnoBotObservation {
    pub(super) turn: UnoTurnState,
    pub(super) hand: Vec<UnoCard>,
    pub(super) playable: Vec<UnoCard>,
    pub(super) opponents: Vec<Opponent>,
    pub(super) flip_side: Option<UnoFlipSide>,
    pub(super) forced_skip: bool,
}

impl UnoBotObservation {
    pub fn from_game(game: &UnoGameState, forced_skip: bool) -> Option<Self> {
        let turn = game.turn()?;
        let hand = game.player(turn.current_player)?.hand().to_vec();
        let playable = hand
            .iter()
            .copied()
            .filter(|card| game.can_play(turn.current_player, *card))
            .collect();
        Some(Self {
            turn,
            hand,
            playable,
            opponents: game
                .players()
                .iter()
                .filter(|opponent| opponent.id() != turn.current_player && !opponent.eliminated())
                .map(|opponent| Opponent {
                    id: opponent.id(),
                    hand_len: opponent.hand().len(),
                })
                .collect(),
            flip_side: game.flip_side(),
            forced_skip,
        })
    }

    pub fn should_call_uno(&self) -> bool {
        self.turn.pending_swap.is_none() && !self.forced_skip
    }
}
