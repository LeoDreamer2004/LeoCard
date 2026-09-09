use super::*;

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

fn raw_adapter() -> TexasHoldemAdapter {
    TexasHoldemAdapter::new(
        MatchId([7; 16]),
        52300,
        HOST,
        vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
        TexasHoldemRuleSet::default(),
        HOST,
        build_deck(false),
    )
    .unwrap()
}

fn adapter() -> TexasHoldemAdapter {
    let mut game = raw_adapter();
    game.act(LEFT, TexasHoldemAction::PostBlind).unwrap();
    game.act(RIGHT, TexasHoldemAction::PostBlind).unwrap();
    game
}

#[test]
fn blind_posting_is_exposed_before_normal_preflop_actions() {
    let mut game = raw_adapter();
    let first = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
    assert_eq!(first.player, LEFT);
    assert_eq!(first.kind, TexasHoldemBlindKind::Small);
    assert_eq!(first.amount, 1);
    assert!(matches!(
        game.act(LEFT, TexasHoldemAction::Call),
        Err(AdapterError::Violation(TexasHoldemViolation::MustPostBlind))
    ));
    game.act(LEFT, TexasHoldemAction::PostBlind).unwrap();
    let second = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
    assert_eq!(second.player, RIGHT);
    assert_eq!(second.kind, TexasHoldemBlindKind::Big);
    game.act(RIGHT, TexasHoldemAction::PostBlind).unwrap();
    assert!(game.snapshot(HOST).unwrap().blind_to_post.is_none());
    assert_eq!(game.current_player(), Some(HOST));
}

#[test]
fn seat_order_maps_platform_ids_to_core_positions() {
    let game = adapter();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(
        snapshot.players.iter().map(|p| p.id).collect::<Vec<_>>(),
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
    let mut game = TexasHoldemAdapter::new(
        MatchId([8; 16]),
        52300,
        HOST,
        vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
        TexasHoldemRuleSet {
            omaha: true,
            ..TexasHoldemRuleSet::default()
        },
        HOST,
        build_deck(false),
    )
    .unwrap();
    assert_eq!(game.snapshot(HOST).unwrap().your_hole_cards.len(), 4);

    game.act(LEFT, TexasHoldemAction::PostBlind).unwrap();
    game.act(RIGHT, TexasHoldemAction::PostBlind).unwrap();
    game.act(HOST, TexasHoldemAction::AllIn).unwrap();
    game.act(LEFT, TexasHoldemAction::AllIn).unwrap();
    game.act(RIGHT, TexasHoldemAction::Call).unwrap();

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
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(523),
        revision: Revision(9),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
}

#[test]
fn adapter_emits_action_and_street_events() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::Call).unwrap();
    game.act(LEFT, TexasHoldemAction::Call).unwrap();
    let events = game.act(RIGHT, TexasHoldemAction::Check).unwrap();
    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        TexasHoldemEvent::ActionApplied {
            player: RIGHT,
            action: TexasHoldemAction::Check,
            amount: 0,
        }
    ));
    assert!(matches!(
        &events[1],
        TexasHoldemEvent::StreetAdvanced { street, dealt }
            if *street == TexasHoldemStreet::Flop && dealt.len() == 3
    ));
    assert_eq!(game.snapshot(HOST).unwrap().community.len(), 3);
}

#[test]
fn rejected_action_returns_a_wire_violation_without_mutating_state() {
    let mut game = adapter();
    let before = game.game().clone();
    assert_eq!(
        game.act(LEFT, TexasHoldemAction::Call),
        Err(AdapterError::Violation(
            TexasHoldemViolation::NotPlayersTurn
        ))
    );
    assert_eq!(game.game(), &before);
}

#[test]
fn showdown_reveals_all_hands_and_exposes_pot_awards() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::AllIn).unwrap();
    game.act(LEFT, TexasHoldemAction::AllIn).unwrap();
    let events = game.act(RIGHT, TexasHoldemAction::Call).unwrap();
    assert!(matches!(
        events.last(),
        Some(TexasHoldemEvent::HandFinished { showdown: true })
    ));
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.community.len(), 5);
    assert_eq!(snapshot.revealed_hands.len(), 3);
    let TexasHoldemPhaseView::HandComplete {
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
    game.act(HOST, TexasHoldemAction::Fold).unwrap();
    game.act(LEFT, TexasHoldemAction::AllIn).unwrap();
    game.act(RIGHT, TexasHoldemAction::Call).unwrap();
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
    game.act(HOST, TexasHoldemAction::Fold).unwrap();
    let events = game.act(LEFT, TexasHoldemAction::Fold).unwrap();
    assert!(matches!(
        events.last(),
        Some(TexasHoldemEvent::HandFinished { showdown: false })
    ));
    let snapshot = game.snapshot(RIGHT).unwrap();
    assert!(snapshot.revealed_hands.is_empty());
}

#[test]
fn next_hand_keeps_stacks_and_rotates_the_dealer() {
    let mut game = adapter();
    game.act(HOST, TexasHoldemAction::Fold).unwrap();
    game.act(LEFT, TexasHoldemAction::Fold).unwrap();
    game.start_next_hand(build_deck(false)).unwrap();
    let snapshot = game.snapshot(HOST).unwrap();
    assert_eq!(snapshot.hand_number, 1);
    assert_eq!(snapshot.dealer, LEFT);
    assert!(matches!(
        snapshot.phase,
        TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::PreFlop
        }
    ));
}

#[test]
fn private_snapshot_round_trips_through_the_wire_frame() {
    let snapshot = adapter().snapshot(HOST).unwrap();
    let message = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(523),
        revision: Revision(8),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot.clone())),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
    let ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(decoded_snapshot)) = decoded.event
    else {
        panic!("wire frame changed the concrete game variant");
    };
    assert_eq!(decoded_snapshot.your_hole_cards, snapshot.your_hole_cards);
    assert!(decoded_snapshot.revealed_hands.is_empty());
}
