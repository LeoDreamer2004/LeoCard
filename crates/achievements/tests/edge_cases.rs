mod common;

use common::{achieved, context, finish};
use leocard_achievements::{ACHIEVEMENT_REGISTRY, AchievementBook, AchievementTrigger};
use leocard_mahjong::{Fan, MahjongMatchLength, MahjongMatchProgress};
use leocard_protocol::{GameEvent, MahjongEvent, PlayerId};

#[test]
fn final_match_conditions_use_match_length_and_complete_game_facts() {
    let mut event = finish(&[(Fan::FullFlush, 1)], 24, 3, true);
    let AchievementTrigger::Game {
        event: GameEvent::Mahjong(MahjongEvent::HandFinished { result }),
        ..
    } = &mut event
    else {
        unreachable!();
    };
    result.winners[0].wait_kind_count = 5;
    result.match_scores = [300, 200, -100, -400];
    result.deltas = [600, 0, 0, -600];
    let mut book = AchievementBook::default();
    book.trigger(&event, context(1, 3, 1), 1);
    assert!(!achieved(&book, "east_round_300"));
    assert!(!achieved(&book, "last_hand_comeback"));
    assert!(achieved(&book, "broad_full_flush"));
    let AchievementTrigger::Game {
        event: GameEvent::Mahjong(MahjongEvent::HandFinished { result }),
        ..
    } = &mut event
    else {
        unreachable!();
    };
    result.match_length = MahjongMatchLength::EastRound;
    result.match_progress = MahjongMatchProgress {
        completed_hands: 4,
        exhaustive_draws: 3,
    };
    book.trigger(&event, context(2, 3, 1), 2);
    assert!(achieved(&book, "east_round_300"));
    assert!(achieved(&book, "last_hand_comeback"));
    assert!(!achieved(&book, "all_draw_match"));
    let AchievementTrigger::Game {
        event: GameEvent::Mahjong(MahjongEvent::HandFinished { result }),
        ..
    } = &mut event
    else {
        unreachable!();
    };
    result.match_progress.exhaustive_draws = 4;
    result.winners.clear();
    result.exhaustive_draw = true;
    book.trigger(&event, context(3, 3, 1), 3);
    assert!(achieved(&book, "all_draw_match"));
}

#[test]
fn insufficient_fan_count_is_private_actor_specific_and_resets_per_hand() {
    let mut book = AchievementBook::default();
    let event = |player| AchievementTrigger::Game {
        player: PlayerId(0),
        event: GameEvent::Mahjong(MahjongEvent::WinUnavailable {
            player,
            points_without_flowers: 7,
        }),
    };
    for sequence in 1..=6 {
        book.trigger(&event(PlayerId(1)), context(1, 0, sequence), 1);
    }
    assert!(!achieved(&book, "six_insufficient_fans"));
    for sequence in 7..=11 {
        book.trigger(&event(PlayerId(0)), context(1, 0, sequence), 1);
    }
    book.trigger(&event(PlayerId(0)), context(1, 1, 12), 1);
    assert!(!achieved(&book, "six_insufficient_fans"));
    for sequence in 13..=17 {
        book.trigger(&event(PlayerId(0)), context(1, 1, sequence), 1);
    }
    assert!(achieved(&book, "six_insufficient_fans"));
}

#[test]
fn each_named_bronze_fan_uses_the_named_rule_fan() {
    let examples = [
        ("初窥门径", Fan::MixedShiftedChows),
        ("读起来就是朗朗上口", Fan::MixedTripleChow),
        ("龙飞凤舞", Fan::MixedStraight),
        ("一气呵成", Fan::PureStraight),
        ("从左往右打就可以了", Fan::HalfFlush),
        ("没有那么难呢", Fan::PureShiftedChows),
        ("五谷丰登", Fan::AllTypes),
        ("有碰才有杠", Fan::AllPungs),
        ("有碰也不碰", Fan::SevenPairs),
        ("拖衣带水", Fan::OutsideHand),
        ("奇怪的组合", Fan::KnittedStraight),
        ("破烂手牌的出路", Fan::LesserHonorsAndKnittedTiles),
        ("我喜欢大的", Fan::UpperFour),
        ("我喜欢小的", Fan::LowerFour),
        ("无中生有", Fan::ChickenHand),
        ("不能没有你", Fan::MeldedHand),
        ("那个杠不成立！", Fan::RobbingTheKong),
        ("高岭之花", Fan::OutWithReplacementTile),
        ("笑到最后", Fan::LastTileClaim),
    ];
    for (title, fan) in examples {
        let definition = ACHIEVEMENT_REGISTRY
            .iter()
            .find(|definition| definition.title == title)
            .unwrap();
        assert!(definition.description.contains(fan.name()));
        let mut book = AchievementBook::default();
        book.trigger(
            &finish(&[(fan, 1)], fan.points(), 0, true),
            context(1, 0, 1),
            1,
        );
        assert!(book.achieved(definition), "{title}");
    }
}
