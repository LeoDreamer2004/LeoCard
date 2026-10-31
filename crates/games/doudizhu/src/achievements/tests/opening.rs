use super::support::{accepted, bomb, game_with_kitty};
use crate::achievements::DoudizhuAchievementFacts;
use crate::{DoudizhuAction, DoudizhuCard, DoudizhuMode, DoudizhuPhase, bot_action};
use leocard_game_common::PlayerId;

fn four_player_opening() -> Vec<DoudizhuCard> {
    let rank = |rank: u8| {
        (0..2)
            .flat_map(move |deck| (0..4).map(move |suit| DoudizhuCard(deck * 54 + rank * 4 + suit)))
    };
    rank(0)
        .chain(rank(1))
        .chain(rank(2))
        .chain(rank(3).take(5))
        .chain([
            DoudizhuCard(52),
            DoudizhuCard(53),
            DoudizhuCard(106),
            DoudizhuCard(107),
        ])
        .collect()
}

#[test]
fn all_bomb_openings_use_each_modes_bombs_and_complete_rockets_and_stay_secret_until_settlement() {
    let three: Vec<_> = (0..5).flat_map(bomb).collect();
    let four = four_player_opening();
    let mut single = four.clone();
    single[24] = DoudizhuCard(32); // Four of a kind remains, plus an unmatched 10.
    let mut incomplete_rocket = four.clone();
    incomplete_rocket[32] = DoudizhuCard(67); // Six of a kind remains, plus only three jokers.
    for (mode, full_hand, expected) in [
        (DoudizhuMode::Three, three, true),
        (DoudizhuMode::Four, four, true),
        (DoudizhuMode::Four, single, false),
        (DoudizhuMode::Four, incomplete_rocket, false),
    ] {
        let split = full_hand.len() - mode.kitty_len();
        let mut prefixes = vec![Vec::new(); usize::from(mode.players())];
        prefixes[0] = full_hand[..split].to_vec();
        let (mut game, mut tracker) = game_with_kitty(mode, &prefixes, &full_hand[split..]);
        // The landlord's complete hand includes the kitty before opening-hand analysis.
        assert_eq!(game.hand(PlayerId(0)).unwrap().len(), full_hand.len());
        let events = accepted(
            &mut game,
            &mut tracker,
            PlayerId(0),
            DoudizhuAction::Play(bomb(0)),
        );
        assert!(
            events
                .iter()
                .all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Hand(_)))
        );
        for _ in 0..1500 {
            if game.phase == DoudizhuPhase::Finished {
                break;
            }
            let player = game.current;
            let action = bot_action(&game).unwrap();
            let events = accepted(&mut game, &mut tracker, player, action);
            assert!(
                events
                    .iter()
                    .all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Hand(_)))
            );
        }
        assert_eq!(game.phase, DoudizhuPhase::Finished);
        let after = game.result.as_ref().unwrap().scores.clone();
        let reports =
            tracker.finish_hand(&game, &vec![0; usize::from(mode.players())], &after, false);
        let facts = reports
            .iter()
            .find_map(|(player, facts)| match facts {
                DoudizhuAchievementFacts::Hand(facts) if *player == PlayerId(0) => Some(facts),
                _ => None,
            })
            .unwrap();
        assert_eq!(facts.all_bomb_hand, expected, "{mode:?}");
        tracker.begin_hand();
        let reports =
            tracker.finish_hand(&game, &vec![0; usize::from(mode.players())], &after, false);
        assert!(reports.iter().all(|(_, facts)| !matches!(facts, DoudizhuAchievementFacts::Hand(facts) if facts.all_bomb_hand)));
    }
}
