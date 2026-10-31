use super::candidates::{combinations, groups, unique};
use super::simulation::Simulation;
use crate::{
    ShengjiClassifiedPlay, ShengjiTeamId, ShengjiTrickPlay, classify_lead, validate_follow,
};
use leocard_game_common::{PlayerId, SearchBudget};

/// Exact two-team minimax on a sampled world, not on the real hidden hands.
/// Incomplete solves are discarded for every root candidate in that world.
pub(super) fn solve(
    position: &Simulation,
    player: PlayerId,
    budget: &SearchBudget,
    nodes: &mut usize,
) -> Option<f64> {
    search(
        position,
        player,
        budget,
        nodes,
        f64::NEG_INFINITY,
        f64::INFINITY,
    )
}

fn search(
    position: &Simulation,
    player: PlayerId,
    budget: &SearchBudget,
    nodes: &mut usize,
    mut alpha: f64,
    mut beta: f64,
) -> Option<f64> {
    if budget.exhausted() || *nodes == 0 {
        return None;
    }
    *nodes -= 1;
    if position.finished() {
        return Some(position.utility(player));
    }
    let maximizing = ShengjiTeamId::of(position.actor) == ShengjiTeamId::of(player);
    let mut value = if maximizing {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };
    for play in legal_plays(position) {
        let mut next = position.clone();
        next.play(play);
        let score = search(&next, player, budget, nodes, alpha, beta)?;
        if maximizing {
            value = value.max(score);
            alpha = alpha.max(value);
        } else {
            value = value.min(score);
            beta = beta.min(value);
        }
        if alpha >= beta {
            break;
        }
    }
    value.is_finite().then_some(value)
}

pub(super) fn legal_plays(position: &Simulation) -> Vec<ShengjiClassifiedPlay> {
    let hand = &position.hands[usize::from(position.actor.0)];
    let faces = groups(hand);
    let mut plays = Vec::new();
    if let Some(trick) = &position.trick {
        combinations(&faces, trick.plays[0].1.cards.len(), 256, &mut |cards| {
            if let Ok(play) = validate_follow(hand, cards, &trick.plays[0].1, position.trump) {
                plays.push(play);
            }
        });
    } else {
        let opponents: Vec<_> = position
            .hands
            .iter()
            .enumerate()
            .filter(|(seat, _)| *seat != usize::from(position.actor.0))
            .map(|(_, hand)| hand.as_slice())
            .collect();
        for count in 1..=hand.len() {
            combinations(&faces, count, 256, &mut |cards| {
                if let Ok(ShengjiTrickPlay::Accepted(play)) =
                    classify_lead(cards, position.trump, &position.rules, &opponents)
                {
                    plays.push(play);
                }
            });
        }
    }
    unique(&mut plays);
    plays
}
