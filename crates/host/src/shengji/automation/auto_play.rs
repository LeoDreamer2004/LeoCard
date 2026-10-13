use super::super::card_sort_key;
use leocard_shengji::{
    ActionOutcome, FiveTrumpCrossingStage, GameState, Phase, ShengjiAutoPlay,
    ShengjiAutoPlayRequest, ShengjiPlayerId,
};

pub(super) struct Policy;

impl Policy {
    pub fn act(game: &mut GameState, player: ShengjiPlayerId) -> Option<ActionOutcome> {
        let outcome = match game.phase() {
            Phase::Burying => {
                let trump = game.trump()?;
                let mut hand = game.players()[usize::from(player.0)].hand.clone();
                hand.sort_by_key(|card| card_sort_key(*card, trump));
                game.bury(player, &hand[..game.rules().kitty_size()]).ok()?
            }
            Phase::BottomCopying => game.choose_bottom_copy(player, None).ok()?,
            Phase::BottomCopyBurying => {
                let trump = game.trump()?;
                let mut hand = game.players()[usize::from(player.0)].hand.clone();
                hand.sort_by_key(|card| card_sort_key(*card, trump));
                game.bury(player, &hand[..game.rules().kitty_size()]).ok()?
            }
            Phase::FiveTrumpCrossing => match game.five_trump_crossing()?.stage() {
                FiveTrumpCrossingStage::Deciding => {
                    game.choose_five_trump_crossing(player, None).ok()?
                }
                FiveTrumpCrossingStage::Returning => {
                    let mut hand = game.players()[usize::from(player.0)].hand.clone();
                    let trump = game.trump()?;
                    hand.sort_by_key(|card| card_sort_key(*card, trump));
                    game.return_five_trump_crossing(player, &hand[..5]).ok()?
                }
            },
            Phase::Playing => {
                let trick = game.current_trick();
                let play = ShengjiAutoPlay::choose(ShengjiAutoPlayRequest {
                    hand: &game.players()[usize::from(player.0)].hand,
                    lead: trick.as_ref().map(|trick| &trick.plays[0].1),
                    trump: game.trump()?,
                })
                .ok()?;
                game.play_cards(player, &play.cards).ok()?
            }
            _ => return None,
        };
        Some(outcome)
    }
}
