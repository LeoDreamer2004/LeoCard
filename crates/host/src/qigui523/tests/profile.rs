use super::*;
use leocard_qigui523::{BombKind, QiGuiRank, TimeControl};
#[test]
fn player_name_limit_counts_unicode_characters() {
    let mut accepted = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    let deliveries = accepted.handle(
        HOST,
        message(1, join_command("一二三四五六七", ReconnectToken(HOST.0))),
    );
    assert_eq!(rejection(&deliveries), None);

    let mut rejected = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    let deliveries = rejected.handle(
        HOST,
        message(1, join_command("一二三四五六七八", ReconnectToken(HOST.0))),
    );
    assert_eq!(
        rejection(&deliveries),
        Some(&RejectReason::NameTooLong {
            max_chars: MAX_PLAYER_NAME_CHARS as u16,
        })
    );
}

#[test]
fn invalid_player_identity_signature_is_rejected() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    let mut command = join_command("玩家", ReconnectToken(HOST.0));
    let ClientCommand::Join(request) = &mut command else {
        unreachable!()
    };
    request.identity_signature[0] ^= 0x80;

    let deliveries = session.handle(HOST, message(1, command));

    assert_eq!(
        rejection(&deliveries),
        Some(&RejectReason::InvalidIdentityProof)
    );
    assert!(session.players.is_empty());
}

#[test]
fn finished_match_applies_reference_points_exactly_once() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    finish_game(&mut session);
    let scores = match session.game().unwrap().phase() {
        Phase::Finished(result) => result.scores.clone(),
        Phase::Playing => unreachable!(),
    };
    let expected = reference_point_deltas(&scores).unwrap();
    let expected_remaining_hands = session
        .game()
        .unwrap()
        .players()
        .iter()
        .enumerate()
        .map(|(index, player)| (PlayerId(index as u8), player.hand().to_vec()))
        .collect::<Vec<_>>();

    let deliveries = session.broadcast_game_after_update(None);

    assert_eq!(
        session
            .players
            .iter()
            .map(|player| player.reference_points)
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|delta| i32::from(*delta))
            .collect::<Vec<_>>()
    );
    assert!(
        session
            .players
            .iter()
            .all(|player| player.completed_games == 1)
    );
    for ((player, score), delta) in session
        .players
        .iter()
        .zip(scores.iter().copied())
        .zip(expected.iter().copied())
    {
        let stats = player.game_profiles.qigui523.as_ref().unwrap();
        assert_eq!(stats.completed_games, 1);
        assert_eq!(stats.total_score, u64::from(score));
        assert_eq!(stats.total_reference_delta, i64::from(delta));
        let placement = 1 + scores.iter().filter(|other| **other > score).count();
        assert_eq!(stats.placement_counts.iter().sum::<u32>(), 1);
        assert_eq!(stats.placement_counts[placement - 1], 1);
    }
    assert!(deliveries.iter().all(|delivery| {
        qigui523_snapshot(&delivery.message.event).is_some_and(|snapshot| {
            matches!(
                &snapshot.phase,
                GamePhaseView::Finished {
                    reference_changes,
                    remaining_hands,
                    ..
                } if reference_changes.len() == expected.len()
                    && remaining_hands
                        .iter()
                        .map(|hand| (hand.player, hand.cards.clone()))
                        .collect::<Vec<_>>() == expected_remaining_hands
            )
        })
    }));

    session.broadcast_game_after_update(None);
    assert_eq!(
        session
            .players
            .iter()
            .map(|player| player.reference_points)
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|delta| i32::from(*delta))
            .collect::<Vec<_>>()
    );
    assert!(
        session
            .players
            .iter()
            .all(|player| player.completed_games == 1)
    );
    assert!(session.players.iter().all(|player| {
        player
            .game_profiles
            .qigui523
            .as_ref()
            .is_some_and(|stats| stats.completed_games == 1)
    }));
}

#[test]
fn qigui523_profile_play_statistics_count_types_and_keep_longest_lengths() {
    let mut stats = QiGui523ProfileStats::default();
    for kind in [
        QiGuiPlayKind::Straight { card_count: 5 },
        QiGuiPlayKind::Straight { card_count: 8 },
        QiGuiPlayKind::ConsecutivePairs { pair_count: 3 },
        QiGuiPlayKind::Airplane { triple_count: 2 },
        QiGuiPlayKind::Bomb(BombKind::OfAKind {
            card_count: 4,
            rank: QiGuiRank::Ace,
        }),
        QiGuiPlayKind::HeavenBomb,
    ] {
        record_qigui523_play(&mut stats, &kind);
    }

    assert_eq!(stats.straight_plays, 2);
    assert_eq!(stats.consecutive_pair_plays, 1);
    assert_eq!(stats.airplane_plays, 1);
    assert_eq!(stats.bomb_plays, 1);
    assert_eq!(stats.heaven_bomb_plays, 1);
    assert_eq!(stats.longest_straight, 8);
    assert_eq!(stats.longest_consecutive_pairs, 3);
    assert_eq!(stats.longest_airplane, 2);
}

#[test]
fn only_host_can_close_room_and_every_connected_player_is_notified() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);

    let denied = session.handle(SECOND, message(3, ClientCommand::CloseRoom));
    assert_eq!(
        rejection(&denied),
        Some(&RejectReason::OnlyHostCanCloseRoom)
    );
    assert!(!session.is_closed());

    let deliveries = session.handle(HOST, message(3, ClientCommand::CloseRoom));
    assert!(session.is_closed());
    assert_eq!(deliveries.len(), 3);
    assert!(
        deliveries
            .iter()
            .all(|delivery| { matches!(delivery.message.event, ServerEvent::RoomClosed) })
    );
}

#[test]
fn unlimited_time_control_never_creates_or_advances_a_turn_timer() {
    let configured_rules = QiGuiRuleSet {
        time_control: TimeControl::Unlimited,
        ..rules()
    };
    let mut session = QiGui523Session::new(
        ROOM,
        configured_rules,
        build_deck(configured_rules.deck_count),
    )
    .unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let current = session.game().unwrap().trick().unwrap().current_player();

    assert!(session.turn_timer.is_none());
    assert!(session.advance_time(Duration::from_secs(600)).is_empty());
    assert_eq!(
        session.game().unwrap().trick().unwrap().current_player(),
        current
    );
}

#[test]
fn turn_timer_spends_base_before_persistent_player_reserve() {
    let configured_rules = QiGuiRuleSet {
        time_control: TimeControl::FivePlusTen,
        ..rules()
    };
    let mut session = QiGui523Session::new(
        ROOM,
        configured_rules,
        build_deck(configured_rules.deck_count),
    )
    .unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let timed_player = session.turn_timer.as_ref().unwrap().player;
    assert_eq!(session.turn_timer_view().unwrap().base_seconds, 5);
    assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 10);

    session.advance_time(Duration::from_secs(5));
    assert_eq!(session.turn_timer_view().unwrap().base_seconds, 0);
    assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 10);
    session.advance_time(Duration::from_secs(3));
    assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 7);

    let card = session
        .game()
        .unwrap()
        .player(to_core_player(timed_player))
        .unwrap()
        .hand()[0];
    let connection = connection_for_player(&session, timed_player);
    send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: vec![card],
        })),
    );

    let timer = session.turn_timer.as_ref().unwrap();
    assert_eq!(
        timer.reserve_remaining[usize::from(timed_player.0)],
        Duration::from_secs(7)
    );
    assert_eq!(duration_ceil_seconds(timer.base_remaining), 5);
}

#[test]
fn timeout_lead_plays_exactly_the_smallest_single_card() {
    let configured_rules = QiGuiRuleSet {
        time_control: TimeControl::FivePlusTen,
        ..rules()
    };
    let mut session = QiGui523Session::new(
        ROOM,
        configured_rules,
        build_deck(configured_rules.deck_count),
    )
    .unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let player = session.turn_timer.as_ref().unwrap().player;
    let expected = session
        .game()
        .unwrap()
        .player(to_core_player(player))
        .unwrap()
        .hand()
        .iter()
        .copied()
        .min_by_key(|card| (card.rank().strength(), card.suit().strength(), card.deck()))
        .unwrap();

    let deliveries = session.advance_time(Duration::from_secs(15));
    assert!(!deliveries.is_empty());
    let game = session.game().unwrap();
    assert!(matches!(
        &game.trick().unwrap().records()[0],
        PlayRecord::Played { player: record_player, play }
            if *record_player == to_core_player(player) && play.cards() == [expected]
    ));
    assert_eq!(session.turn_timer_view().unwrap().base_seconds, 5);
    assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 10);
}
