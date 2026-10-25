use super::super::MahjongAchievementEvent;
use super::support::{Progress, win};
use crate::Fan;
use leocard_achievements::ParticipantId;

#[test]
fn drawn_flowers_and_winning_flowers_have_independent_totals_and_owners() {
    let mut progress = Progress::default();
    let drawn = MahjongAchievementEvent::FlowerReplaced {
        player: ParticipantId(0),
    };
    for _ in 0..1999 {
        progress.apply(drawn.clone());
    }
    progress.apply(MahjongAchievementEvent::FlowerReplaced {
        player: ParticipantId(1),
    });
    assert_eq!(
        progress.count("leocard:mahjong/two_thousand_drawn_flowers"),
        1999
    );
    assert!(!progress.achieved("leocard:mahjong/two_thousand_drawn_flowers"));
    assert_eq!(
        progress.count("leocard:mahjong/five_hundred_winning_flowers"),
        0
    );
    assert!(!progress.achieved("leocard:mahjong/three_flowers_win"));
    progress.apply(drawn);
    assert!(progress.achieved("leocard:mahjong/two_thousand_drawn_flowers"));

    progress.apply(win(&[(Fan::AllPungs, 1)], 2, None, true));
    assert!(!progress.achieved("leocard:mahjong/three_flowers_win"));
    progress.apply(win(&[(Fan::AllPungs, 1)], 3, None, true));
    assert!(progress.achieved("leocard:mahjong/three_flowers_win"));
    progress.apply_for(ParticipantId(1), win(&[(Fan::AllPungs, 1)], 8, None, true));
    assert!(!progress.achieved("leocard:mahjong/eight_flowers_win"));
    for _ in 0..61 {
        progress.apply(win(&[(Fan::AllPungs, 1)], 8, None, true));
    }
    assert!(progress.achieved("leocard:mahjong/eight_flowers_win"));
    assert_eq!(
        progress.count("leocard:mahjong/five_hundred_winning_flowers"),
        493
    );
    assert!(!progress.achieved("leocard:mahjong/five_hundred_winning_flowers"));
    progress.apply(win(&[(Fan::AllPungs, 1)], 7, None, true));
    assert!(progress.achieved("leocard:mahjong/five_hundred_winning_flowers"));
    assert_eq!(
        progress.count("leocard:mahjong/two_thousand_drawn_flowers"),
        2000
    );
}
