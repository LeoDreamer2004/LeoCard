use crate::{
    GuandanAction, GuandanCard, GuandanGameState, GuandanPlay, GuandanPlayKind,
    GuandanTributeAction, can_beat, classify,
};
use leocard_game_common::PlayerId;

pub fn auto_play_action(game: &GuandanGameState, player: PlayerId) -> Option<GuandanAction> {
    if game.current_player()? != player {
        return None;
    }
    let hand = game.player(player)?.hand();
    let level = game.rules().starting_level;
    match game.pending_tribute_action() {
        Some(GuandanTributeAction::Tribute) => {
            let card = hand.iter().copied().max_by_key(|card| {
                (
                    crate::play::rank_strength(card.rank, level),
                    card.suit,
                    card.deck,
                )
            })?;
            return Some(GuandanAction::Tribute(card));
        }
        Some(GuandanTributeAction::ReturnTribute) => {
            let offered = game.pending_offered_card()?;
            let card = hand
                .iter()
                .copied()
                .filter(|card| *card != offered && card.rank <= 10)
                .min_by_key(|card| (card.rank, card.suit, card.deck))
                .or_else(|| {
                    hand.iter()
                        .copied()
                        .filter(|card| *card != offered)
                        .min_by_key(|card| crate::play::rank_strength(card.rank, level))
                })?;
            return Some(GuandanAction::ReturnTribute(card));
        }
        None => {}
    }
    let mut choices = candidate_plays(hand, level);
    let current = game.winning_play();
    choices.retain(|play| current.is_none_or(|current| can_beat(play, current, level)));
    choices.sort_unstable_by_key(|play| play_cost(play, level));
    choices
        .into_iter()
        .next()
        .map(|play| GuandanAction::Play(play.cards))
        .or_else(|| current.map(|_| GuandanAction::Pass))
}

/// One representative per playable pattern and strength, with ordinary plays
/// first when following and larger combinations first when leading.
pub fn hint_options(
    hand: &[GuandanCard],
    level: u8,
    current: Option<&GuandanPlay>,
) -> Vec<GuandanPlay> {
    let mut choices = candidate_plays(hand, level);
    let mixed_straights: Vec<_> = choices
        .iter()
        .filter(|play| play.kind == GuandanPlayKind::StraightFlush)
        .filter_map(|play| mixed_straight(hand, play, level))
        .collect();
    choices.extend(mixed_straights);
    choices.retain(|play| current.is_none_or(|current| can_beat(play, current, level)));
    choices.sort_by_key(|play| {
        (
            play_cost(play, level),
            play.cards.iter().filter(|card| card.is_wild(level)).count(),
        )
    });
    let mut unique = Vec::<GuandanPlay>::new();
    for play in choices {
        if !unique
            .iter()
            .any(|option| option.kind == play.kind && option.value == play.value)
        {
            unique.push(play);
        }
    }
    unique
}

// A flush can also supply a regular straight without spending a bomb.
fn mixed_straight(hand: &[GuandanCard], play: &GuandanPlay, level: u8) -> Option<GuandanPlay> {
    let (index, replacement) = play
        .cards
        .iter()
        .enumerate()
        .filter(|(_, card)| !card.is_wild(level))
        .find_map(|(index, card)| {
            hand.iter()
                .copied()
                .find(|other| {
                    other.rank == card.rank && other.suit != card.suit && !other.is_wild(level)
                })
                .map(|replacement| (index, replacement))
        })?;
    let mut cards = play.cards.clone();
    cards[index] = replacement;
    classify(&cards, level).ok()
}

fn candidate_plays(hand: &[GuandanCard], level: u8) -> Vec<GuandanPlay> {
    let wilds: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| card.is_wild(level))
        .collect();
    let mut candidates = Vec::new();
    for rank in 2..=17 {
        if rank == 15 {
            continue;
        }
        for len in 1..=10 {
            let shape = [(rank, len)];
            add_shape(hand, &wilds, level, &shape, None, &mut candidates);
        }
    }
    for triple in 2..=14 {
        for pair in 2..=17 {
            if triple != pair && pair != 15 {
                add_shape(
                    hand,
                    &wilds,
                    level,
                    &[(triple, 3), (pair, 2)],
                    None,
                    &mut candidates,
                );
            }
        }
    }
    for start in 1..=10 {
        let ranks = sequence_ranks(start, 5);
        let shape: Vec<_> = ranks.iter().map(|rank| (*rank, 1)).collect();
        add_shape(hand, &wilds, level, &shape, None, &mut candidates);
        for suit in [
            crate::GuandanSuit::Spades,
            crate::GuandanSuit::Hearts,
            crate::GuandanSuit::Clubs,
            crate::GuandanSuit::Diamonds,
        ] {
            add_shape(hand, &wilds, level, &shape, Some(suit), &mut candidates);
        }
    }
    for start in 1..=12 {
        let ranks = sequence_ranks(start, 3);
        let shape: Vec<_> = ranks.iter().map(|rank| (*rank, 2)).collect();
        add_shape(hand, &wilds, level, &shape, None, &mut candidates);
    }
    for start in 1..=13 {
        let ranks = sequence_ranks(start, 2);
        let shape: Vec<_> = ranks.iter().map(|rank| (*rank, 3)).collect();
        add_shape(hand, &wilds, level, &shape, None, &mut candidates);
    }
    let jokers: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| card.is_joker())
        .collect();
    if jokers.len() == 4
        && let Ok(play) = classify(&jokers, level)
    {
        candidates.push(play);
    }
    candidates
}

fn add_shape(
    hand: &[GuandanCard],
    wilds: &[GuandanCard],
    level: u8,
    shape: &[(u8, usize)],
    suit: Option<crate::GuandanSuit>,
    candidates: &mut Vec<GuandanPlay>,
) {
    let mut cards = Vec::new();
    for (rank, needed) in shape {
        let found: Vec<_> = hand
            .iter()
            .copied()
            .filter(|card| {
                !card.is_wild(level)
                    && card.rank == *rank
                    && suit.is_none_or(|suit| card.suit == suit)
            })
            .take(*needed)
            .collect();
        cards.extend(found);
    }
    let needed: usize = shape.iter().map(|(_, count)| count).sum();
    if cards.len() > needed || needed - cards.len() > wilds.len() {
        return;
    }
    cards.extend(wilds.iter().copied().take(needed - cards.len()));
    if let Ok(play) = classify(&cards, level) {
        candidates.push(play);
    }
}

fn sequence_ranks(start: u8, len: u8) -> Vec<u8> {
    (start..start + len)
        .map(|rank| if rank == 1 { 14 } else { rank })
        .collect()
}

fn play_cost(play: &GuandanPlay, level: u8) -> (u8, usize, u8) {
    let tier = match play.kind {
        GuandanPlayKind::Bomb(len) => len,
        GuandanPlayKind::StraightFlush => 6,
        GuandanPlayKind::JokerBomb => 12,
        _ => 0,
    };
    let value = match play.kind {
        GuandanPlayKind::Straight
        | GuandanPlayKind::StraightFlush
        | GuandanPlayKind::ConsecutivePairs
        | GuandanPlayKind::ConsecutiveTriples => play.value,
        _ => crate::play::rank_strength(play.value, level),
    };
    (tier, usize::MAX - play.cards.len(), value)
}

#[cfg(test)]
mod tests {
    use super::{candidate_plays, hint_options};
    use crate::{GuandanCard, GuandanPlayKind, GuandanSuit, build_deck, classify};

    #[test]
    fn bot_can_consider_the_four_joker_bomb() {
        let jokers = build_deck()
            .into_iter()
            .filter(|card| card.is_joker())
            .collect::<Vec<_>>();
        let choices = candidate_plays(&jokers, 2);
        assert!(
            choices
                .iter()
                .any(|play| play.kind == GuandanPlayKind::JokerBomb)
        );
    }

    #[test]
    fn hints_include_a_regular_straight_before_spending_a_flush() {
        let spade = |rank| GuandanCard {
            deck: 0,
            rank,
            suit: GuandanSuit::Spades,
        };
        let heart = |rank| GuandanCard {
            suit: GuandanSuit::Hearts,
            ..spade(rank)
        };
        let current = classify(&[spade(3), spade(4), spade(5), spade(6), heart(7)], 2).unwrap();
        let hand = [spade(6), spade(7), spade(8), spade(9), spade(10), heart(8)];
        let hints = hint_options(&hand, 2, Some(&current));
        assert_eq!(hints.first().unwrap().kind, GuandanPlayKind::Straight);
        assert!(
            hints
                .iter()
                .any(|hint| hint.kind == GuandanPlayKind::StraightFlush)
        );
    }
}
