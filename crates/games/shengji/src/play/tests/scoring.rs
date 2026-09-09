use super::*;

#[test]
fn kitty_multiplier_uses_the_strongest_throw_component() {
    let cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Ace).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::King).as_slice(),
        &[card(0, ShengjiSuit::Spade, ShengjiRank::Queen)],
    ]
    .concat();
    let play = classify_cards(&cards, trump()).unwrap();
    assert_eq!(play.kitty_multiplier(), 8);

    for (pair_count, multiplier) in [(2, 8), (3, 16), (4, 32)] {
        let ranks = [
            ShengjiRank::Ace,
            ShengjiRank::King,
            ShengjiRank::Queen,
            ShengjiRank::Jack,
        ];
        let tractor = ranks[..pair_count]
            .iter()
            .flat_map(|rank| pair(ShengjiSuit::Spade, *rank))
            .collect::<Vec<_>>();
        assert_eq!(
            classify_cards(&tractor, trump())
                .unwrap()
                .kitty_multiplier(),
            multiplier
        );
    }
}
