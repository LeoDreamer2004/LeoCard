use super::arrangement::{HandArrangement, HandLayoutMode};
use super::hand_layout::GuandanHandLayout;
use super::hit_testing::{CardHitRegion, crossed_cards};
use super::state::GuandanUiState;
use crate::app::games::HandSelection;
use crate::app::presentation::{CardAnimationState, CardSize};
use bevy::prelude::*;
use leocard_guandan::{GuandanCard, GuandanSuit, build_deck};
use leocard_ui::CardDragSelection;
use std::collections::HashSet;

fn spades(ranks: &[u8]) -> Vec<GuandanCard> {
    ranks
        .iter()
        .map(|rank| GuandanCard {
            deck: 0,
            suit: GuandanSuit::Spades,
            rank: *rank,
        })
        .collect()
}

#[test]
fn manual_groups_preserve_physical_copies_and_sequence_order_across_the_level_rank() {
    let straight = spades(&[5, 6, 7, 8, 9]);
    let mut hand = straight.clone();
    hand.extend(straight.iter().map(|card| GuandanCard { deck: 1, ..*card }));
    let selected = straight.iter().copied().collect();
    let mut arrangement = HandArrangement::default();
    assert!(arrangement.group_selected(&selected, &hand, 6));
    let columns = arrangement.columns(&hand, 6);
    assert_eq!(
        columns
            .last()
            .unwrap()
            .cards
            .iter()
            .map(|card| card.rank)
            .collect::<Vec<_>>(),
        [9, 8, 7, 6, 5]
    );
    assert!(columns.last().unwrap().arranged);
    assert!(
        columns[..5]
            .iter()
            .all(|column| column.cards.len() == 1 && column.cards[0].deck == 1)
    );
    let all = columns
        .iter()
        .flat_map(|column| column.cards.iter().copied())
        .collect::<HashSet<_>>();
    assert_eq!(all, hand.iter().copied().collect());
    assert!(!arrangement.group_selected(&selected, &hand, 6));
    let invalid = [hand[0], hand[1]].into_iter().collect();
    assert!(!arrangement.group_selected(&invalid, &hand, 6));
    hand.retain(|card| *card != straight[4]);
    arrangement.reconcile(&hand, 6);
    assert!(!arrangement.has_groups());
}

#[test]
fn grouping_a_bomb_retains_a_legal_remainder_and_reset_keeps_the_layout_choice() {
    let mut hand = build_deck()
        .into_iter()
        .filter(|card| card.rank == 9)
        .take(4)
        .collect::<Vec<_>>();
    let selected = hand.iter().copied().collect();
    let mut arrangement = HandArrangement::default();
    assert!(arrangement.group_selected(&selected, &hand, 2));
    hand.pop();
    arrangement.reconcile(&hand, 2);
    assert_eq!(arrangement.columns(&hand, 2).last().unwrap().cards.len(), 3);
    assert!(arrangement.has_groups());
    arrangement.toggle_layout();
    arrangement.reset_groups();
    assert!(arrangement.mode == HandLayoutMode::Row);
    assert!(!arrangement.has_groups());
}

#[test]
fn original_size_layout_preserves_physical_cards_and_each_exposed_corner_remains_selectable() {
    let deck = build_deck();
    let mut bomb = deck
        .iter()
        .copied()
        .filter(|card| card.rank == 9 || card.is_wild(2))
        .collect::<Vec<_>>();
    let bomb_selection = bomb.iter().copied().collect();
    bomb.extend(
        deck.iter()
            .copied()
            .filter(|card| card.rank != 9 && !card.is_wild(2))
            .take(17),
    );
    for hand in [deck[..27].to_vec(), bomb, spades(&[14]), Vec::new()] {
        let mut arrangement = HandArrangement::default();
        arrangement.group_selected(&bomb_selection, &hand, 2);
        for mode in [HandLayoutMode::Columns, HandLayoutMode::Row] {
            let layout = GuandanHandLayout::new(&arrangement.columns(&hand, 2), mode);
            assert_eq!(layout.cards.len(), hand.len());
            assert_eq!(
                layout
                    .cards
                    .iter()
                    .map(|(card, _)| *card)
                    .collect::<HashSet<_>>(),
                hand.iter().copied().collect()
            );
            assert!(layout.size.x <= 940.0);
            let (width, height) = CardSize::ShengjiHand.dimensions();
            let regions = layout
                .cards
                .iter()
                .map(|(_, placement)| CardHitRegion {
                    index: placement.index,
                    rect: placement.hit_rect(),
                })
                .collect::<Vec<_>>();
            for (_, placement) in &layout.cards {
                assert_eq!(placement.size, Vec2::new(width, height));
                let corner = placement.position + Vec2::splat(3.0);
                assert_eq!(crossed_cards(corner, corner, &regions), [placement.index]);
                let pose = placement.pose(CardAnimationState {
                    face_hover_amount: 1.0,
                    selected_amount: 1.0,
                    ..default()
                });
                let (Val::Px(x), Val::Px(y)) = (pose.translation.x, pose.translation.y) else {
                    panic!("card offsets use pixels");
                };
                let moved = placement.position + Vec2::new(x, y);
                let hit = placement.hit_rect();
                assert!(hit.contains(moved) && hit.contains(moved + placement.size));
            }
            if mode == HandLayoutMode::Columns && arrangement.has_groups() {
                let group = layout
                    .cards
                    .iter()
                    .filter(|(_, placement)| placement.arranged)
                    .collect::<Vec<_>>();
                assert_eq!(group.len(), 10);
                assert!(
                    group
                        .iter()
                        .all(|(_, placement)| placement.column == group[0].1.column)
                );
                assert!(
                    group
                        .iter()
                        .any(|(_, placement)| placement.position.x != group[0].1.position.x)
                );
            }
        }
    }
}

#[test]
fn fast_diagonal_drag_selects_visible_crossed_cards_and_ignores_covered_cards() {
    let mut drag = CardDragSelection::default();
    drag.begin_path(0, true);
    let mut regions = Vec::new();
    for column in 0..3 {
        for row in 0..3 {
            let min = Vec2::new(column as f32 * 50.0, row as f32 * 22.0);
            regions.push(CardHitRegion {
                index: column * 3 + row,
                rect: Rect::from_corners(min, min + Vec2::new(40.0, 18.0)),
            });
        }
    }
    let start = Vec2::new(20.0, 9.0);
    let end = Vec2::new(120.0, 53.0);
    regions.reverse();
    for index in crossed_cards(start, end, &regions) {
        drag.visit(index);
    }
    let mut selected = HashSet::new();
    assert!(drag.finish(&mut selected, (0..9).map(|index| (index, index))));
    assert_eq!(selected, HashSet::from([0, 4, 8]));
    regions.push(CardHitRegion {
        index: 9,
        rect: Rect::from_corners(Vec2::ZERO, Vec2::new(150.0, 66.0)),
    });
    assert_eq!(crossed_cards(start, end, &regions), [9]);
    drag.begin(0, false);
    drag.visit(8);
    assert!(drag.contains(3));
}

#[test]
fn double_click_selects_or_clears_a_column_without_touching_other_columns() {
    let column = [
        spades(&[9])[0],
        GuandanCard {
            deck: 1,
            ..spades(&[9])[0]
        },
    ];
    let outside = spades(&[14])[0];
    for original in [
        HashSet::new(),
        HashSet::from([column[0]]),
        column.into_iter().collect(),
    ] {
        let all_selected = column.iter().all(|card| original.contains(card));
        let mut ui = GuandanUiState::default();
        ui.selected.extend(original);
        ui.selected.insert(outside);
        let mut drag = CardDragSelection::default();
        for _ in 0..2 {
            drag.begin_path(0, !ui.selected.contains(&column[0]));
            drag.finish(&mut ui.selected, [(0, column[0])].into_iter());
        }
        ui.toggle_selection(column);
        assert_eq!(
            column.iter().all(|card| ui.selected.contains(card)),
            !all_selected
        );
        assert!(ui.selected.contains(&outside));
    }
}
