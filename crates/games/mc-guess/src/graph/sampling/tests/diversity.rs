use super::super::index::GraphIndex;
use crate::McGuessEntryKind;

fn block(index: &GraphIndex, id: &str) -> usize {
    index
        .data
        .nodes
        .iter()
        .position(|node| {
            node.entry
                .is_some_and(|entry| entry.kind == McGuessEntryKind::Block && entry.id == id)
        })
        .unwrap()
}

#[test]
fn versions_are_balanced_against_the_whole_puzzle_instead_of_only_the_last_node() {
    let index = GraphIndex::bundled(true);
    let census = index.diversity.census([
        block(index, "minecraft:granite"),
        block(index, "minecraft:andesite"),
        block(index, "minecraft:stone"),
        block(index, "minecraft:andesite_stairs"),
    ]);
    let repeat = census.gain(&index.diversity, [block(index, "minecraft:diorite")]);
    let unseen = census.gain(&index.diversity, [block(index, "minecraft:tuff")]);
    assert!(unseen > repeat);
}

#[test]
fn color_and_material_variants_share_a_global_repetition_cost() {
    let index = GraphIndex::bundled(true);
    let wool = index.diversity.census([
        block(index, "minecraft:white_wool"),
        block(index, "minecraft:blue_wool"),
    ]);
    // These both have the same introduction version; only family repetition differs.
    assert!(
        wool.gain(&index.diversity, [block(index, "minecraft:stone")])
            > wool.gain(&index.diversity, [block(index, "minecraft:red_wool")])
    );
    let stone = index.diversity.census([block(index, "minecraft:andesite")]);
    // Both added in 1.8; polished andesite still belongs to the existing material family.
    assert!(
        stone.gain(&index.diversity, [block(index, "minecraft:granite")])
            > stone.gain(
                &index.diversity,
                [block(index, "minecraft:polished_andesite")]
            )
    );
}

#[test]
fn incremental_path_scoring_matches_the_final_global_objective() {
    let index = GraphIndex::bundled(true);
    let base = [
        block(index, "minecraft:white_wool"),
        block(index, "minecraft:granite"),
    ];
    let additions = [
        block(index, "minecraft:blue_wool"),
        block(index, "minecraft:red_wool"),
        block(index, "minecraft:tuff"),
    ];
    let census = index.diversity.census(base);
    let complete = index.diversity.census(base.into_iter().chain(additions));
    let expected = (complete.score() - census.score()) / additions.len() as f32;
    assert!((census.gain(&index.diversity, additions) - expected).abs() < 0.0001);
}
