use crate::{UnoColor, UnoGameState, UnoPendingSwap, UnoPhase, UnoTurnState};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug)]
pub struct UnoActionContext {
    pub(super) player: PlayerId,
    pub(super) hands: Vec<usize>,
    pub(super) exposed: bool,
    pub(super) eliminated: Vec<bool>,
    pub(super) turn: UnoTurnState,
    pub(super) color: Option<UnoColor>,
}

impl UnoActionContext {
    pub fn capture(game: &UnoGameState, player: PlayerId) -> Option<Self> {
        if !matches!(game.phase(), UnoPhase::Playing) || game.player(player).is_none() {
            return None;
        }
        Some(Self {
            player,
            hands: game
                .players()
                .iter()
                .map(|state| state.hand().len())
                .collect(),
            eliminated: game
                .players()
                .iter()
                .map(|state| state.eliminated())
                .collect(),
            exposed: game
                .uno_exposed_players()
                .any(|candidate| candidate == player),
            turn: game.turn()?,
            color: game.current_color(),
        })
    }

    pub(super) fn seven_swap(&self) -> bool {
        matches!(
            self.turn.pending_swap,
            Some(UnoPendingSwap::SevenSwap { .. })
        )
    }
}
