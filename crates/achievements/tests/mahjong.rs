mod common;

use common::{achieved, context, finish};
use leocard_achievements::{AchievementBook, AchievementTrigger, achievement_by_id};
use leocard_mahjong::Fan;
use leocard_protocol::{GameEvent, MahjongEvent};

#[test]
fn low_fan_occurrences_exclude_flowers_and_replays_and_survive_reload() {
    let mut book = AchievementBook::default();
    let event = finish(
        &[
            (Fan::FlowerTiles, 8),
            (Fan::PureDoubleChow, 2),
            (Fan::AllChows, 1),
        ],
        4,
        0,
        false,
    );
    for sequence in 1..=66 {
        book.trigger(&event, context(1, 0, sequence), 100);
    }
    let definition = achievement_by_id("leocard:mahjong/small_fan_collector").unwrap();
    assert_eq!(book.criterion_count(definition, "progress"), 198);
    book.trigger(&event, context(1, 0, 66), 200);
    assert!(!book.achieved(definition));
    let mut book: AchievementBook =
        postcard::from_bytes(&postcard::to_allocvec(&book).unwrap()).unwrap();
    book.trigger(&event, context(1, 0, 67), 201);
    assert_eq!(book.earned_at(definition), Some(201));
    assert!(achieved(&book, "first_ten_wins"));
    assert!(!achieved(&book, "five_hundred_wins"));
}

#[test]
fn high_win_pair_resets_between_matches_but_spans_hands_of_one_match() {
    let mut book = AchievementBook::default();
    let event = finish(&[(Fan::BigFourWinds, 1)], 88, 0, false);
    book.trigger(&event, context(1, 0, 1), 1);
    book.trigger(&event, context(2, 0, 1), 2);
    assert!(!achieved(&book, "two_major_wins"));
    book.trigger(&event, context(2, 1, 2), 3);
    assert!(achieved(&book, "two_major_wins"));
    assert!(!achieved(&book, "two_major_fans"));
    book.trigger(
        &finish(
            &[(Fan::BigFourWinds, 1), (Fan::BigThreeDragons, 1)],
            176,
            2,
            false,
        ),
        context(2, 2, 3),
        4,
    );
    assert!(achieved(&book, "two_major_fans"));
}

#[test]
fn one_point_achievement_requires_eight_distinct_nonflower_fans_and_exactly_eight_points() {
    let fans = [
        Fan::PureDoubleChow,
        Fan::MixedDoubleChow,
        Fan::ShortStraight,
        Fan::TwoTerminalChows,
        Fan::PungOfTerminalsOrHonors,
        Fan::MeldedKong,
        Fan::OneVoidedSuit,
        Fan::NoHonors,
    ];
    let values = fans.map(|fan| (fan, 1));
    let mut book = AchievementBook::default();
    book.trigger(&finish(&values, 9, 0, false), context(1, 0, 1), 1);
    assert!(!achieved(&book, "eight_one_point_fans"));
    book.trigger(&finish(&values, 8, 1, false), context(1, 1, 2), 2);
    assert!(achieved(&book, "eight_one_point_fans"));
    assert!(achieved(&book, "six_fan_eight_points"));
    let mut book = AchievementBook::default();
    let mut with_flower = values;
    with_flower[7] = (Fan::FlowerTiles, 1);
    book.trigger(&finish(&with_flower, 8, 0, false), context(2, 0, 1), 1);
    assert!(!achieved(&book, "eight_one_point_fans"));
}

#[test]
fn collection_requires_every_64_or_88_point_fan_and_no_48_point_fan() {
    let definition = achievement_by_id("leocard:mahjong/all_major_fans").unwrap();
    assert_eq!(definition.criteria.len(), 13);
    let fans = [
        Fan::BigFourWinds,
        Fan::BigThreeDragons,
        Fan::AllGreen,
        Fan::NineGates,
        Fan::FourKongs,
        Fan::SevenShiftedPairs,
        Fan::ThirteenOrphans,
        Fan::AllTerminals,
        Fan::LittleFourWinds,
        Fan::LittleThreeDragons,
        Fan::AllHonors,
        Fan::FourConcealedPungs,
        Fan::PureTerminalChows,
    ];
    let mut book = AchievementBook::default();
    for (index, fan) in fans.into_iter().enumerate() {
        book.trigger(
            &finish(&[(fan, 1)], fan.points(), 0, true),
            context(index as u8, 0, 1),
            index as u64,
        );
        assert_eq!(book.achieved(definition), index == 12);
    }
}

#[test]
fn completed_hand_and_group_counters_count_draws_and_fan_occurrences_separately() {
    let mut book = AchievementBook::default();
    let mut draw = finish(&[], 0, 0, false);
    if let AchievementTrigger::Game {
        event: GameEvent::Mahjong(MahjongEvent::HandFinished { result }),
        ..
    } = &mut draw
    {
        result.winners.clear();
        result.exhaustive_draw = true;
    }
    book.trigger(&draw, context(1, 0, 1), 1);
    let hands = achievement_by_id("leocard:mahjong/ten_thousand_hands").unwrap();
    let wins = achievement_by_id("leocard:mahjong/first_ten_wins").unwrap();
    assert_eq!(book.criterion_count(hands, "progress"), 1);
    assert_eq!(book.criterion_count(wins, "progress"), 0);
    for sequence in 2..=26 {
        book.trigger(
            &finish(
                &[(Fan::MixedShiftedChows, 1), (Fan::MixedTripleChow, 1)],
                14,
                0,
                false,
            ),
            context(1, 0, sequence),
            2,
        );
    }
    assert!(!achieved(&book, "big_eight_collector"));
    for sequence in 27..=51 {
        book.trigger(
            &finish(&[(Fan::MixedTripleChow, 1)], 8, 0, false),
            context(1, 0, sequence),
            3,
        );
    }
    assert!(achieved(&book, "big_eight_collector"));
    assert!(!achieved(&book, "small_eight_collector"));
}
