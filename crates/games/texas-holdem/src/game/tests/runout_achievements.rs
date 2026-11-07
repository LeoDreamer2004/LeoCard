use super::{c, ordered_deck, post_blinds};
use crate::TexasRank::{
    Ace, Eight, Five, Four, Jack, King, Nine, Queen, Seven, Six, Ten, Three, Two,
};
use crate::TexasSuit::{Club, Diamond, Heart, Spade};
use crate::achievements::{DEFINITIONS, TexasAchievementEvent};
use crate::{TexasAction, TexasCard, TexasGameState, TexasRuleSet};
use leocard_achievements::AchievementTrigger;
use leocard_game_common::PlayerId;

const OWN: PlayerId = PlayerId(0);
const OPPONENT: PlayerId = PlayerId(1);
const THIRD: PlayerId = PlayerId(2);

fn winning_board() -> [TexasCard; 5] {
    [
        c(Two, Club),
        c(Three, Diamond),
        c(Seven, Spade),
        c(Eight, Club),
        c(Nine, Diamond),
    ]
}

fn start(
    first: [TexasCard; 5],
    second: Option<[TexasCard; 5]>,
    own_short_stack: bool,
) -> TexasGameState {
    let mut prefix = vec![
        c(King, Spade),
        c(Five, Spade),
        c(Ace, Spade),
        c(King, Heart),
        c(Five, Heart),
        c(Ace, Heart),
    ];
    prefix.extend(first);
    prefix.extend(second.into_iter().flatten());
    let mut game = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 3,
            side_pots: second.is_none(),
            run_it_twice: second.is_some(),
            ..Default::default()
        },
        OWN,
        if own_short_stack {
            vec![20, 50, 20]
        } else {
            vec![50, 20, 20]
        },
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut game);
    game
}

fn assert_achievements(game: &TexasGameState, expected: (bool, bool)) {
    let trigger = AchievementTrigger::Game {
        player: OWN,
        event: TexasAchievementEvent::HandAnalyzed {
            player: OWN,
            statistics: Box::new(game.hand_statistics(OWN).unwrap()),
        }
        .into(),
    };
    for (id, expected) in [
        ("leocard:texas/all_in_runout_win", expected.0),
        ("leocard:texas/runout_twice_comeback", expected.1),
    ] {
        let definition = DEFINITIONS
            .iter()
            .find(|definition| definition.id == id)
            .unwrap();
        assert_eq!(
            (definition.criteria[0].amount)(&trigger),
            u64::from(expected),
            "{id}"
        );
    }
}

#[test]
fn runout_achievements_distinguish_scoops_comebacks_and_ties() {
    let win = winning_board();
    let another_win = [
        c(Two, Diamond),
        c(Four, Club),
        c(Six, Spade),
        c(Ten, Diamond),
        c(Jack, Diamond),
    ];
    let mut loss = win;
    loss[0] = c(King, Diamond);
    let tie = [
        c(Ten, Club),
        c(Jack, Club),
        c(Queen, Club),
        c(King, Club),
        c(Ace, Club),
    ];
    for (first, second, own_short_stack, expected) in [
        (win, None, true, (true, false)),
        (win, None, false, (false, false)),
        (win, Some(another_win), true, (true, false)),
        (loss, Some(another_win), false, (false, true)),
        (another_win, Some(loss), true, (false, false)),
        (tie, Some(another_win), true, (false, false)),
        (loss, Some(tie), true, (false, false)),
    ] {
        let mut game = start(first, second, own_short_stack);
        let (open, response) = if own_short_stack {
            (TexasAction::AllIn, TexasAction::Call)
        } else {
            (TexasAction::Call, TexasAction::AllIn)
        };
        game.act(OWN, open).unwrap();
        game.act(OPPONENT, response).unwrap();
        game.act(THIRD, TexasAction::Fold).unwrap();
        if !own_short_stack {
            game.act(OWN, TexasAction::Call).unwrap();
        }
        while let Some(player) = game.current_player() {
            game.choose_runout(player, true).unwrap();
        }
        assert_achievements(&game, expected);
    }
}

#[test]
fn all_in_without_a_runout_does_not_unlock_the_win() {
    let mut uncontested = start(winning_board(), None, true);
    uncontested.act(OWN, TexasAction::AllIn).unwrap();
    uncontested.act(OPPONENT, TexasAction::Fold).unwrap();
    uncontested.act(THIRD, TexasAction::Fold).unwrap();
    assert_achievements(&uncontested, (false, false));

    let mut river = start(winning_board(), None, true);
    river.act(OWN, TexasAction::Call).unwrap();
    river.act(OPPONENT, TexasAction::Call).unwrap();
    river.act(THIRD, TexasAction::Fold).unwrap();
    while river.community().len() < 5 {
        river
            .act(river.current_player().unwrap(), TexasAction::Check)
            .unwrap();
    }
    river.act(OPPONENT, TexasAction::Check).unwrap();
    river.act(OWN, TexasAction::AllIn).unwrap();
    river.act(OPPONENT, TexasAction::Call).unwrap();
    assert_achievements(&river, (false, false));
}
