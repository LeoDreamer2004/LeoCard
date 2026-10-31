use super::ShengjiBotObservation;
use super::candidates::{hand_value, plays};
use super::evaluation::quick_value;
use super::observation::category_index;
use crate::ShengjiTeamId;
use crate::play::category;
use crate::{
    ShengjiCard, ShengjiClassifiedPlay, ShengjiTrickRecord, ShengjiTrump, compare_for_trick,
};
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use std::cmp::Ordering;

pub(super) fn choose_play(
    observation: &ShengjiBotObservation,
    budget: &SearchBudget,
) -> Vec<ShengjiCard> {
    let lead = observation
        .trick
        .as_ref()
        .and_then(|trick| trick.plays.first())
        .map(|(_, play)| play);
    let mut choices = plays(&observation.hand, lead, observation.trump, 64);
    choices.sort_by(|left, right| {
        quick_value(
            right,
            &observation.hand,
            observation.trick.as_ref(),
            observation.player,
            observation.trump,
        )
        .total_cmp(&quick_value(
            left,
            &observation.hand,
            observation.trick.as_ref(),
            observation.player,
            observation.trump,
        ))
    });
    choices.truncate(12);
    let mut best = choices[0].cards.clone();
    let mut totals = vec![0.0; choices.len()];
    let seed = observation.hand.iter().fold(
        observation.unseen.len() as u64 + u64::from(observation.player.0),
        |seed, card| {
            seed.wrapping_mul(59)
                .wrapping_add(card.rank() as u64 * 4 + card.suit().map_or(4, |suit| suit as u64))
        },
    );
    let mut random = SearchRandom::new(seed);
    let mut samples = 0;
    for _ in 0..96 {
        if budget.exhausted() {
            break;
        }
        let Some((hands, bottom_points)) = sample(observation, &mut random) else {
            continue;
        };
        let mut values = Vec::new();
        for candidate in &choices {
            if budget.exhausted() {
                return best;
            }
            let Some(value) = rollout(
                observation,
                hands.clone(),
                candidate.clone(),
                bottom_points,
                budget,
            ) else {
                return best;
            };
            values.push(value);
        }
        samples += 1;
        for (total, value) in totals.iter_mut().zip(values) {
            *total += value;
        }
        let index = totals
            .iter()
            .enumerate()
            .max_by(|(left_index, left), (right_index, right)| {
                let value = |index: usize, sum: f64| {
                    sum / f64::from(samples)
                        + quick_value(
                            &choices[index],
                            &observation.hand,
                            observation.trick.as_ref(),
                            observation.player,
                            observation.trump,
                        ) * 0.15
                };
                value(*left_index, **left).total_cmp(&value(*right_index, **right))
            })
            .expect("candidate")
            .0;
        best = choices[index].cards.clone();
    }
    best
}

fn sample(
    observation: &ShengjiBotObservation,
    random: &mut SearchRandom,
) -> Option<(Vec<Vec<ShengjiCard>>, u16)> {
    for _ in 0..12 {
        let mut pool = observation.unseen.clone();
        let mut hands = observation.exposed.clone();
        hands[usize::from(observation.player.0)] = observation.hand.clone();
        let mut players: Vec<_> = (0..usize::from(observation.rules.player_count()))
            .filter(|index| *index != usize::from(observation.player.0))
            .collect();
        players.sort_by_key(|index| {
            pool.iter()
                .filter(|card| {
                    !observation.voids[*index][category_index(category(**card, observation.trump))]
                })
                .count()
        });
        let mut valid = true;
        for player in players {
            if hands[player].len() > observation.lengths[player] {
                valid = false;
                break;
            }
            while hands[player].len() < observation.lengths[player] {
                let allowed: Vec<_> = pool
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| {
                        !observation.voids[player]
                            [category_index(category(**card, observation.trump))]
                    })
                    .map(|(index, _)| index)
                    .collect();
                if allowed.is_empty() {
                    valid = false;
                    break;
                }
                hands[player].push(pool.swap_remove(allowed[random.index(allowed.len())]));
            }
            if !valid {
                break;
            }
        }
        if valid {
            let bottom = observation.bottom.as_ref().unwrap_or(&pool);
            return Some((hands, bottom.iter().map(|card| card.points()).sum()));
        }
    }
    None
}

fn rollout(
    observation: &ShengjiBotObservation,
    mut hands: Vec<Vec<ShengjiCard>>,
    first: ShengjiClassifiedPlay,
    bottom_points: u16,
    budget: &SearchBudget,
) -> Option<f64> {
    let player = observation.player;
    let trump = observation.trump;
    let mut trick = observation.trick.clone().unwrap_or(ShengjiTrickRecord {
        leader: player,
        plays: Vec::new(),
        winner: player,
        points: 0,
    });
    append(&mut trick, player, first, &mut hands, trump);
    let mut value = 0.0;
    let mut collected = observation.collecting_score;
    for turn in 0..if observation.hand.len() <= 9 { 8 } else { 3 } {
        while trick.plays.len() < usize::from(observation.rules.player_count()) {
            if budget.exhausted() {
                return None;
            }
            let actor = PlayerId(
                (trick.leader.0 + trick.plays.len() as u8) % observation.rules.player_count(),
            );
            let candidates = plays(
                &hands[usize::from(actor.0)],
                Some(&trick.plays[0].1),
                trump,
                16,
            );
            let selected = candidates
                .into_iter()
                .max_by(|left, right| {
                    quick_value(
                        left,
                        &hands[usize::from(actor.0)],
                        Some(&trick),
                        actor,
                        trump,
                    )
                    .total_cmp(&quick_value(
                        right,
                        &hands[usize::from(actor.0)],
                        Some(&trick),
                        actor,
                        trump,
                    ))
                })
                .expect("legal follow");
            append(&mut trick, actor, selected, &mut hands, trump);
        }
        let sign = if ShengjiTeamId::of(trick.winner) == ShengjiTeamId::of(player) {
            1.0
        } else {
            -1.0
        };
        value += sign * (f64::from(trick.points) + 2.0) * 0.85f64.powi(turn);
        if ShengjiTeamId::of(trick.winner) == observation.collecting_team {
            collected += u32::from(trick.points);
        }
        if hands.iter().all(Vec::is_empty) {
            let winner = &trick
                .plays
                .iter()
                .find(|(player, _)| *player == trick.winner)
                .expect("winner")
                .1;
            if ShengjiTeamId::of(trick.winner) == observation.collecting_team {
                let award = u32::from(bottom_points) * winner.kitty_multiplier();
                collected += award;
                value += sign * f64::from(award);
            }
            break;
        }
        let leader = trick.winner;
        let candidates = plays(&hands[usize::from(leader.0)], None, trump, 32);
        let selected = candidates
            .into_iter()
            .max_by(|left, right| {
                quick_value(left, &hands[usize::from(leader.0)], None, leader, trump).total_cmp(
                    &quick_value(right, &hands[usize::from(leader.0)], None, leader, trump),
                )
            })
            .expect("leader cards");
        trick = ShengjiTrickRecord {
            leader,
            plays: Vec::new(),
            winner: leader,
            points: 0,
        };
        append(&mut trick, leader, selected, &mut hands, trump);
    }
    for (index, hand) in hands.iter().enumerate() {
        value += hand_value(hand, trump)
            * if index % 2 == usize::from(player.0 % 2) {
                0.35
            } else {
                -0.35
            };
    }
    let step = observation.rules.score_step();
    let crossed = collected / step - observation.collecting_score / step;
    value += f64::from(crossed)
        * if ShengjiTeamId::of(player) == observation.collecting_team {
            15.0
        } else {
            -15.0
        };
    Some(value)
}

fn append(
    trick: &mut ShengjiTrickRecord,
    player: PlayerId,
    play: ShengjiClassifiedPlay,
    hands: &mut [Vec<ShengjiCard>],
    trump: ShengjiTrump,
) {
    if let Some((_, lead)) = trick.plays.first() {
        let current = &trick
            .plays
            .iter()
            .find(|(player, _)| *player == trick.winner)
            .expect("winner")
            .1;
        if compare_for_trick(lead, current, &play, trump) == Ordering::Greater {
            trick.winner = player;
        }
    }
    hands[usize::from(player.0)].retain(|card| !play.cards.contains(card));
    trick.points += play.cards.iter().map(|card| card.points()).sum::<u16>();
    trick.plays.push((player, play));
}
