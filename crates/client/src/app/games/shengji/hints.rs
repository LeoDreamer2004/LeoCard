//! 双升出牌提示的候选生成与轮换。

use super::*;
use leocard_protocol::ShengjiSnapshot;
use leocard_shengji::follow_suggestions;
use leocard_shengji::{ShengjiCard, ShengjiGreedyBot, ShengjiGreedyBotRequest, ShengjiSuit};

pub fn next_shengji_hint(
    game: &ShengjiSnapshot,
    selected: &HashSet<ShengjiCard>,
) -> Option<Vec<ShengjiCard>> {
    const MAX_HINTS: usize = 64;

    if game.current_player != Some(game.you) {
        return None;
    }
    let trump = game.trump?;
    let lead = game
        .trick
        .as_ref()
        .and_then(|trick| trick.plays.first())
        .map(|play| &play.play);
    let mut candidates = if let Some(lead) = lead {
        follow_suggestions(&game.your_hand, lead, trump, MAX_HINTS)
            .into_iter()
            .map(|play| play.cards)
            .collect::<Vec<_>>()
    } else {
        let mut cards = game.your_hand.clone();
        cards.sort_by_key(|card| {
            (
                trump.is_trump(*card),
                trump.strength(*card),
                card.suit().map_or(4, ShengjiSuit::bid_strength),
                card.deck(),
            )
        });
        cards.into_iter().map(|card| vec![card]).collect()
    };
    if candidates.is_empty()
        && let Ok(play) = ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
            hand: &game.your_hand,
            lead,
            trump,
        })
    {
        candidates.push(play.cards);
    }
    let current = candidates.iter().position(|cards| {
        cards.len() == selected.len() && cards.iter().all(|card| selected.contains(card))
    });
    let next = current.map_or(0, |index| (index + 1) % candidates.len());
    candidates.into_iter().nth(next)
}
