use crate::{Action, Card, GameState, Phase, Play, PlayKind, RuleSet, classify};
use std::collections::HashSet;

/// Uses only the player's hand, public table and public team membership.
pub fn bot_action(game: &GameState) -> Option<Action> {
    let hand = game.hand(game.current)?;
    match game.phase {
        Phase::Bidding => {
            let power: usize = hand
                .iter()
                .map(|c| if c.rank() >= 12 { 2 } else { 0 })
                .sum();
            let bid = if power >= 8 {
                3
            } else if power >= 5 {
                2
            } else if power >= 3 {
                1
            } else {
                0
            };
            Some(Action::Bid(if bid > game.highest_bid { bid } else { 0 }))
        }
        Phase::Playing => {
            let play = hint(hand, game.table.as_ref().map(|(_, p)| p), game.rules);
            if let Some((leader, _)) = &game.table
                && Some(*leader) != game.landlord
                && Some(game.current) != game.landlord
                && play.as_ref().is_none_or(|p| p.cards.len() != hand.len())
            {
                return Some(Action::Pass);
            }
            Some(play.map_or(Action::Pass, |p| Action::Play(p.cards)))
        }
        Phase::Redeal | Phase::Finished => None,
    }
}

/// Manual auto-play and disconnected-player takeover currently share the bot strategy.
pub fn auto_play_action(game: &GameState) -> Option<Action> {
    bot_action(game)
}

pub fn hint(hand: &[Card], table: Option<&Play>, rules: RuleSet) -> Option<Play> {
    hint_options(hand, table, rules).into_iter().next()
}

/// Distinct suggested responses, ordered by the same priorities used by bots.
pub fn hint_options(hand: &[Card], table: Option<&Play>, rules: RuleSet) -> Vec<Play> {
    let mut groups = vec![Vec::new(); 15];
    for &card in hand {
        groups[usize::from(card.rank())].push(card);
    }
    let mut candidates = Vec::new();
    let mut add = |cards: Vec<Card>| {
        if let Some(play) = classify(&cards, rules)
            && table.is_none_or(|previous| play.beats(previous))
        {
            candidates.push(play);
        }
    };
    add(hand.to_vec());
    for rank in 0..15 {
        for size in 1..=groups[rank].len() {
            add(groups[rank][..size].to_vec());
        }
        if rank < 13 && groups[rank].len() >= 3 {
            if rules.allows_single_wings() {
                attachments(&groups, rank, rank + 1, 3, 1, 1, &mut add);
            }
            attachments(&groups, rank, rank + 1, 3, 2, 1, &mut add);
        }
        if rank < 13 && groups[rank].len() >= 4 {
            attachments(&groups, rank, rank + 1, 4, 1, 2, &mut add);
            attachments(&groups, rank, rank + 1, 4, 2, 2, &mut add);
        }
    }
    let mut rocket = groups[13].clone();
    rocket.extend_from_slice(&groups[14]);
    add(rocket);
    for (width, minimum) in [(1, 5), (2, 3), (3, 2)] {
        for start in 0..12 {
            for end in start + minimum..=12 {
                if !groups[start..end].iter().all(|g| g.len() >= width) {
                    break;
                }
                add(groups[start..end]
                    .iter()
                    .flat_map(|g| g[..width].iter().copied())
                    .collect());
                if width == 3 {
                    if rules.allows_single_wings() {
                        attachments(&groups, start, end, 3, 1, end - start, &mut add);
                    }
                    attachments(&groups, start, end, 3, 2, end - start, &mut add);
                }
            }
        }
    }
    candidates.sort_by_key(|p| {
        (
            p.cards.len() != hand.len(),
            matches!(p.kind, PlayKind::Bomb | PlayKind::Rocket),
            if table.is_some() {
                p.cards.len()
            } else {
                hand.len() - p.cards.len()
            },
            p.rank,
        )
    });
    let mut seen = HashSet::new();
    candidates.retain(|play| seen.insert(play.cards.clone()));
    candidates
}

fn attachments(
    groups: &[Vec<Card>],
    start: usize,
    end: usize,
    body: usize,
    wing: usize,
    count: usize,
    add: &mut impl FnMut(Vec<Card>),
) {
    let base: Vec<_> = groups[start..end]
        .iter()
        .flat_map(|g| g[..body].iter().copied())
        .collect();
    let available: Vec<_> = groups
        .iter()
        .enumerate()
        .filter(|(r, g)| !(start..end).contains(r) && g.len() >= wing)
        .collect();
    // Each rotation offers a different low-cost wing selection without exponential search.
    for offset in 0..available.len() {
        let mut cards = base.clone();
        let mut needed = count;
        for step in 0..available.len() {
            let (_, group) = available[(offset + step) % available.len()];
            let take = if wing == 1 {
                group.len().min(needed)
            } else {
                2
            };
            cards.extend_from_slice(&group[..take]);
            needed -= if wing == 1 { take } else { 1 };
            if needed == 0 {
                add(cards);
                break;
            }
        }
    }
}
