use super::support::{Progress, REGISTRY, win};
use crate::MahjongFan;
use leocard_achievements::AchievementTier;
use leocard_game_common::PlayerId;

#[test]
fn repeated_fans_count_winning_hands_instead_of_copies_and_ignore_other_winners() {
    for (id, fan, target, copies) in [
        (
            "leocard:mahjong/fully_concealed_hand",
            MahjongFan::FullyConcealedHand,
            20,
            1,
        ),
        ("leocard:mahjong/tile_hog", MahjongFan::TileHog, 50, 2),
        (
            "leocard:mahjong/pure_double_chow",
            MahjongFan::PureDoubleChow,
            100,
            2,
        ),
    ] {
        let mut progress = Progress::default();
        for _ in 1..target {
            progress.apply(win(&[(fan, copies)], 0, None, true));
        }
        assert_eq!(progress.count(id), target - 1);
        assert!(!progress.achieved(id));
        progress.apply_for(PlayerId(1), win(&[(fan, copies)], 0, None, true));
        assert_eq!(progress.count(id), target - 1);
        progress.apply(win(&[(fan, copies)], 0, None, true));
        assert!(progress.achieved(id));
    }
}

#[test]
fn grouped_fans_share_one_increment_per_hand_and_follow_the_updated_targets() {
    for (id, fans, target) in [
        (
            "leocard:mahjong/big_eight_collector",
            &[MahjongFan::AllPungs, MahjongFan::HalfFlush][..],
            200,
        ),
        (
            "leocard:mahjong/small_eight_collector",
            &[MahjongFan::SevenPairs][..],
            200,
        ),
        (
            "leocard:mahjong/almost_major_collector",
            &[MahjongFan::BigThreeWinds][..],
            50,
        ),
        (
            "leocard:mahjong/last_wall_collector",
            &[MahjongFan::LastTileDraw][..],
            30,
        ),
        (
            "leocard:mahjong/kong_win_collector",
            &[MahjongFan::ThreeKongs, MahjongFan::OutWithReplacementTile][..],
            30,
        ),
        (
            "leocard:mahjong/tight_wait_collector",
            &[MahjongFan::SingleWait][..],
            500,
        ),
        (
            "leocard:mahjong/pure_color_collector",
            &[MahjongFan::FullFlush, MahjongFan::PureShiftedChows][..],
            100,
        ),
    ] {
        let definition = REGISTRY.by_id(id).unwrap();
        assert_eq!(definition.tier, AchievementTier::Silver);
        assert_eq!(definition.criteria[0].target, target);
        let mut progress = Progress::default();
        let values = fans.iter().map(|&fan| (fan, 1)).collect::<Vec<_>>();
        for _ in 1..target {
            progress.apply(win(&values, 0, None, true));
        }
        assert_eq!(progress.count(id), target - 1);
        assert!(!progress.achieved(id));
        progress.apply(win(&values, 0, None, true));
        assert!(progress.achieved(id));
    }

    let mut progress = Progress::default();
    progress.apply(win(&[(MahjongFan::PrevalentWind, 1)], 0, None, true));
    progress.apply(win(&[(MahjongFan::SeatWind, 1)], 0, None, true));
    assert!(!progress.achieved("leocard:mahjong/prevalent_seat_wind_win"));
    progress.apply(win(
        &[(MahjongFan::PrevalentWind, 1), (MahjongFan::SeatWind, 1)],
        0,
        None,
        true,
    ));
    assert!(progress.achieved("leocard:mahjong/prevalent_seat_wind_win"));
    assert_eq!(progress.count("leocard:mahjong/honor_pung_collector"), 3);
    assert_eq!(
        REGISTRY
            .by_id("leocard:mahjong/five_hundred_wins")
            .unwrap()
            .title,
        "抓炮大王"
    );
    assert_eq!(
        REGISTRY
            .by_id("leocard:mahjong/seven_shifted_pairs")
            .unwrap()
            .title,
        "七步成诗"
    );
}
