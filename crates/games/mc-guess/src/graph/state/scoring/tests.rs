use super::super::analysis::GraphAnalysis;
use super::{McGuessGraphGame, NodeValue};
use crate::graph::data::{GraphData, GraphEdge};
use crate::{
    McGuessAttributeFeedback, McGuessCatalogue, McGuessEntryKind, McGuessGraphGuessRecord,
    McGuessGraphRelation, McGuessGraphRules, McGuessMatchQuality, McGuessRecord, McGuessResult,
};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

#[test]
fn formula_uses_blindness_and_smooth_rarity() {
    let mut value = NodeValue {
        full_degree: 20,
        sub_degree: 10,
        local_revealed: 0,
        total: 20,
        revealed: 0,
    };
    assert!((value.score(1) - 3138.888888888889).abs() < 1e-9);
    value.local_revealed = 10;
    value.revealed = 20;
    assert!((value.score(1) - 1666.666666666667).abs() < 1e-9);
    value.full_degree = 1;
    value.sub_degree = 1;
    value.local_revealed = 1;
    assert!((value.score(1) - 2757.575757575758).abs() < 1e-9);
}

#[test]
fn combo_discount_recovers_at_six_and_caps_at_eight() {
    let value = NodeValue {
        full_degree: 2,
        sub_degree: 0,
        local_revealed: 0,
        total: 1,
        revealed: 0,
    };
    for (combo, expected) in [
        (1, 3640.0),
        (2, 2184.0),
        (3, 2548.0),
        (6, 3640.0),
        (7, 4004.0),
        (8, 4368.0),
        (40, 4368.0),
    ] {
        assert!((value.score(combo) - expected).abs() < 1e-9);
    }
}

#[test]
fn participation_counts_only_exact_attributes_and_handles_empty_or_unknown_entries() {
    use McGuessMatchQuality::{Exact, Partial, Wrong};

    let game =
        McGuessGraphGame::pending(vec![PlayerId(0), PlayerId(1)], McGuessGraphRules::default());
    for (qualities, expected) in [
        (vec![Exact, Exact, Exact, Partial, Wrong], 190),
        (vec![Partial, Wrong], 100),
        (vec![], 100),
    ] {
        let result = McGuessResult::Known {
            entry: McGuessCatalogue::bundled()
                .find(McGuessEntryKind::Block, "minecraft:stone")
                .unwrap()
                .clone(),
            matches: qualities
                .into_iter()
                .map(|quality| McGuessAttributeFeedback {
                    quality,
                    direction: None,
                })
                .collect(),
            correct: false,
        };
        assert_eq!(game.score_guess(&result, 8), expected);
    }
    assert_eq!(
        game.score_guess(
            &McGuessResult::Unknown {
                name: "未知".to_owned()
            },
            8
        ),
        0
    );
}

#[test]
fn topology_is_unique_and_scores_precede_answer_and_tag_reveals() {
    let data = GraphData::bundled();
    let mut game =
        McGuessGraphGame::pending(vec![PlayerId(0), PlayerId(1)], McGuessGraphRules::default());
    game.nodes = data
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.entry
                .filter(|entry| entry.kind == McGuessEntryKind::Block)
                .map(|_| index)
        })
        .take(3)
        .collect();
    game.nodes.push(
        data.nodes
            .iter()
            .position(|node| node.entry.is_none())
            .unwrap(),
    );
    game.revealed = vec![false, true, false, false];
    game.analysis = GraphAnalysis::new(&game.players, game.nodes.len());
    game.edges = [(0, 1), (1, 0), (0, 1), (0, 0), (0, 3), (1, 3)]
        .into_iter()
        .map(|(from, to)| GraphEdge {
            from: game.nodes[from],
            to: game.nodes[to],
            kind: McGuessGraphRelation::Other,
            directed: true,
        })
        .collect();
    let value = NodeValue::before_guess(&game);
    assert_eq!(
        (
            value.sub_degree,
            value.local_revealed,
            value.total,
            value.revealed
        ),
        (2, 1, 4, 1)
    );
    let source = game.nodes[0];
    let full_neighbours: HashSet<_> = data
        .edges
        .iter()
        .filter_map(|edge| {
            if edge.from == source && edge.to != source {
                Some(edge.to)
            } else if edge.to == source && edge.from != source {
                Some(edge.from)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(value.full_degree, full_neighbours.len());
    let answer = game.entry(0).unwrap().id.clone();
    game.guess(PlayerId(0), &answer).unwrap();
    assert_eq!(
        game.take_score_gains(),
        [(PlayerId(0), (value.score(1) * 2.0).round() as u32)]
    );
    assert_eq!(game.revealed, [true, true, false, true]);
    let after = NodeValue::before_guess(&game);
    assert_eq!((after.local_revealed, after.revealed), (2, 3));
}

#[test]
fn kind_weights_and_attempt_decay_apply_before_rounding_to_every_reward() {
    let data = GraphData::bundled();
    for (kind, weight) in [
        (McGuessEntryKind::Block, 1.0),
        (McGuessEntryKind::Item, 1.0),
        (McGuessEntryKind::Mob, 2.0 / 3.0),
        (McGuessEntryKind::Enchantment, 2.0 / 3.0),
        (McGuessEntryKind::Effect, 2.0 / 3.0),
        (McGuessEntryKind::Landscape, 2.0 / 3.0),
        (McGuessEntryKind::Biome, 2.0 / 3.0),
        (McGuessEntryKind::VillagerProfession, 0.5),
    ] {
        let mut game =
            McGuessGraphGame::pending(vec![PlayerId(0), PlayerId(1)], McGuessGraphRules::default());
        let index = data
            .nodes
            .iter()
            .position(|node| node.entry.is_some_and(|entry| entry.kind == kind))
            .unwrap();
        game.nodes = vec![index, (index + 1) % data.nodes.len()];
        game.revealed = vec![false, false];
        game.analysis = GraphAnalysis::new(&game.players, game.nodes.len());
        let value = NodeValue::before_guess(&game);
        let mut result = McGuessResult::Known {
            entry: game.entry(0).unwrap().clone(),
            matches: vec![McGuessAttributeFeedback {
                quality: McGuessMatchQuality::Exact,
                direction: None,
            }],
            correct: false,
        };
        assert_eq!(
            game.score_guess(&result, 0),
            (value.score(1) * weight).round() as u32
        );
        game.guesses.push(McGuessGraphGuessRecord {
            node: 1,
            guess: McGuessRecord {
                player: PlayerId(0),
                result: result.clone(),
            },
            combo: 0,
        });
        assert_eq!(
            game.score_guess(&result, 0),
            (value.score(1) * 0.3 * weight * 0.95).round() as u32
        );
        if let McGuessResult::Known { correct, .. } = &mut result {
            *correct = true;
        }
        assert_eq!(
            game.score_guess(&result, 8),
            (value.score(8) * 2.0 * weight * 0.95).round() as u32
        );
        if let McGuessResult::Known {
            correct, matches, ..
        } = &mut result
        {
            *correct = false;
            matches[0].quality = McGuessMatchQuality::Wrong;
        }
        game.guesses.push(McGuessGraphGuessRecord {
            node: 2,
            guess: McGuessRecord {
                player: PlayerId(1),
                result: result.clone(),
            },
            combo: 0,
        });
        assert_eq!(
            game.score_guess(&result, 0),
            (100.0 * weight * 0.95).round() as u32
        );
        game.guesses.push(McGuessGraphGuessRecord {
            node: 1,
            guess: McGuessRecord {
                player: PlayerId(1),
                result: McGuessResult::Unknown {
                    name: "未知".to_owned(),
                },
            },
            combo: 0,
        });
        assert_eq!(
            game.score_guess(&result, 0),
            (100.0 * weight * 0.95_f64.powi(2)).round() as u32
        );
    }
}
