use super::*;

#[test]
fn two_players_can_start_and_receive_private_hands() {
    let mut session = UnoSession::new(ROOM, UnoRuleSet::default(), build_deck()).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    let deliveries = session.handle(HOST, message(2, ClientCommand::StartGame));
    let snapshots = deliveries
        .iter()
        .filter_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) => Some(snapshot),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 2);
    assert!(
        snapshots
            .iter()
            .all(|snapshot| snapshot.your_hand.len() == 7)
    );
    assert!(snapshots.iter().all(|snapshot| snapshot.players.len() == 2));
}

#[test]
fn flip_snapshots_hide_your_inactive_faces_but_show_opponents_backs() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, rules, build_deck_for_rules(rules)).unwrap();
    session.handle(HOST, message(1, join_command("甲", 1)));
    let second = ConnectionId(2);
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    let deliveries = session.handle(HOST, message(2, ClientCommand::StartGame));

    for delivery in deliveries {
        let ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) = delivery.message.event else {
            continue;
        };
        let own = snapshot
            .players
            .iter()
            .find(|player| player.id == snapshot.you)
            .unwrap();
        assert!(own.inactive_hand.is_empty());
        assert!(
            snapshot
                .your_hand
                .iter()
                .all(|card| card.opposite().is_none())
        );
        assert!(
            snapshot
                .players
                .iter()
                .filter(|player| player.id != snapshot.you)
                .all(|player| player.inactive_hand.len() == player.hand_len as usize)
        );
        assert_eq!(snapshot.draw_pile_inactive_cards.len(), 6);
    }
}

#[test]
fn no_mercy_draw_until_playable_reveals_authoritative_cards_one_at_a_time() {
    fn place(deck: &mut [UnoCard], index: usize, card: UnoCard) {
        let current = deck
            .iter()
            .position(|candidate| *candidate == card)
            .expect("No Mercy deck contains the requested card");
        deck.swap(index, current);
    }

    let rules = UnoRuleSet {
        mode: Mode::NoMercy,
        ..UnoRuleSet::default()
    };
    let mut deck = build_no_mercy_deck();
    for (index, value) in [0, 1, 2, 3, 4, 6, 7].into_iter().enumerate() {
        place(
            &mut deck,
            index * 2,
            UnoCard::number(UnoColor::Blue, value, 0),
        );
    }
    let top = UnoCard::number(UnoColor::Red, 5, 0);
    let miss = UnoCard::number(UnoColor::Green, 8, 0);
    let playable = UnoCard::number(UnoColor::Red, 3, 0);
    place(&mut deck, 14, top);
    place(&mut deck, 15, miss);
    place(&mut deck, 16, playable);

    let mut session = UnoSession::new(ROOM, rules, deck).unwrap();
    let second = ConnectionId(2);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    for (index, player) in session.room.players.iter_mut().enumerate() {
        player.seat = Some(SeatId(index as u8));
    }
    session.handle(second, message(3, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(3, ClientCommand::StartGame));

    let deliveries = session.handle(
        HOST,
        message(
            4,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::DrawCard)),
        ),
    );
    let snapshot = deliveries
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot))
                if delivery.recipient == HOST =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(snapshot.your_hand.len(), 7);
    assert_eq!(snapshot.current_player, None);
    assert!(session.advance_time(Duration::from_millis(779)).is_empty());

    let first = session.advance_time(Duration::from_millis(1));
    let first = first
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot))
                if delivery.recipient == HOST =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(first.your_hand.len(), 8);
    assert!(first.your_hand.contains(&miss));
    assert_eq!(first.current_player, None);

    let second = session.advance_time(DRAW_REVEAL_INTERVAL);
    let second = second
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot))
                if delivery.recipient == HOST =>
            {
                Some(snapshot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(second.your_hand.len(), 9);
    assert!(second.your_hand.contains(&playable));
    assert_eq!(second.current_player, Some(PlayerId(0)));
    assert_eq!(second.your_drawn_card, Some(playable));
}

#[test]
fn flip_draw_events_expose_the_matching_card_backs_in_order() {
    let cards = build_deck_for_rules(UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    })
    .into_iter()
    .take(3)
    .collect::<Vec<_>>();
    let expected = cards
        .iter()
        .filter_map(|card| card.opposite_public_face())
        .collect::<Vec<_>>();

    let outcome = ActionOutcome::ColorRouletteResolved {
        player: UnoPlayerId(0),
        color: UnoColor::Pink,
        cards: cards.clone(),
        next_player: UnoPlayerId(1),
    };
    let events = events_for_outcome(&outcome, &[]);
    assert_eq!(
        events,
        vec![UnoEvent::ColorRouletteResolved {
            player: PlayerId(0),
            color: UnoColor::Pink,
            count: 3,
            card_backs: expected,
        }]
    );

    let game = GameState::new_with_deck(
        UnoRuleSet {
            mode: Mode::Flip,
            ..UnoRuleSet::default()
        },
        2,
        build_deck_for_rules(UnoRuleSet {
            mode: Mode::Flip,
            ..UnoRuleSet::default()
        }),
    )
    .unwrap();
    let reveal = pending_draw_reveal_for_outcome(&outcome, &game).unwrap();
    assert_eq!(reveal.cards, cards);
    assert_eq!(reveal.revealed, 0);
    assert_eq!(reveal.remaining, COLOR_ROULETTE_REVEAL_START_DELAY);
}

#[test]
fn flip_color_draw_reveals_cards_and_draw_pile_backs_one_at_a_time() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, rules, build_deck_for_rules(rules)).unwrap();
    let second = ConnectionId(2);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));

    let cards = session
        .game
        .as_ref()
        .unwrap()
        .player(UnoPlayerId(0))
        .unwrap()
        .hand()
        .iter()
        .copied()
        .take(2)
        .collect::<Vec<_>>();
    session.pending_draw_reveal = Some(PendingDrawReveal {
        player: PlayerId(0),
        cards: cards.clone(),
        revealed: 0,
        remaining: COLOR_ROULETTE_REVEAL_START_DELAY,
    });

    let initial = session.game_snapshot(PlayerId(1));
    assert_eq!(
        initial.draw_pile_inactive_cards.first().copied(),
        cards[0].opposite_public_face()
    );
    assert_eq!(
        initial.players[0].hand_len,
        session
            .game
            .as_ref()
            .unwrap()
            .player(UnoPlayerId(0))
            .unwrap()
            .hand()
            .len() as u8
            - 2
    );

    session.advance_time(COLOR_ROULETTE_REVEAL_START_DELAY);
    let after_first = session.game_snapshot(PlayerId(1));
    assert_eq!(
        after_first.draw_pile_inactive_cards.first().copied(),
        cards[1].opposite_public_face()
    );
    assert_eq!(
        after_first.players[0].hand_len,
        initial.players[0].hand_len + 1
    );

    session.advance_time(DRAW_REVEAL_INTERVAL);
    assert!(session.pending_draw_reveal.is_none());
}

#[test]
fn uno_call_and_report_remain_reactive_during_incremental_draws() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, rules, build_deck_for_rules(rules)).unwrap();
    let second = ConnectionId(2);
    session.handle(HOST, message(1, join_command("甲", 1)));
    session.handle(second, message(1, join_command("乙", 2)));
    session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
    session.handle(HOST, message(2, ClientCommand::StartGame));

    let cards = session
        .game
        .as_ref()
        .unwrap()
        .player(UnoPlayerId(0))
        .unwrap()
        .hand()[..2]
        .to_vec();
    session.pending_draw_reveal = Some(PendingDrawReveal {
        player: PlayerId(0),
        cards,
        revealed: 0,
        remaining: COLOR_ROULETTE_REVEAL_START_DELAY,
    });

    let call = session.handle(
        HOST,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::CallUno)),
        ),
    );
    assert!(call.iter().any(|delivery| {
        matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Uno(UnoViolation::CannotCallUno))
            }
        )
    }));
    assert!(session.pending_draw_reveal.is_some());

    let reporter = session.room.player_id(second).unwrap();
    let target = session
        .room
        .players
        .iter()
        .find(|player| player.id != reporter)
        .unwrap()
        .id;
    let report = session.handle(
        second,
        message(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::ReportUno { target })),
        ),
    );
    assert!(report.iter().any(|delivery| {
        matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Uno(UnoViolation::PlayerNotReportable))
            }
        )
    }));
    assert!(session.pending_draw_reveal.is_some());
}
