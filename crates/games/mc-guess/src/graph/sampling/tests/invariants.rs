use super::super::super::data::{GraphData, GraphEdge};
use super::super::index::GraphIndex;
use super::super::mesh::{Sample, Topology};
use super::super::temperature::Temperature;
use crate::{McGuessGraphRelation, McGuessGraphRules, McGuessRuleSet};
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

type EdgeKey = (usize, usize, McGuessGraphRelation, bool);

fn edge_key(edge: &GraphEdge) -> EdgeKey {
    (edge.from, edge.to, edge.kind, edge.directed)
}

fn real_edges() -> &'static HashSet<EdgeKey> {
    static EDGES: OnceLock<HashSet<EdgeKey>> = OnceLock::new();
    EDGES.get_or_init(|| GraphData::bundled().edges.iter().map(edge_key).collect())
}

fn check_temperature(temperature: u8) {
    let real_edges = real_edges();
    for nodes in [16, 40] {
        for include_tags in [false, true] {
            let rules = McGuessGraphRules {
                temperature,
                nodes,
                include_tags,
                ..Default::default()
            };
            let sample = Sample::generate(rules, 27).expect("playable mesh at every step");
            let index = GraphIndex::bundled(include_tags);
            let topology = Topology::new(index, &sample);
            assert_eq!(sample.nodes.len() - topology.tags, usize::from(nodes));
            assert!(
                topology.tags
                    <= if include_tags {
                        usize::from(nodes / 8)
                    } else {
                        0
                    }
            );
            let mut pairs = HashSet::new();
            for edge in &sample.edges {
                assert_ne!(edge.from, edge.to);
                assert!(sample.nodes.contains(&edge.from) && sample.nodes.contains(&edge.to));
                assert!(pairs.insert((edge.from.min(edge.to), edge.from.max(edge.to))));
                assert!(real_edges.contains(&edge_key(edge)));
            }
            for &node in &sample.nodes {
                let degree = topology.degree(node);
                assert!((1..=Temperature::new(temperature).degree_limit()).contains(&degree));
                if index.data.nodes[node].entry.is_none() || temperature == 0 {
                    assert!(degree >= 2);
                }
            }
            assert!(topology.leaves <= Temperature::new(temperature).leaf_limit(nodes.into()));
            let mut reached = HashSet::from([sample.nodes[0]]);
            let mut queue = VecDeque::from([sample.nodes[0]]);
            while let Some(node) = queue.pop_front() {
                for edge in &sample.edges {
                    let other = if edge.from == node {
                        edge.to
                    } else if edge.to == node {
                        edge.from
                    } else {
                        continue;
                    };
                    if reached.insert(other) {
                        queue.push_back(other);
                    }
                }
            }
            assert_eq!(reached.len(), sample.nodes.len());
        }
    }
}

macro_rules! temperature_cases {
    ($($name:ident: $temperature:literal),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                check_temperature($temperature);
            }
        )+
    };
}

temperature_cases! {
    temperature_0: 0,
    temperature_1: 1,
    temperature_2: 2,
    temperature_3: 3,
    temperature_4: 4,
    temperature_5: 5,
    temperature_6: 6,
    temperature_7: 7,
    temperature_8: 8,
    temperature_9: 9,
    temperature_10: 10,
}

#[test]
fn fixed_seed_repeats_cold_and_hot_meshes() {
    // 完整结构矩阵由温度用例覆盖，重复生成只在冷、热端点抽查。
    for (temperature, include_tags) in [(0, false), (10, true)] {
        let rules = McGuessGraphRules {
            nodes: 40,
            temperature,
            include_tags,
            ..Default::default()
        };
        let sample = Sample::generate(rules, 27).unwrap();
        let repeated = Sample::generate(rules, 27).unwrap();
        assert_eq!(sample.nodes, repeated.nodes);
        assert_eq!(
            sample.edges.iter().map(edge_key).collect::<Vec<_>>(),
            repeated.edges.iter().map(edge_key).collect::<Vec<_>>()
        );
    }
}

#[test]
fn temperatures_above_the_supported_range_are_rejected() {
    let mut invalid = McGuessGraphRules {
        temperature: 11,
        ..Default::default()
    };
    assert!(
        !McGuessRuleSet {
            graph: invalid,
            ..Default::default()
        }
        .is_valid()
    );
    assert!(Sample::generate(invalid, 0).is_none());
    invalid.temperature = 10;
    assert!(
        McGuessRuleSet {
            graph: invalid,
            ..Default::default()
        }
        .is_valid()
    );
}
