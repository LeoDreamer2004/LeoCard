use super::support::{Progress, win};
use crate::MahjongFan;
use leocard_game_common::PlayerId;

#[test]
fn actual_self_draws_and_concealed_shapes_count_even_when_the_fans_are_excluded() {
    let mut progress = Progress::default();
    // A high fan suppresses the lower self-draw/concealed-hand fan labels.
    let fans = [(MahjongFan::FourConcealedPungs, 1)];
    for _ in 0..199 {
        progress.apply(win(&fans, 0, None, true));
    }
    assert!(!progress.achieved("leocard:mahjong/two_hundred_self_draws"));
    assert_eq!(progress.count("leocard:mahjong/self_drawn"), 0);
    assert_eq!(progress.count("leocard:mahjong/concealed_hand"), 0);
    progress.apply(win(&fans, 0, None, true));
    assert!(progress.achieved("leocard:mahjong/two_hundred_self_draws"));
    for _ in 0..99 {
        progress.apply(win(&fans, 0, Some(PlayerId(1)), true));
    }
    assert!(!progress.achieved("leocard:mahjong/three_hundred_concealed_wins"));
    progress.apply(win(
        &[(MahjongFan::AllPungs, 1)],
        0,
        Some(PlayerId(1)),
        false,
    ));
    assert_eq!(
        progress.count("leocard:mahjong/three_hundred_concealed_wins"),
        299
    );
    progress.apply(win(&fans, 0, Some(PlayerId(1)), true));
    assert!(progress.achieved("leocard:mahjong/three_hundred_concealed_wins"));
}
