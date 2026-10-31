use leocard_game_common::PlayerId;
use leocard_protocol::ShengjiTrickView;
use leocard_shengji::{ShengjiCategory, ShengjiClassifiedPlay, ShengjiGameState, ShengjiTrump};
use std::collections::HashMap;

/// 只使用已经公开且实际打出的牌，不读取玩家手牌或甩牌尝试。
pub(super) fn missing_suits(
    game: &ShengjiGameState,
    trick: Option<&ShengjiTrickView>,
) -> HashMap<PlayerId, Vec<ShengjiCategory>> {
    let mut result = HashMap::new();
    let Some(trump) = game.trump() else {
        return result;
    };
    for record in game.history() {
        record_missing(
            &mut result,
            trump,
            record.plays.iter().map(|(player, play)| (*player, play)),
        );
    }
    if let Some(trick) = trick {
        record_missing(
            &mut result,
            trump,
            trick.plays.iter().map(|play| (play.player, &play.play)),
        );
    }
    result
}

fn record_missing<'a>(
    result: &mut HashMap<PlayerId, Vec<ShengjiCategory>>,
    trump: ShengjiTrump,
    mut plays: impl Iterator<Item = (PlayerId, &'a ShengjiClassifiedPlay)>,
) {
    let Some((_, lead)) = plays.next() else {
        return;
    };
    let door = lead.category;
    if door == ShengjiCategory::Mixed {
        return;
    }
    for (player, play) in plays {
        if play.cards.iter().any(|card| {
            let category = if trump.is_trump(*card) {
                ShengjiCategory::Trump
            } else {
                ShengjiCategory::Suit(card.suit().expect("非主牌有花色"))
            };
            category != door
        }) {
            let missing = result.entry(player).or_default();
            if !missing.contains(&door) {
                missing.push(door);
            }
        }
    }
}
