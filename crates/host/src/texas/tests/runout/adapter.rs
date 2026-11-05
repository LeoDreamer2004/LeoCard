use super::fixtures::*;
use crate::texas::{AdapterError, TexasAdapter};
use leocard_game_common::PlayerId;
use leocard_protocol::{MatchId, TexasEvent, TexasPhaseView, TexasViolation};
use leocard_texas_holdem::{
    TexasAction, TexasCard, TexasPhase, TexasRank, TexasRuleSet, TexasSuit, build_deck,
};

#[test]
fn refunds_follow_actual_payment_and_preserve_protocol_ids() {
    let mut game = adapter();
    unequal_stacks(&mut game);
    game.act(LEFT, TexasAction::AllIn).unwrap();
    game.act(RIGHT, TexasAction::AllIn).unwrap();
    let events = game.act(HOST, TexasAction::Call).unwrap();
    assert!(matches!(
        events[0],
        TexasEvent::ActionApplied {
            player: HOST,
            amount: 17,
            ..
        }
    ));
    assert_eq!(
        events[1],
        TexasEvent::ChipsReturned {
            player: HOST,
            amount: 5
        }
    );
    assert_eq!(
        events[2],
        TexasEvent::ChipsReturned {
            player: LEFT,
            amount: 5
        }
    );
    assert_eq!(
        events[3],
        TexasEvent::RunoutVoteStarted {
            current_player: RIGHT
        }
    );
    assert_eq!(events.len(), 4);
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.pot, 42);
    assert!(snapshot.players.iter().all(|p| p.committed_total == 14));
    assert_eq!(
        snapshot.players.iter().map(|p| p.stack).collect::<Vec<_>>(),
        vec![13, 5, 0]
    );
    assert!(snapshot.players[1].all_in, "refunded all-in still votes");
    assert_eq!(snapshot.current_player, Some(RIGHT));
}

#[test]
fn vote_progression_is_public_but_cards_wait_for_showdown() {
    let mut game = adapter();
    let events = start_voting(&mut game);
    assert_eq!(
        events.last(),
        Some(&TexasEvent::RunoutVoteStarted {
            current_player: LEFT
        })
    );
    let spectator = PlayerId(99);
    let snapshot = game.spectator_snapshot(spectator);
    assert_eq!(snapshot.phase, TexasPhaseView::ChoosingRunout);
    assert!(snapshot.your_hole_cards.is_empty());
    assert!(snapshot.revealed_hands.is_empty());
    assert!(snapshot.second_community.is_empty());
    let history_len = game.game().betting_history().len();
    for (player, next) in [(LEFT, Some(RIGHT)), (RIGHT, Some(HOST)), (HOST, None)] {
        let events = game.choose_runout(player, true).unwrap();
        assert_eq!(
            events[0],
            TexasEvent::RunoutChosen {
                player,
                twice: true,
                next_player: next
            }
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, TexasEvent::RunoutChosen { .. }))
                .count(),
            1
        );
        if next.is_some() {
            assert_eq!(events.len(), 1);
            let view = game.spectator_snapshot(spectator);
            assert!(view.revealed_hands.is_empty());
            assert!(view.second_community.is_empty());
            assert_eq!(view.runout_votes.last().unwrap().player, player);
        } else {
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, TexasEvent::HandAnalyzed { .. }))
                    .count(),
                3
            );
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, TexasEvent::HandFinished { .. }))
                    .count(),
                1
            );
        }
    }
    assert_eq!(game.game().betting_history().len(), history_len);
    let view = game.spectator_snapshot(spectator);
    assert!(view.your_hole_cards.is_empty());
    assert_eq!(view.community.len(), 5);
    assert_eq!(view.second_community.len(), 5);
    assert!(
        view.revealed_hands
            .iter()
            .all(|h| h.best.is_some() && h.second_best.is_some())
    );
    assert_eq!(
        view.runout_votes
            .iter()
            .map(|v| v.player)
            .collect::<Vec<_>>(),
        vec![LEFT, RIGHT, HOST]
    );
    let TexasPhaseView::HandComplete { awards, .. } = view.phase else {
        panic!("showdown")
    };
    assert_eq!(awards.iter().map(|award| award.amount).sum::<u32>(), 60);
    assert!(awards.iter().any(|a| a.run == 1));
    assert!(awards.iter().any(|a| a.run == 2));
    assert!(
        awards
            .iter()
            .flat_map(|a| &a.winners)
            .all(|id| [HOST, LEFT, RIGHT].contains(id))
    );
}

#[test]
fn final_vote_emits_squid_award_and_analysis_exactly_once() {
    let players = adapter().players().to_vec();
    let prefix = [
        (TexasSuit::Spade, TexasRank::King),
        (TexasSuit::Heart, TexasRank::Queen),
        (TexasSuit::Club, TexasRank::Ace),
        (TexasSuit::Diamond, TexasRank::King),
        (TexasSuit::Club, TexasRank::Queen),
        (TexasSuit::Diamond, TexasRank::Ace),
        (TexasSuit::Club, TexasRank::Two),
        (TexasSuit::Diamond, TexasRank::Three),
        (TexasSuit::Heart, TexasRank::Seven),
        (TexasSuit::Spade, TexasRank::Eight),
        (TexasSuit::Club, TexasRank::Nine),
        (TexasSuit::Diamond, TexasRank::Four),
        (TexasSuit::Club, TexasRank::Five),
        (TexasSuit::Spade, TexasRank::Seven),
        (TexasSuit::Heart, TexasRank::Nine),
        (TexasSuit::Diamond, TexasRank::Jack),
    ]
    .map(|(suit, rank)| TexasCard::new(suit, rank));
    let deck = prefix
        .into_iter()
        .chain(
            build_deck(false)
                .into_iter()
                .filter(|card| !prefix.contains(card)),
        )
        .collect();
    let mut game = TexasAdapter::new(
        MatchId([9; 16]),
        52300,
        HOST,
        players,
        TexasRuleSet {
            squid_markers: 1,
            ..rules()
        },
        HOST,
        deck,
    )
    .unwrap();
    post_blinds(&mut game);
    let started = start_voting(&mut game);
    assert!(!started.iter().any(|event| matches!(
        event,
        TexasEvent::SquidAwarded { .. } | TexasEvent::HandAnalyzed { .. }
    )));
    game.choose_runout(LEFT, true).unwrap();
    game.choose_runout(RIGHT, true).unwrap();
    let events = game.choose_runout(HOST, true).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, TexasEvent::SquidAwarded { player: HOST, .. }))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, TexasEvent::HandAnalyzed { .. }))
            .count(),
        3
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, TexasEvent::HandFinished { .. }))
            .count(),
        1
    );
    let settled = game.game().clone();
    assert!(game.choose_runout(HOST, true).is_err());
    assert_eq!(game.game(), &settled);
}

#[test]
fn invalid_and_duplicate_votes_are_atomic_and_once_does_not_skip_voters() {
    let mut game = adapter();
    let before = game.game().clone();
    assert!(matches!(
        game.choose_runout(LEFT, true),
        Err(AdapterError::Violation(TexasViolation::NotChoosingRunout))
    ));
    assert_eq!(game.game(), &before);
    start_voting(&mut game);
    for (actor, violation) in [
        (HOST, TexasViolation::NotPlayersTurn),
        (RIGHT, TexasViolation::NotPlayersTurn),
    ] {
        let before = game.game().clone();
        assert!(
            matches!(game.choose_runout(actor, true), Err(AdapterError::Violation(v)) if v == violation)
        );
        assert_eq!(game.game(), &before);
    }
    let before = game.game().clone();
    assert!(game.choose_runout(PlayerId(99), true).is_err());
    assert!(matches!(
        game.act(LEFT, TexasAction::Fold),
        Err(AdapterError::Violation(TexasViolation::MustChooseRunout))
    ));
    assert_eq!(game.game(), &before);
    game.choose_runout(LEFT, false).unwrap();
    let before = game.game().clone();
    assert!(matches!(
        game.choose_runout(LEFT, true),
        Err(AdapterError::Violation(TexasViolation::RunoutAlreadyChosen))
    ));
    assert_eq!(game.game(), &before);
    game.choose_runout(RIGHT, true).unwrap();
    assert!(matches!(game.game().phase(), TexasPhase::ChoosingRunout));
    game.choose_runout(HOST, true).unwrap();
    let view = game.snapshot(HOST).unwrap();
    assert_eq!(view.runout_votes.len(), 3);
    assert!(view.second_community.is_empty());
    assert!(view.revealed_hands.iter().all(|h| h.second_best.is_none()));
}
