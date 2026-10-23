use crate::{ActionOutcome, GameError, GameState, UnoCard, UnoColor, UnoPlayerId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnoBotAction {
    ResolveSkip,
    SwapOneTarget {
        target: UnoPlayerId,
        index: usize,
    },
    GiveSwapOneCard(UnoCard),
    ForceTrade {
        first: UnoPlayerId,
        second: UnoPlayerId,
    },
    ChooseColor(UnoColor),
    SevenSwap(UnoPlayerId),
    Play {
        card: UnoCard,
        color: Option<UnoColor>,
    },
    AcceptDrawPenalty,
    Draw,
}

impl UnoBotAction {
    pub fn played_cards(&self) -> Vec<(UnoCard, Option<UnoColor>)> {
        match *self {
            Self::Play { card, color } => vec![(card, color)],
            _ => Vec::new(),
        }
    }

    pub fn apply(
        self,
        game: &mut GameState,
        player: UnoPlayerId,
    ) -> Result<ActionOutcome, GameError> {
        match self {
            Self::ResolveSkip => game.resolve_skip(player),
            Self::SwapOneTarget { target, index } => {
                game.choose_swap_one_target(player, target, index)
            }
            Self::GiveSwapOneCard(card) => game.give_swap_one_card(player, card),
            Self::ForceTrade { first, second } => game.force_trade_hands(player, first, second),
            Self::ChooseColor(color) => game.choose_initial_color(player, color),
            Self::SevenSwap(target) => game.choose_seven_swap_target(player, target),
            Self::Play { card, color } => game.play_card(player, card, color),
            Self::AcceptDrawPenalty => game.accept_draw_penalty(player),
            Self::Draw => game.draw_card(player),
        }
    }
}
