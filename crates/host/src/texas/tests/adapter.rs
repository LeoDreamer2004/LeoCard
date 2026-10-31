use super::super::*;
use super::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameSnapshot, MatchId, PROTOCOL_VERSION, ProfileId, Revision, RoomId, SeatId, ServerEvent,
    ServerMessage, TexasEvent, TexasPhaseView, TexasViolation, decode_frame, encode_frame,
};
use leocard_texas_holdem::{TexasAction, TexasBlindKind, TexasRuleSet, TexasStreet, build_deck};

const HOST: PlayerId = PlayerId(20);
const LEFT: PlayerId = PlayerId(30);
const RIGHT: PlayerId = PlayerId(10);

fn player(id: PlayerId, seat: u8) -> TablePlayer {
    TablePlayer {
        id,
        profile_id: ProfileId([id.0; 32]),
        name: format!("玩家{}", id.0),
        avatar: None,
        seat: SeatId(seat),
        connected: true,
        reference_points: 0,
        completed_games: 0,
    }
}

fn raw_adapter() -> TexasAdapter {
    TexasAdapter::new(
        MatchId([7; 16]),
        52300,
        HOST,
        vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
        TexasRuleSet::default(),
        HOST,
        build_deck(false),
    )
    .unwrap()
}

fn adapter() -> TexasAdapter {
    let mut game = raw_adapter();
    game.act(LEFT, TexasAction::PostBlind).unwrap();
    game.act(RIGHT, TexasAction::PostBlind).unwrap();
    game
}

#[test]
fn blind_posting_is_exposed_before_normal_preflop_actions() {
    let mut game = raw_adapter();
    let first = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
    assert_eq!(first.player, LEFT);
    assert_eq!(first.kind, TexasBlindKind::Small);
    assert_eq!(first.amount, 1);
    assert!(matches!(
        game.act(LEFT, TexasAction::Call),
        Err(AdapterError::Violation(TexasViolation::MustPostBlind))
    ));
    game.act(LEFT, TexasAction::PostBlind).unwrap();
    let second = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
    assert_eq!(second.player, RIGHT);
    assert_eq!(second.kind, TexasBlindKind::Big);
    game.act(RIGHT, TexasAction::PostBlind).unwrap();
    assert!(game.snapshot(HOST).unwrap().blind_to_post.is_none());
    assert_eq!(game.current_player(), Some(HOST));
}

#[test]
fn session_posts_each_human_blind_automatically_after_a_short_delay() {
    let mut session = session_waiting_for_blinds();
    assert!(session.game().unwrap().game().blind_to_post().is_some());
    assert!(
        session
            .advance_time(std::time::Duration::from_millis(1799))
            .is_empty()
    );
    assert!(session.game().unwrap().game().blind_to_post().is_some());

    session.advance_time(std::time::Duration::from_millis(1));
    assert_eq!(
        session.game().unwrap().game().blind_to_post().unwrap().1,
        TexasBlindKind::Big
    );
    assert!(
        session
            .advance_time(std::time::Duration::from_millis(899))
            .is_empty()
    );
    assert_eq!(
        session.game().unwrap().game().blind_to_post().unwrap().1,
        TexasBlindKind::Big
    );
    session.advance_time(std::time::Duration::from_millis(1));
    assert!(session.game().unwrap().game().blind_to_post().is_none());
}

#[test]
fn seat_order_maps_platform_ids_to_core_positions() {
    let game = adapter();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(
        snapshot
            .players
            .iter()
            .map(|p| p.player.id)
            .collect::<Vec<_>>(),
        vec![HOST, LEFT, RIGHT]
    );
    assert_eq!(snapshot.dealer, HOST);
    assert_eq!(snapshot.small_blind, LEFT);
    assert_eq!(snapshot.big_blind, RIGHT);
    assert_eq!(snapshot.current_player, Some(HOST));
}

#[test]
fn betting_snapshots_only_contain_the_recipient_hole_cards() {
    let game = adapter();
    let host = game.snapshot(HOST).unwrap();
    let left = game.snapshot(LEFT).unwrap();
    assert_eq!(host.your_hole_cards.len(), 2);
    assert_eq!(left.your_hole_cards.len(), 2);
    assert_ne!(host.your_hole_cards, left.your_hole_cards);
    assert!(host.revealed_hands.is_empty());
    assert!(left.revealed_hands.is_empty());
    assert_eq!(host.players, left.players);
}

#[test]
fn omaha_snapshots_deal_and_reveal_four_cards_with_an_evaluated_hand() {
    let mut game = TexasAdapter::new(
        MatchId([8; 16]),
        52300,
        HOST,
        vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
        TexasRuleSet {
            omaha: true,
            ..TexasRuleSet::default()
        },
        HOST,
        build_deck(false),
    )
    .unwrap();
    assert_eq!(game.snapshot(HOST).unwrap().your_hole_cards.len(), 4);

    game.act(LEFT, TexasAction::PostBlind).unwrap();
    game.act(RIGHT, TexasAction::PostBlind).unwrap();
    game.act(HOST, TexasAction::AllIn).unwrap();
    game.act(LEFT, TexasAction::AllIn).unwrap();
    game.act(RIGHT, TexasAction::Call).unwrap();

    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.community.len(), 5);
    assert_eq!(snapshot.revealed_hands.len(), 3);
    assert!(
        snapshot
            .revealed_hands
            .iter()
            .all(|hand| hand.cards.len() == 4 && hand.best.is_some())
    );

    let message = ServerMessage {
        game_context: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(523),
        revision: Revision(9),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::Texas(snapshot)),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
}

#[test]
fn adapter_emits_action_and_street_events() {
    let mut game = adapter();
    game.act(HOST, TexasAction::Call).unwrap();
    game.act(LEFT, TexasAction::Call).unwrap();
    let events = game.act(RIGHT, TexasAction::Check).unwrap();
    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        TexasEvent::ActionApplied {
            player: RIGHT,
            action: TexasAction::Check,
            amount: 0,
            ..
        }
    ));
    assert!(matches!(
        &events[1],
        TexasEvent::StreetAdvanced { street, dealt }
            if *street == TexasStreet::Flop && dealt.len() == 3
    ));
    assert_eq!(game.snapshot(HOST).unwrap().community.len(), 3);
}

#[test]
fn rejected_action_returns_a_wire_violation_without_mutating_state() {
    let mut game = adapter();
    let before = game.game().clone();
    assert_eq!(
        game.act(LEFT, TexasAction::Call),
        Err(AdapterError::Violation(TexasViolation::NotPlayersTurn))
    );
    assert_eq!(game.game(), &before);
}

#[test]
fn showdown_reveals_all_hands_and_exposes_pot_awards() {
    let mut game = adapter();
    game.act(HOST, TexasAction::AllIn).unwrap();
    game.act(LEFT, TexasAction::AllIn).unwrap();
    let events = game.act(RIGHT, TexasAction::Call).unwrap();
    assert!(matches!(
        events.last(),
        Some(TexasEvent::HandFinished { showdown: true })
    ));
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.community.len(), 5);
    assert_eq!(snapshot.revealed_hands.len(), 3);
    let TexasPhaseView::HandComplete {
        showdown, awards, ..
    } = snapshot.phase
    else {
        panic!("the hand should be complete");
    };
    assert!(showdown);
    assert_eq!(awards.iter().map(|award| award.amount).sum::<u32>(), 60);
}

#[test]
fn showdown_keeps_previously_folded_hands_private() {
    let mut game = adapter();
    game.act(HOST, TexasAction::Fold).unwrap();
    game.act(LEFT, TexasAction::AllIn).unwrap();
    game.act(RIGHT, TexasAction::Call).unwrap();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.revealed_hands.len(), 2);
    assert!(
        snapshot
            .revealed_hands
            .iter()
            .all(|hand| hand.player != HOST)
    );
}

#[test]
fn uncontested_win_does_not_reveal_any_hole_cards() {
    let mut game = adapter();
    game.act(HOST, TexasAction::Fold).unwrap();
    let events = game.act(LEFT, TexasAction::Fold).unwrap();
    assert!(matches!(
        events.last(),
        Some(TexasEvent::HandFinished { showdown: false })
    ));
    let snapshot = game.snapshot(RIGHT).unwrap();
    assert!(snapshot.revealed_hands.is_empty());
}

#[test]
fn next_hand_keeps_stacks_and_rotates_the_dealer() {
    let mut game = adapter();
    game.act(HOST, TexasAction::Fold).unwrap();
    game.act(LEFT, TexasAction::Fold).unwrap();
    game.start_next_hand(build_deck(false)).unwrap();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.hand_number, 1);
    assert_eq!(snapshot.dealer, LEFT);
    assert!(matches!(
        snapshot.phase,
        TexasPhaseView::Betting {
            street: TexasStreet::PreFlop
        }
    ));
}

#[test]
fn private_snapshot_round_trips_through_the_wire_frame() {
    let snapshot = adapter().snapshot(HOST).unwrap();
    let message = ServerMessage {
        game_context: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(523),
        revision: Revision(8),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::Texas(snapshot.clone())),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
    let ServerEvent::GameSnapshot(GameSnapshot::Texas(decoded_snapshot)) = decoded.event else {
        panic!("wire frame changed the concrete game variant");
    };
    assert_eq!(decoded_snapshot.your_hole_cards, snapshot.your_hole_cards);
    assert!(decoded_snapshot.revealed_hands.is_empty());
}

#[test]
fn spectator_equity_is_exact_and_waits_for_the_folded_betting_round() {
    use leocard_texas_holdem::{TexasCard, TexasRank, TexasSuit};
    use std::{
        thread,
        time::{Duration, Instant},
    };

    // LEFT holds A♠K♠; the flop is Q♠J♠T♠. No possible turn/river can beat this hand.
    let prefix = [
        TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
        TexasCard::new(TexasSuit::Heart, TexasRank::Two),
        TexasCard::new(TexasSuit::Club, TexasRank::Two),
        TexasCard::new(TexasSuit::Spade, TexasRank::King),
        TexasCard::new(TexasSuit::Heart, TexasRank::Three),
        TexasCard::new(TexasSuit::Club, TexasRank::Three),
        TexasCard::new(TexasSuit::Spade, TexasRank::Queen),
        TexasCard::new(TexasSuit::Spade, TexasRank::Jack),
        TexasCard::new(TexasSuit::Spade, TexasRank::Ten),
    ];
    let mut deck = prefix.to_vec();
    deck.extend(
        build_deck(false)
            .into_iter()
            .filter(|card| !prefix.contains(card)),
    );
    let mut game = TexasAdapter::new(
        MatchId([7; 16]),
        52300,
        HOST,
        vec![player(HOST, 0), player(LEFT, 1), player(RIGHT, 2)],
        TexasRuleSet::default(),
        HOST,
        deck,
    )
    .unwrap();
    game.act(LEFT, TexasAction::PostBlind).unwrap();
    game.act(RIGHT, TexasAction::PostBlind).unwrap();
    game.act(HOST, TexasAction::Fold).unwrap();
    assert!(game.snapshot(HOST).unwrap().spectator_equities.is_none());
    game.act(LEFT, TexasAction::Call).unwrap();
    assert!(game.snapshot(HOST).unwrap().spectator_equities.is_none());
    game.act(RIGHT, TexasAction::Check).unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let observer = game.snapshot(HOST).unwrap();
    assert!(observer.spectator_available);
    assert!(observer.spectator_equities.is_none());
    game.set_spectator_win_rates(HOST, true).unwrap();
    let rates = loop {
        let snapshot = game.snapshot(HOST).unwrap();
        assert!(snapshot.revealed_hands.is_empty());
        assert_eq!(
            snapshot.your_hole_cards,
            game.game().players()[0].hole_cards()
        );
        let rates = snapshot.spectator_equities.unwrap();
        if !rates.is_empty() {
            break rates;
        }
        assert!(
            Instant::now() < deadline,
            "exact equity calculation did not finish"
        );
        thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(
        rates
            .iter()
            .find(|rate| rate.player == LEFT)
            .unwrap()
            .basis_points,
        10_000
    );
    assert!(
        rates
            .iter()
            .filter(|rate| rate.player != LEFT)
            .all(|rate| rate.basis_points == 0)
    );
    for recipient in [LEFT, RIGHT] {
        let snapshot = game.snapshot(recipient).unwrap();
        assert!(snapshot.spectator_equities.is_none());
        assert!(snapshot.revealed_hands.is_empty());
    }
    game.act(LEFT, TexasAction::Fold).unwrap();
    assert!(game.snapshot(HOST).unwrap().spectator_equities.is_none());
}
