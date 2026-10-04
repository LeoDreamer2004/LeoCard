use crate::{GameState, PendingSwap, Phase, TurnState, UnoColor, UnoPlayerId};

#[derive(Clone, Debug)]
pub struct UnoActionContext {
    pub(super) player: UnoPlayerId,
    pub(super) hands: Vec<usize>,
    pub(super) exposed: bool,
    pub(super) eliminated: Vec<bool>,
    pub(super) turn: TurnState,
    pub(super) color: Option<UnoColor>,
}

impl UnoActionContext {
    pub fn capture(game: &GameState, player: UnoPlayerId) -> Option<Self> {
        if !matches!(game.phase(), Phase::Playing) || game.player(player).is_none() {
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
        matches!(self.turn.pending_swap, Some(PendingSwap::SevenSwap { .. }))
    }
}
