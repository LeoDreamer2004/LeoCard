use super::*;
use leocard_protocol::{ClientCommand, GameCommand, QiGui523Command, ServerEvent};
use leocard_qigui523::TimeControl;
use leocard_qigui523::{
    PlayRecord, QiGui523Bot, QiGui523BotRequest, QiGuiPlayerId, QiGuiRuleSet, build_deck, classify,
};

#[test]
fn enabling_auto_play_waits_one_second_before_acting_on_own_turn() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let current = from_core_player(session.game().unwrap().trick().unwrap().current_player());
    let connection = connection_for_player(&session, current);
    let hand_len_before = session
        .game()
        .unwrap()
        .player(to_core_player(current))
        .unwrap()
        .hand()
        .len();

    let enabled = send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
            enabled: true,
        })),
    );

    assert!(session.players[usize::from(current.0)].auto_play);
    assert_eq!(
        session
            .game()
            .unwrap()
            .player(to_core_player(current))
            .unwrap()
            .hand()
            .len(),
        hand_len_before
    );
    assert!(!enabled.iter().any(|delivery| {
        qigui523_play_effect(&delivery.message.event).is_some_and(|(player, _)| player == current)
    }));
    assert!(
        enabled
            .iter()
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(snapshot) => snapshot.qigui523(),
                _ => None,
            })
            .all(|snapshot| snapshot
                .players
                .iter()
                .find(|player| player.id == current)
                .is_some_and(|player| player.auto_play))
    );

    assert!(session.advance_time(Duration::from_millis(999)).is_empty());
    assert_eq!(
        session
            .game()
            .unwrap()
            .player(to_core_player(current))
            .unwrap()
            .hand()
            .len(),
        hand_len_before
    );
    let acted = session.advance_time(Duration::from_millis(1));
    assert!(acted.iter().any(|delivery| {
        qigui523_play_effect(&delivery.message.event).is_some_and(|(player, _)| player == current)
    }));
    assert_eq!(
        session
            .game()
            .unwrap()
            .player(to_core_player(current))
            .unwrap()
            .hand()
            .len(),
        hand_len_before - 1
    );

    let disabled = send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
            enabled: false,
        })),
    );
    assert!(!session.players[usize::from(current.0)].auto_play);
    assert!(
        disabled
            .iter()
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(snapshot) => snapshot.qigui523(),
                _ => None,
            })
            .all(|snapshot| snapshot
                .players
                .iter()
                .find(|player| player.id == current)
                .is_some_and(|player| !player.auto_play))
    );
}

#[test]
fn auto_play_also_waits_one_second_when_its_turn_arrives_later() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let (leader, managed, lead_card) = {
        let game = session.game().unwrap();
        let leader = game.trick().unwrap().current_player();
        let managed = QiGuiPlayerId((leader.0 + game.players().len() - 1) % game.players().len());
        let lead_card = game.player(leader).unwrap().hand()[0];
        (
            from_core_player(leader),
            from_core_player(managed),
            lead_card,
        )
    };
    let managed_connection = connection_for_player(&session, managed);
    send_next(
        &mut session,
        managed_connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
            enabled: true,
        })),
    );
    let leader_connection = connection_for_player(&session, leader);
    send_next(
        &mut session,
        leader_connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: vec![lead_card],
        })),
    );
    assert_eq!(
        from_core_player(session.game().unwrap().trick().unwrap().current_player()),
        managed
    );
    assert_eq!(session.game().unwrap().trick().unwrap().records().len(), 1);

    assert!(session.advance_time(Duration::from_millis(999)).is_empty());
    assert_eq!(session.game().unwrap().trick().unwrap().records().len(), 1);
    assert!(!session.advance_time(Duration::from_millis(1)).is_empty());
    assert!(session.game().unwrap().trick().unwrap().records().len() > 1);
}

#[test]
fn timeout_follow_uses_the_smallest_greedy_response() {
    let configured_rules = QiGuiRuleSet {
        time_control: TimeControl::FivePlusTen,
        ..rules()
    };
    let mut session = QiGui523Session::new(
        ROOM,
        52300,
        configured_rules,
        build_deck(configured_rules.deck_count),
    )
    .unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let (leader, lead_card, follower, expected) = {
        let game = session.game().unwrap();
        let leader = game.trick().unwrap().current_player();
        let follower = QiGuiPlayerId((leader.0 + game.players().len() - 1) % game.players().len());
        game.player(leader)
            .unwrap()
            .hand()
            .iter()
            .copied()
            .find_map(|lead_card| {
                let current_play = classify(&[lead_card], game.rules()).unwrap();
                let response = QiGui523Bot::new().choose(QiGui523BotRequest {
                    hand: game.player(follower).unwrap().hand(),
                    current_play: &current_play,
                    played_cards: &[lead_card],
                    rules: game.rules(),
                })?;
                Some((leader, lead_card, follower, response.cards().to_vec()))
            })
            .expect("deterministic hands contain a beatable single")
    };

    let leader = from_core_player(leader);
    let connection = connection_for_player(&session, leader);
    send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: vec![lead_card],
        })),
    );
    assert_eq!(
        session.turn_timer.as_ref().unwrap().player,
        from_core_player(follower)
    );

    session.advance_time(Duration::from_secs(15));
    let last_record = session
        .game()
        .unwrap()
        .trick()
        .unwrap()
        .records()
        .last()
        .unwrap();
    assert!(matches!(
        last_record,
        PlayRecord::Played { player, play }
            if *player == follower && play.cards() == expected
    ));
}

#[test]
fn disconnected_current_player_is_replaced_immediately_without_spending_time() {
    let configured_rules = QiGuiRuleSet {
        time_control: TimeControl::FivePlusTen,
        ..rules()
    };
    let mut session = QiGui523Session::new(
        ROOM,
        52300,
        configured_rules,
        build_deck(configured_rules.deck_count),
    )
    .unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    if session
        .turn_timer
        .as_ref()
        .is_some_and(|timer| connection_for_player(&session, timer.player) == HOST)
    {
        session.play_automatic_action();
        session.reset_timer_for_current_turn();
    }
    let disconnected = session.turn_timer.as_ref().unwrap().player;
    let connection = connection_for_player(&session, disconnected);
    assert_ne!(connection, HOST);
    let records_before = session.game().unwrap().trick().unwrap().records().len();
    let deliveries = session.disconnect(connection);

    assert!(!deliveries.is_empty());
    let game = session.game().unwrap();
    assert!(matches!(
        game.trick().unwrap().records().get(records_before),
        Some(PlayRecord::Played { player, play })
            if *player == to_core_player(disconnected) && play.cards().len() == 1
    ));
    let timer = session.turn_timer_view().unwrap();
    assert_ne!(timer.player, disconnected);
    assert_eq!(timer.base_seconds, 5);
    assert_eq!(timer.reserve_seconds, 10);
}
