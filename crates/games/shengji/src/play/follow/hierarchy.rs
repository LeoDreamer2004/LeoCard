use super::super::{Component, ShengjiClassifiedPlay};

#[derive(Clone, Copy, Debug)]
pub(crate) struct FollowPattern {
    pub(super) run_width: u8,
    pub(super) run_len: u8,
    pub(super) quads: u8,
    pub(super) triples: u8,
    pub(super) pairs: u8,
    pub(super) singles: u8,
    pub(super) wildcards: u8,
}

const fn pattern(
    run_width: u8,
    run_len: u8,
    quads: u8,
    triples: u8,
    pairs: u8,
    singles: u8,
) -> FollowPattern {
    FollowPattern {
        run_width,
        run_len,
        quads,
        triples,
        pairs,
        singles,
        wildcards: 0,
    }
}

const fn pattern_with_wildcards(run_width: u8, run_len: u8, wildcards: u8) -> FollowPattern {
    FollowPattern {
        run_width,
        run_len,
        quads: 0,
        triples: 0,
        pairs: 0,
        singles: 0,
        wildcards,
    }
}

pub(crate) fn special_follow_hierarchy(
    lead: &ShengjiClassifiedPlay,
) -> Option<Vec<Vec<FollowPattern>>> {
    let p = pattern;
    match lead.components.as_slice() {
        [Component::Quad { .. }] => Some(vec![
            vec![p(0, 0, 1, 0, 0, 0)],
            vec![p(2, 2, 0, 0, 0, 0)],
            vec![p(0, 0, 0, 0, 2, 0)],
            vec![p(0, 0, 0, 1, 0, 1)],
            vec![p(0, 0, 0, 0, 1, 2)],
            vec![p(0, 0, 0, 0, 0, 4)],
        ]),
        [Component::Tractor { pair_count: 2, .. }] => Some(vec![
            vec![p(2, 2, 0, 0, 0, 0), p(0, 0, 1, 0, 0, 0)],
            vec![p(0, 0, 0, 0, 2, 0)],
            vec![p(0, 0, 0, 0, 1, 2)],
            vec![p(0, 0, 0, 0, 0, 4)],
        ]),
        [Component::Tractor { pair_count: 3, .. }] => Some(vec![
            vec![p(2, 3, 0, 0, 0, 0), p(3, 2, 0, 0, 0, 0)],
            vec![p(0, 0, 1, 0, 1, 0)],
            vec![p(2, 2, 0, 0, 1, 0)],
            vec![p(0, 0, 0, 0, 3, 0)],
            vec![p(0, 0, 0, 0, 2, 2)],
            vec![p(0, 0, 0, 0, 1, 4)],
            vec![p(0, 0, 0, 0, 0, 6)],
        ]),
        [
            Component::Titanic {
                triple_count: 2, ..
            },
        ] => Some(vec![
            vec![p(3, 2, 0, 0, 0, 0)],
            vec![pattern_with_wildcards(2, 2, 2)],
            vec![p(0, 0, 0, 2, 0, 0)],
            vec![p(0, 0, 0, 1, 1, 1)],
            vec![p(0, 0, 0, 1, 0, 3)],
            vec![p(0, 0, 0, 0, 2, 2)],
            vec![p(0, 0, 0, 0, 1, 4)],
            vec![p(0, 0, 0, 0, 0, 6)],
        ]),
        [Component::Tractor { pair_count: 4, .. }] => Some(eight_card_tractor_hierarchy()),
        [Component::Spaceship { quad_count: 2, .. }]
        | [Component::Quad { .. }, Component::Quad { .. }] => Some(eight_card_quad_hierarchy()),
        _ => None,
    }
}

pub(crate) fn eight_card_tractor_hierarchy() -> Vec<Vec<FollowPattern>> {
    let p = pattern;
    vec![
        vec![p(4, 2, 0, 0, 0, 0)],
        vec![p(3, 2, 0, 0, 1, 0)],
        vec![p(3, 2, 0, 0, 0, 2)],
        vec![p(2, 2, 0, 0, 2, 0)],
        vec![p(2, 2, 0, 0, 1, 2)],
        vec![p(2, 2, 0, 0, 0, 4)],
        vec![p(0, 0, 1, 0, 2, 0)],
        vec![p(0, 0, 1, 0, 1, 2)],
        vec![p(0, 0, 1, 0, 0, 4)],
        vec![p(0, 0, 0, 2, 0, 2)],
        vec![p(0, 0, 0, 1, 2, 1)],
        vec![p(0, 0, 0, 1, 1, 3)],
        vec![p(0, 0, 0, 1, 0, 5)],
        vec![p(0, 0, 0, 0, 4, 0)],
        vec![p(0, 0, 0, 0, 3, 2)],
        vec![p(0, 0, 0, 0, 2, 4)],
        vec![p(0, 0, 0, 0, 1, 6)],
        vec![p(0, 0, 0, 0, 0, 8)],
    ]
}

fn eight_card_quad_hierarchy() -> Vec<Vec<FollowPattern>> {
    let p = pattern;
    vec![
        vec![p(4, 2, 0, 0, 0, 0)],
        vec![p(0, 0, 2, 0, 0, 0)],
        vec![p(3, 2, 0, 0, 1, 0)],
        vec![p(3, 2, 0, 0, 0, 2)],
        vec![p(2, 3, 0, 0, 1, 0)],
        vec![p(2, 3, 0, 0, 0, 2)],
        vec![p(2, 2, 0, 0, 2, 0)],
        vec![p(2, 2, 0, 0, 1, 2)],
        vec![p(2, 2, 0, 0, 0, 4)],
        vec![p(0, 0, 1, 0, 2, 0)],
        vec![p(0, 0, 1, 0, 1, 2)],
        vec![p(0, 0, 1, 0, 0, 4)],
        vec![p(0, 0, 0, 2, 0, 2)],
        vec![p(0, 0, 0, 1, 2, 1)],
        vec![p(0, 0, 0, 1, 1, 3)],
        vec![p(0, 0, 0, 1, 0, 5)],
        vec![p(0, 0, 0, 0, 4, 0)],
        vec![p(0, 0, 0, 0, 3, 2)],
        vec![p(0, 0, 0, 0, 2, 4)],
        vec![p(0, 0, 0, 0, 1, 6)],
        vec![p(0, 0, 0, 0, 0, 8)],
    ]
}
