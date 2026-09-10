use super::*;
use leocard_protocol::{ClientCommand, GameCommand, ServerEvent};
use leocard_protocol::{
    GameEvent, GameViolation, PlayerId, RejectReason, ShengjiCommand, ShengjiDeclarationView,
    ShengjiEvent, ShengjiPhaseView, ShengjiThrowFailureStage, ShengjiThrowFailureView,
    ShengjiViolation,
};
#[cfg(test)]
use leocard_shengji::build_deck;
use leocard_shengji::{ActionOutcome, ShengjiCard, ShengjiPlayerId, ShengjiRuleSet};
use leocard_shengji::{ShengjiRank, ShengjiSuit};

#[test]
fn bid_pass_confirmations_reset_on_a_new_declaration_and_close_early_when_unanimous() {
    let target = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two);
    let mut deck = build_deck();
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(1, index);
    let (mut session, connections) = started_session_with_deck(ShengjiRuleSet::default(), deck);
    session.advance_time(DEAL_INTERVAL * 100);

    let first_confirmation = session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
        ),
    );
    assert!(matches!(
        game_snapshot(&first_confirmation, connections[0]).phase,
        ShengjiPhaseView::BiddingGrace {
            confirmed_count: 1,
            you_confirmed: true,
            ..
        }
    ));
    assert!(matches!(
        game_snapshot(&first_confirmation, connections[1]).phase,
        ShengjiPhaseView::BiddingGrace {
            confirmed_count: 1,
            you_confirmed: false,
            ..
        }
    ));

    let declaration = session.handle(
        connections[1],
        message(
            1,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![target],
            })),
        ),
    );
    assert!(matches!(
        game_snapshot(&declaration, connections[0]).phase,
        ShengjiPhaseView::BiddingGrace {
            confirmed_count: 0,
            you_confirmed: false,
            ..
        }
    ));

    for (index, connection) in connections[..3].iter().copied().enumerate() {
        let deliveries = session.handle(
            connection,
            message(
                index as u8,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
            ),
        );
        assert!(matches!(
            game_snapshot(&deliveries, connections[3]).phase,
            ShengjiPhaseView::BiddingGrace { .. }
        ));
    }
    let locked = session.handle(
        connections[3],
        message(
            3,
            6,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
        ),
    );
    assert!(matches!(
        game_snapshot(&locked, connections[3]).phase,
        ShengjiPhaseView::Burying
    ));
    assert!(locked.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::BiddingLocked {
            declaration: Some(_)
        }))
    )));
}

#[test]
fn declaration_grace_closes_into_private_kitty_and_burying() {
    let (mut session, connections, target) = started_session();
    session.advance_time(DEAL_INTERVAL);
    let declaration = session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![target],
            })),
        ),
    );
    assert!(matches!(
        game_snapshot(&declaration, connections[0]).declaration,
        Some(ShengjiDeclarationView {
            player: PlayerId(0),
            ..
        })
    ));
    assert_eq!(
        game_snapshot(&declaration, connections[0]).dealer,
        Some(PlayerId(0))
    );
    assert_eq!(
        game_snapshot(&declaration, connections[0]).your_exposed_cards,
        vec![target]
    );
    assert!(
        game_snapshot(&declaration, connections[1])
            .your_exposed_cards
            .is_empty()
    );

    let deliveries = session.advance_time(DEAL_INTERVAL * 99);
    assert!(matches!(
        game_snapshot(&deliveries, connections[0]).phase,
        ShengjiPhaseView::BiddingGrace { .. }
    ));
    let deliveries = session.advance_time(BIDDING_GRACE);
    let dealer = game_snapshot(&deliveries, connections[0]);
    let guest = game_snapshot(&deliveries, connections[1]);
    assert_eq!(dealer.your_hand.len(), 33);
    assert_eq!(guest.your_hand.len(), 25);
    assert_eq!(dealer.buried_count, 0);
    assert!(matches!(dealer.phase, ShengjiPhaseView::Burying));
}

#[test]
fn first_hand_dealer_badge_follows_initial_bid_and_counter_immediately() {
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let spades = [
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Two),
    ];
    let mut deck = build_deck();
    for (target_index, card) in [(0, diamond), (1, spades[0]), (5, spades[1])] {
        let source_index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, source_index);
    }
    let (mut session, connections) = started_session_with_deck(ShengjiRuleSet::default(), deck);
    session.advance_time(DEAL_INTERVAL * 6);

    let initial = session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![diamond],
            })),
        ),
    );
    assert_eq!(
        game_snapshot(&initial, connections[0]).dealer,
        Some(PlayerId(0))
    );

    let counter = session.handle(
        connections[1],
        message(
            1,
            6,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: spades.to_vec(),
            })),
        ),
    );
    assert_eq!(
        game_snapshot(&counter, connections[0]).dealer,
        Some(PlayerId(1))
    );
}

#[test]
fn later_hand_snapshot_exposes_the_fixed_dealer_while_dealing() {
    let (mut session, _, _) = started_session();
    session.next_dealer = Some(ShengjiPlayerId(2));
    session.start_hand(true).unwrap();

    let snapshot = session.game_snapshot(PlayerId(0));
    assert!(matches!(snapshot.phase, ShengjiPhaseView::Dealing { .. }));
    assert_eq!(snapshot.dealer, Some(PlayerId(2)));
}

#[test]
fn failed_throw_is_shown_then_returned_before_the_forced_play_is_revealed() {
    let (mut session, connections, target) = started_session();
    session.advance_time(DEAL_INTERVAL);
    session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![target],
            })),
        ),
    );
    session.advance_time(DEAL_INTERVAL * 99);
    session.advance_time(BIDDING_GRACE);
    let buried = session.game.as_ref().unwrap().players()[0].hand[..8].to_vec();
    session.handle(
        connections[0],
        message(
            0,
            6,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury { cards: buried })),
        ),
    );

    let hand = &session.game.as_ref().unwrap().players()[0].hand;
    let forced_card = hand[0];
    let returned_card = hand[1];
    session
        .game
        .as_mut()
        .unwrap()
        .play_cards(ShengjiPlayerId(0), &[forced_card])
        .unwrap();
    let forced = session
        .game
        .as_ref()
        .unwrap()
        .current_trick()
        .unwrap()
        .plays[0]
        .1
        .clone();
    session.after_game_outcome(&ActionOutcome::ThrowFailed {
        player: ShengjiPlayerId(0),
        attempted: vec![forced_card, returned_card],
        forced: forced.clone(),
        penalty_points: 10,
        next: ShengjiPlayerId(1),
    });

    let showing = session.game_snapshot(PlayerId(0));
    assert_eq!(showing.current_player, None);
    assert_eq!(
        showing.throw_failure,
        Some(ShengjiThrowFailureView {
            player: PlayerId(0),
            attempted: vec![forced_card, returned_card],
            forced: forced.clone(),
            penalty_points: 10,
            stage: ShengjiThrowFailureStage::Showing,
        })
    );
    let blocked = session.handle(
        connections[1],
        message(
            1,
            7,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::PlayCards {
                cards: vec![session.game.as_ref().unwrap().players()[1].hand[0]],
            })),
        ),
    );
    assert!(blocked.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::Shengji(ShengjiViolation::WrongPhase))
        }
    )));

    assert!(
        session
            .advance_time(THROW_FAILURE_SHOW_DURATION - Duration::from_millis(1))
            .is_empty()
    );
    let returning_deliveries = session.advance_time(Duration::from_millis(1));
    let returning = game_snapshot(&returning_deliveries, connections[0]);
    assert_eq!(
        returning.throw_failure.unwrap().stage,
        ShengjiThrowFailureStage::Returning
    );
    assert_eq!(returning.current_player, None);

    assert!(
        session
            .advance_time(THROW_FAILURE_RETURN_DURATION - Duration::from_millis(1))
            .is_empty()
    );
    let revealed_deliveries = session.advance_time(Duration::from_millis(1));
    assert!(revealed_deliveries.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsPlayed {
            play,
            is_lead: true,
        })) if play.player == PlayerId(0)
            && play.play == forced
            && play.throw_penalty == 10
    )));
    let revealed = game_snapshot(&revealed_deliveries, connections[0]);
    assert_eq!(revealed.throw_failure, None);
    assert_eq!(revealed.current_player, Some(PlayerId(1)));
}
