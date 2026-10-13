use crate::{
    BettingRecord, GameState, Phase, TexasHoldemAction, TexasHoldemCard, TexasHoldemPlayerId,
    TexasHoldemRuleSet, TexasHoldemStreet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Opponent {
    pub id: TexasHoldemPlayerId,
    pub stack: u32,
    pub committed: u32,
    pub street_commit: u32,
    pub all_in: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexasHoldemBotObservation {
    pub(super) rules: TexasHoldemRuleSet,
    pub(super) player: TexasHoldemPlayerId,
    pub(super) hand_number: u32,
    pub(super) hole: Vec<TexasHoldemCard>,
    pub(super) board: Vec<TexasHoldemCard>,
    pub(super) opponents: Vec<Opponent>,
    pub(super) history: Vec<BettingRecord>,
    pub(super) street: TexasHoldemStreet,
    pub(super) position: f64,
    pub(super) stack: u32,
    pub(super) committed: u32,
    pub(super) total_commit: u32,
    pub(super) pot: u32,
    pub(super) call: u32,
    pub(super) minimum_raise: u32,
    pub(super) current_bet: u32,
    pub(super) can_raise: bool,
    pub(super) blind: bool,
}

impl TexasHoldemBotObservation {
    pub fn from_game(game: &GameState, player: TexasHoldemPlayerId) -> Option<Self> {
        if game.current_player() != Some(player) {
            return None;
        }
        let Phase::Betting(street) = game.phase() else {
            return None;
        };
        let state = game.players().get(player.0)?;
        let count = game.players().len();
        let distance = (game.dealer().0 + count - player.0) % count;
        Some(Self {
            rules: *game.rules(),
            player,
            hand_number: game.hand_number(),
            hole: state.hole_cards().to_vec(),
            board: game.community().to_vec(),
            opponents: game
                .players()
                .iter()
                .filter(|state| state.id() != player && !state.folded())
                .map(|state| Opponent {
                    id: state.id(),
                    stack: state.stack(),
                    committed: state.committed_total(),
                    street_commit: state.committed_street(),
                    all_in: state.all_in(),
                })
                .collect(),
            history: game.betting_history().to_vec(),
            street: *street,
            position: 1.0 - distance as f64 / count as f64,
            stack: state.stack(),
            committed: state.committed_street(),
            total_commit: state.committed_total(),
            pot: game.pot(),
            call: game.amount_to_call(player).ok()?.min(state.stack()),
            minimum_raise: game.minimum_raise_to(),
            current_bet: game.current_bet(),
            can_raise: game.raise_allowed(player).ok()?,
            blind: game.blind_to_post().is_some(),
        })
    }

    pub fn immediate_action(&self) -> Option<TexasHoldemAction> {
        self.blind.then_some(TexasHoldemAction::PostBlind)
    }

    pub fn fallback(&self) -> TexasHoldemAction {
        if self.call == 0 {
            TexasHoldemAction::Check
        } else {
            TexasHoldemAction::Fold
        }
    }

    pub(super) fn seed(&self) -> u64 {
        self.hole.iter().chain(&self.board).fold(
            u64::from(self.hand_number) * 101 + self.history.len() as u64,
            |seed, card| {
                seed.wrapping_mul(53)
                    .wrapping_add(u64::from(card.rank().value()) * 4 + card.suit() as u64)
            },
        )
    }
}
