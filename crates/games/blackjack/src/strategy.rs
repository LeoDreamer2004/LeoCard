use crate::{
    BlackjackAction, BlackjackAvailableActions, BlackjackCard, BlackjackGameState,
    BlackjackHandValue, BlackjackPhase, BlackjackRank,
};
use leocard_game_common::PlayerId;

/// Only exposed player cards and the dealer upcard, never the hole card or shoe.
#[derive(Clone, Debug, PartialEq)]
pub struct BlackjackObservation {
    pub cards: Vec<BlackjackCard>,
    pub dealer: Option<BlackjackCard>,
    pub available: BlackjackAvailableActions,
    pub exposed: Vec<BlackjackCard>,
}

impl BlackjackObservation {
    pub fn for_player(game: &BlackjackGameState, player: PlayerId) -> Option<Self> {
        if game.current_player() != Some(player) {
            return None;
        }
        Some(Self {
            cards: game.current_hand().map_or_else(Vec::new, |hand| {
                game.players()[player.index()].hands[hand].cards.clone()
            }),
            dealer: game.visible_dealer().first().copied(),
            available: game.available_actions(player),
            exposed: game
                .players()
                .iter()
                .enumerate()
                .flat_map(|(index, state)| {
                    state
                        .hands
                        .iter()
                        .enumerate()
                        .filter(move |(hand, _)| {
                            index != player.index() || Some(*hand) != game.current_hand()
                        })
                        .flat_map(|(_, hand)| hand.cards.iter().copied())
                })
                .collect(),
        })
    }
}

pub struct BlackjackBot;

impl BlackjackBot {
    pub fn choose(observation: &BlackjackObservation) -> BlackjackAction {
        let allowed = observation.available;
        if allowed.minimum_bet > 0 {
            return BlackjackAction::Bet {
                amount: allowed.minimum_bet,
            };
        }
        let value = BlackjackHandValue::of(&observation.cards);
        let dealer = observation.dealer.map_or(10, |card| {
            if card.rank == BlackjackRank::Ace {
                11
            } else {
                card.rank.points()
            }
        });
        let pair = observation
            .cards
            .first()
            .map_or(0, |card| card.rank.points());
        if allowed.split
            && match pair {
                1 | 8 => true,
                2 | 3 | 7 => dealer <= 7,
                4 => matches!(dealer, 5 | 6),
                6 => dealer <= 6,
                9 => dealer <= 9 && dealer != 7,
                _ => false,
            }
        {
            return BlackjackAction::Split;
        }
        let double = if value.soft {
            match value.total {
                13 | 14 => matches!(dealer, 5 | 6),
                15 | 16 => (4..=6).contains(&dealer),
                17 => (3..=6).contains(&dealer),
                18 => (2..=6).contains(&dealer),
                19 => dealer == 6,
                _ => false,
            }
        } else {
            match value.total {
                9 => (3..=6).contains(&dealer),
                10 => dealer <= 9,
                11 => true,
                _ => false,
            }
        };
        if allowed.double_down && double {
            return BlackjackAction::DoubleDown;
        }
        let hit = if value.soft {
            value.total < 18 || (value.total == 18 && dealer >= 9)
        } else {
            value.total < 12
                || (value.total == 12 && !(4..=6).contains(&dealer))
                || (value.total <= 16 && dealer >= 7)
        };
        if hit {
            BlackjackAction::Hit
        } else {
            BlackjackAction::Stand
        }
    }
}

pub struct BlackjackAutoPlay;

impl BlackjackAutoPlay {
    pub fn choose(observation: &BlackjackObservation) -> BlackjackAction {
        BlackjackBot::choose(observation)
    }

    pub fn timeout(phase: BlackjackPhase, minimum_bet: u32) -> BlackjackAction {
        if phase == BlackjackPhase::Betting {
            BlackjackAction::Bet {
                amount: minimum_bet,
            }
        } else {
            BlackjackAction::Stand
        }
    }
}
