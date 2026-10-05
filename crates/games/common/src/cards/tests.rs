use super::{DeckError, contains_unique_cards, has_unique_cards, validate_deck};

#[test]
fn deck_validation_uses_physical_identity_and_distinguishes_size_from_contents() {
    let expected = [(7, 0), (7, 1), (8, 0)];
    assert_eq!(validate_deck(&[(8, 0), (7, 1), (7, 0)], &expected), Ok(()));
    assert_eq!(
        validate_deck(&[(7, 0), (7, 0), (8, 0)], &expected),
        Err(DeckError::InvalidContents)
    );
    assert_eq!(
        validate_deck(&[(7, 0), (7, 1), (8, 1)], &expected),
        Err(DeckError::InvalidContents)
    );
    assert_eq!(
        validate_deck(&[(7, 0), (7, 0)], &expected),
        Err(DeckError::InvalidSize {
            expected: 3,
            actual: 2
        })
    );
}

#[test]
fn selections_allow_distinct_copies_but_reject_repeated_or_unowned_cards() {
    let hand = [(7, 0), (7, 1), (8, 0)];
    assert!(has_unique_cards(&hand));
    assert!(contains_unique_cards(&hand, &[(7, 1), (7, 0)]));
    assert!(contains_unique_cards(&hand, &[]));
    assert!(!has_unique_cards(&[(7, 0), (7, 0)]));
    assert!(!contains_unique_cards(&hand, &[(7, 0), (7, 0)]));
    assert!(!contains_unique_cards(&hand, &[(8, 1)]));
}
