use super::prelude::*;
use leocard_protocol::{PlayerId, UnoPendingSwapView};
use leocard_uno::{UnoCard, UnoColor, UnoFace, build_flip_deck};
use std::path::Path;

#[test]
fn swap_pack_target_selection_replaces_one_target_but_requires_deselecting_a_full_pair() {
    let you = PlayerId(0);
    let first = PlayerId(1);
    let second = PlayerId(2);
    let third = PlayerId(3);
    let mut selected = vec![first];

    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::SwapOneTarget { player: you }),
        you,
        second,
        &mut selected,
    );
    assert_eq!(selected, vec![second]);
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::SwapOneTarget { player: you }),
        you,
        second,
        &mut selected,
    );
    assert!(selected.is_empty());

    selected = vec![first, second];
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::ForceTrade { player: you }),
        you,
        third,
        &mut selected,
    );
    assert_eq!(selected, vec![first, second]);
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::ForceTrade { player: you }),
        you,
        first,
        &mut selected,
    );
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::ForceTrade { player: you }),
        you,
        third,
        &mut selected,
    );
    assert_eq!(selected, vec![second, third]);
}

#[test]
fn every_stack_pack_face_resolves_to_a_valid_card_texture() {
    let mut cards = Vec::new();
    for color in UnoColor::ALL {
        cards.push(UnoCard::action(color, UnoFace::StackOne, 0));
        cards.push(UnoCard::action(color, UnoFace::StackTwo, 0));
    }
    cards.push(UnoCard::wild(UnoFace::WildStackThree, 0));
    cards.push(UnoCard::wild(UnoFace::WildStackNumber, 0));

    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for card in cards {
        let path = assets.join(uno_card_asset_path(card));
        let image = image::open(&path)
            .unwrap_or_else(|error| panic!("{} 无法解码：{error}", path.display()));
        assert_eq!(image.width(), 256);
        assert_eq!(image.height(), 400);
    }
}

#[test]
fn every_flip_face_resolves_to_a_valid_card_texture() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for card in build_flip_deck() {
        for face in [Some(card.public_face()), card.opposite_public_face()] {
            let face = face.expect("FLIP cards have two faces");
            let path = assets.join(uno_card_asset_path(face));
            let image = image::open(&path)
                .unwrap_or_else(|error| panic!("{} 无法解码：{error}", path.display()));
            assert_eq!(image.width(), 256);
            assert_eq!(image.height(), 400);
        }
    }
}
