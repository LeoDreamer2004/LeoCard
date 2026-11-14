use super::index::GraphIndex;
use super::mesh::{Sample, Topology};
use super::temperature::Temperature;
use crate::McGuessGraphRules;
use fastrand::Rng;
use std::collections::HashSet;
use std::time::Instant;

mod diversity;
mod invariants;

#[test]
fn hot_neighbour_candidates_favor_low_source_degree() {
    let index = GraphIndex::bundled(true);
    let hub = *index
        .starts
        .iter()
        .max_by_key(|&&node| index.data.degree(node))
        .unwrap();
    let mut total_degrees = [0, 0];
    for (slot, temperature) in [0, 10].into_iter().enumerate() {
        let mut random = Rng::with_seed(523);
        for _ in 0..256 {
            let neighbours =
                index.neighbours(hub, 8, false, Temperature::new(temperature), &mut random);
            assert_eq!(neighbours.len(), 8);
            assert_eq!(neighbours.iter().copied().collect::<HashSet<_>>().len(), 8);
            for node in neighbours {
                assert!(index.adjacent[hub].contains(&node));
                total_degrees[slot] += index.data.degree(node);
            }
        }
    }
    assert!(total_degrees[1] < total_degrees[0]);
}

#[derive(Default, Debug)]
struct Metrics {
    degree_variance: f32,
    leaves: usize,
    ring_spread: f32,
    max_degree: usize,
}

#[test]
fn hotter_samples_have_more_uneven_degrees_leaves_and_ring_sizes() {
    let mut totals = [Metrics::default(), Metrics::default()];
    let mut longest = 0;
    for (slot, temperature) in [0, 10].into_iter().enumerate() {
        for seed in 0..12 {
            let start = Instant::now();
            let sample = Sample::generate(
                McGuessGraphRules {
                    nodes: 40,
                    temperature,
                    ..Default::default()
                },
                seed,
            )
            .expect("temperature profile can fill 40 nodes");
            longest = longest.max(start.elapsed().as_millis());
            let topology = Topology::new(GraphIndex::bundled(true), &sample);
            let average = sample.edges.len() as f32 * 2.0 / sample.nodes.len() as f32;
            totals[slot].degree_variance += sample
                .nodes
                .iter()
                .map(|&node| (topology.degree(node) as f32 - average).powi(2))
                .sum::<f32>()
                / sample.nodes.len() as f32;
            totals[slot].leaves += topology.leaves;
            totals[slot].max_degree = totals[slot].max_degree.max(
                sample
                    .nodes
                    .iter()
                    .map(|&node| topology.degree(node))
                    .max()
                    .unwrap(),
            );
            let mean = sample.cycles.iter().sum::<usize>() as f32 / sample.cycles.len() as f32;
            totals[slot].ring_spread += sample
                .cycles
                .iter()
                .map(|&size| (size as f32 - mean).powi(2))
                .sum::<f32>()
                / sample.cycles.len() as f32;
        }
    }
    eprintln!("cold/hot totals for 12 seeds: {totals:?}; longest generation {longest} ms");
    assert!(totals[1].degree_variance > totals[0].degree_variance);
    assert!(totals[1].leaves > totals[0].leaves);
    assert!(totals[1].max_degree > totals[0].max_degree);
    assert!(totals[1].ring_spread > totals[0].ring_spread);
}
