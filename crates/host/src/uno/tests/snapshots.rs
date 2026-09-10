use super::*;

#[test]
fn flip_snapshots_hide_your_inactive_faces_but_show_opponents_backs() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    };
    let mut session = UnoSession::new(ROOM, 52300, rules, build_deck_for_rules(rules)).unwrap();
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

    let mut session = UnoSession::new(ROOM, 52300, rules, deck).unwrap();
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
