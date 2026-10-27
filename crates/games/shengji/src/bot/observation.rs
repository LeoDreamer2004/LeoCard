use super::ShengjiBotAction;
use super::candidates::plays;
use crate::play::category;
use crate::{
    Category, FiveTrumpCrossingStage, GameState, Phase, ShengjiCard, ShengjiPlayerId,
    ShengjiRuleSet, ShengjiTeamId, ShengjiTrump, TrickRecord, build_deck_for,
};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct ShengjiBotObservation {
    pub(super) player: ShengjiPlayerId,
    pub(super) hand: Vec<ShengjiCard>,
    pub(super) trump: ShengjiTrump,
    pub(super) rules: ShengjiRuleSet,
    pub(super) phase: Phase,
    pub(super) trick: Option<TrickRecord>,
    pub(super) unseen: Vec<ShengjiCard>,
    pub(super) exposed: Vec<Vec<ShengjiCard>>,
    pub(super) lengths: Vec<usize>,
    pub(super) voids: Vec<[bool; 5]>,
    pub(super) bottom: Option<Vec<ShengjiCard>>,
    pub(super) counter_options: Vec<Vec<ShengjiCard>>,
    pub(super) crossing: Option<FiveTrumpCrossingStage>,
    pub(super) collecting_team: ShengjiTeamId,
    pub(super) collecting_score: u32,
}

impl ShengjiBotObservation {
    pub fn from_game(game: &GameState, player: ShengjiPlayerId) -> Option<Self> {
        let trump = game.trump()?;
        let hand = game.players().get(usize::from(player.0))?.hand.clone();
        let trick = game.current_trick();
        let played: HashSet<_> = game
            .history()
            .iter()
            .chain(trick.iter())
            .flat_map(|trick| {
                trick
                    .plays
                    .iter()
                    .flat_map(|(_, play)| play.cards.iter().copied())
            })
            .collect();
        let exposed = (0..usize::from(game.rules().player_count()))
            .map(|index| {
                // Secret burial/copying or a partner exchange can relocate an exposed physical card.
                // Only retain ownership when public rules still make it certain.
                if game.dealer() == Some(ShengjiPlayerId(index as u8))
                    || game.rules().bottom_copy
                    || game.rules().five_trump_crossing
                {
                    return Vec::new();
                }
                game.bidding()
                    .exposed_cards(ShengjiPlayerId(index as u8))
                    .into_iter()
                    .filter(|card| !played.contains(card))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let bottom = (game.bottom_burier() == Some(player)).then(|| game.buried().to_vec());
        let known: HashSet<_> = hand
            .iter()
            .chain(played.iter())
            .chain(exposed.iter().flatten())
            .chain(bottom.iter().flatten())
            .copied()
            .collect();
        let mut voids = vec![[false; 5]; usize::from(game.rules().player_count())];
        for trick in game.history().iter().chain(trick.iter()) {
            let lead = trick.plays.first()?.1.category;
            for (other, play) in &trick.plays {
                if play.cards.iter().any(|card| category(*card, trump) != lead) {
                    voids[usize::from(other.0)][category_index(lead)] = true;
                }
            }
        }
        Some(Self {
            player,
            hand,
            trump,
            rules: *game.rules(),
            phase: game.phase().clone(),
            trick,
            unseen: build_deck_for(game.rules().deck_count)
                .into_iter()
                .filter(|card| !known.contains(card))
                .collect(),
            exposed,
            lengths: game
                .players()
                .iter()
                .map(|player| player.hand.len())
                .collect(),
            voids,
            bottom,
            counter_options: game
                .bidding()
                .counter_options(player, &game.players()[usize::from(player.0)].hand),
            crossing: game.five_trump_crossing().map(|state| state.stage()),
            collecting_team: game.dealer()?.team().other(),
            collecting_score: game.collecting_score(),
        })
    }

    pub fn fallback(&self) -> ShengjiBotAction {
        match self.phase {
            Phase::Burying | Phase::BottomCopyBurying => {
                ShengjiBotAction::Bury(self.hand[..self.rules.kitty_size()].to_vec())
            }
            Phase::BottomCopying => ShengjiBotAction::Copy(self.counter_options.first().cloned()),
            Phase::FiveTrumpCrossing
                if self.crossing == Some(FiveTrumpCrossingStage::Returning) =>
            {
                ShengjiBotAction::Return(self.hand[..5].to_vec())
            }
            Phase::FiveTrumpCrossing => ShengjiBotAction::Cross(None),
            _ => ShengjiBotAction::Play(
                plays(
                    &self.hand,
                    self.trick
                        .as_ref()
                        .and_then(|trick| trick.plays.first())
                        .map(|(_, play)| play),
                    self.trump,
                    1,
                )[0]
                .cards
                .clone(),
            ),
        }
    }
}

pub(super) fn category_index(category: Category) -> usize {
    match category {
        Category::Trump => 4,
        Category::Suit(suit) => suit as usize,
        Category::Mixed => unreachable!("mixed cannot lead"),
    }
}
