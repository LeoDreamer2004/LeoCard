use super::super::{McGuessBot, McGuessBotAction, McGuessBotObservation};
use crate::{
    McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessGraphEdgeView,
    McGuessGraphGuessRecord, McGuessGraphNodeView, McGuessGraphRelation, McGuessGraphRules,
    McGuessGraphView, McGuessMode, McGuessRecord, McGuessResult, McGuessRuleSet,
};
use fastrand::Rng;
use leocard_game_common::PlayerId;

fn node(number: u16, kind: McGuessEntryKind, revealed: Option<&str>) -> McGuessGraphNodeView {
    McGuessGraphNodeView {
        number,
        kind: Some(kind),
        character_count: None,
        entry: revealed.map(|id| McGuessCatalogue::bundled().find(kind, id).unwrap().clone()),
        tag_name: None,
    }
}

fn edge(source: u16, target: u16, relation: McGuessGraphRelation) -> McGuessGraphEdgeView {
    McGuessGraphEdgeView {
        source,
        target,
        relation,
        directed: Some(true),
        active: false,
    }
}

fn glass_graph() -> McGuessGraphView {
    McGuessGraphView {
        nodes: vec![
            node(1, McGuessEntryKind::Block, None),
            node(2, McGuessEntryKind::Block, Some("minecraft:glass")),
        ],
        edges: vec![edge(1, 2, McGuessGraphRelation::Processing)],
        selected: 1,
        combo: None,
        guesses: Vec::new(),
    }
}

#[test]
fn revealed_neighbours_intersect_their_relationship_candidates() {
    let mut graph = glass_graph();
    let candidates = graph.candidates_from_revealed(1).unwrap();
    assert!(candidates.contains("minecraft:sand"));
    assert!(candidates.contains("minecraft:red_sand"));
    graph.nodes.push(node(
        3,
        McGuessEntryKind::Block,
        Some("minecraft:sandstone"),
    ));
    graph.edges.push(edge(1, 3, McGuessGraphRelation::Crafting));
    let observation = McGuessBotObservation::from_graph(&graph, true).unwrap();
    assert_eq!(observation.related_entries.as_ref().unwrap().len(), 1);
    assert_eq!(
        McGuessBot::choose(&observation, &mut Rng::with_seed(7)),
        Some(McGuessBotAction::Guess {
            node: Some(1),
            entry_id: "minecraft:sand".to_owned(),
        })
    );
}

#[test]
fn hidden_neighbour_metadata_and_relationships_cannot_change_a_decision() {
    let mut graph = glass_graph();
    graph.nodes.push(node(3, McGuessEntryKind::Item, None));
    graph.nodes[2].character_count = Some(2);
    graph.edges.push(edge(1, 3, McGuessGraphRelation::Crafting));
    let before = McGuessBotObservation::from_graph(&graph, true).unwrap();
    graph.nodes[2].kind = Some(McGuessEntryKind::Mob);
    graph.nodes[2].character_count = Some(12);
    graph.edges[1] = edge(3, 1, McGuessGraphRelation::Loot);
    graph.edges[1].directed = None;
    let after = McGuessBotObservation::from_graph(&graph, true).unwrap();
    assert_eq!(before, after);
    assert_eq!(
        McGuessBot::choose(&before, &mut Rng::with_seed(19)),
        McGuessBot::choose(&after, &mut Rng::with_seed(19))
    );
    graph.nodes[1].entry = None;
    assert!(
        McGuessBotObservation::from_graph(&graph, true)
            .unwrap()
            .related_entries
            .is_none()
    );
}

#[test]
fn hidden_direction_and_coarse_relations_do_not_reveal_extra_information() {
    let mut graph = glass_graph();
    graph.nodes[1] = node(2, McGuessEntryKind::Block, Some("minecraft:stone"));
    graph.edges[0].directed = None;
    let hidden = graph.candidates_from_revealed(1).unwrap();
    assert!(hidden.contains("minecraft:cobblestone"));
    assert!(hidden.contains("minecraft:stone_bricks"));
    assert!(hidden.contains("minecraft:smooth_stone"));
    graph.edges[0].source = 2;
    graph.edges[0].target = 1;
    assert_eq!(hidden, graph.candidates_from_revealed(1).unwrap());
    graph.edges[0].directed = Some(true);
    let outgoing = graph.candidates_from_revealed(1).unwrap();
    assert!(outgoing.contains("minecraft:stone_bricks"));
    graph.edges[0].source = 1;
    graph.edges[0].target = 2;
    let incoming = graph.candidates_from_revealed(1).unwrap();
    assert_eq!(incoming.len(), 1);
    assert!(incoming.contains("minecraft:cobblestone"));
}

#[test]
fn bidirectional_conflicts_and_revealed_tags_are_searchable() {
    let mut graph = glass_graph();
    graph.nodes[0] = node(1, McGuessEntryKind::Enchantment, None);
    graph.nodes[1] = node(
        2,
        McGuessEntryKind::Enchantment,
        Some("minecraft:sharpness"),
    );
    graph.edges[0].relation = McGuessGraphRelation::ConflictsWith;
    graph.edges[0].directed = Some(false);
    let conflicts = graph.candidates_from_revealed(1).unwrap();
    assert!(conflicts.contains("minecraft:smite"));
    assert!(conflicts.contains("minecraft:bane_of_arthropods"));
    assert!(!conflicts.contains("minecraft:unbreaking"));
    graph.nodes[0] = node(1, McGuessEntryKind::Block, None);
    graph.nodes[1] = McGuessGraphNodeView {
        number: 2,
        kind: None,
        character_count: None,
        entry: None,
        tag_name: Some("木板".to_owned()),
    };
    graph.edges[0] = edge(1, 2, McGuessGraphRelation::MemberOf);
    let members = graph.candidates_from_revealed(1).unwrap();
    assert!(members.contains("minecraft:oak_planks"));
    assert!(!members.contains("minecraft:oak_log"));
}

#[test]
fn relationship_search_still_respects_public_wordle_feedback() {
    let mut graph = glass_graph();
    let answer = McGuessCatalogue::bundled()
        .find(McGuessEntryKind::Block, "minecraft:sand")
        .unwrap();
    graph.guesses.push(McGuessGraphGuessRecord {
        node: 1,
        guess: McGuessRecord {
            player: PlayerId(0),
            result: McGuessResult::resolve(answer, "minecraft:red_sand", false).unwrap(),
        },
        combo: 0,
    });
    let observation = McGuessBotObservation::from_graph(&graph, true).unwrap();
    assert_eq!(
        McGuessBot::choose(&observation, &mut Rng::with_seed(4)),
        Some(McGuessBotAction::Guess {
            node: Some(1),
            entry_id: "minecraft:sand".to_owned(),
        })
    );
}

#[test]
fn public_relationship_search_never_excludes_the_real_sampled_answer() {
    for show_directions in [false, true] {
        let rules = McGuessRuleSet {
            mode: McGuessMode::Graph,
            graph: McGuessGraphRules {
                show_directions,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut game =
            McGuessGameState::new_graph(vec![PlayerId(0), PlayerId(1)], rules, 5).unwrap();
        let mut solved = game.clone();
        solved.reveal_graph().unwrap();
        let answers = solved.graph_view().unwrap();
        game.guess(PlayerId(0), &answers.nodes[0].entry.as_ref().unwrap().id)
            .unwrap();
        let view = game.graph_view().unwrap();
        let observation = McGuessBotObservation::from_game(&game, PlayerId(0)).unwrap();
        assert!(observation.related_entries.is_some());
        for node in view
            .nodes
            .iter()
            .filter(|node| node.kind.is_some() && !node.revealed())
        {
            if let Some(candidates) = view.candidates_from_revealed(node.number) {
                let answer = answers.nodes[usize::from(node.number - 1)]
                    .entry
                    .as_ref()
                    .unwrap();
                assert!(candidates.contains(&answer.id), "{}", answer.name);
            }
        }
    }
}
