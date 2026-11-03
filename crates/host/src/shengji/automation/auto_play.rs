use leocard_game_common::PlayerId;

use leocard_shengji::{
    ShengjiActionOutcome, ShengjiAutoPlay, ShengjiAutoPlayRequest, ShengjiBotAction,
    ShengjiBotObservation, ShengjiFiveTrumpCrossingStage, ShengjiGameState, ShengjiPhase,
};

pub(super) struct Policy;

impl Policy {
    pub fn act(game: &mut ShengjiGameState, player: PlayerId) -> Option<ShengjiActionOutcome> {
        let outcome = match game.phase() {
            ShengjiPhase::Burying | ShengjiPhase::BottomCopyBurying => {
                let ShengjiBotAction::Bury(cards) =
                    ShengjiBotObservation::from_game(game, player)?.fallback()
                else {
                    return None;
                };
                game.bury(player, &cards).ok()?
            }
            ShengjiPhase::BottomCopying => game.choose_bottom_copy(player, None).ok()?,
            ShengjiPhase::FiveTrumpCrossing => match game.five_trump_crossing()?.stage() {
                ShengjiFiveTrumpCrossingStage::Deciding => {
                    game.choose_five_trump_crossing(player, None).ok()?
                }
                ShengjiFiveTrumpCrossingStage::Returning => {
                    let ShengjiBotAction::Return(cards) =
                        ShengjiBotObservation::from_game(game, player)?.fallback()
                    else {
                        return None;
                    };
                    game.return_five_trump_crossing(player, &cards).ok()?
                }
            },
            ShengjiPhase::Playing => {
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
