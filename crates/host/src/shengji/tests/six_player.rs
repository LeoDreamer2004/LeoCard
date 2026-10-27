use super::settlement::result_for_reference_points;
use super::support::*;
use leocard_protocol::{
    ClientCommand, GameCommand, PlayerId, SeatId, ShengjiCommand, ShengjiPhaseView,
};
use leocard_shengji::{
    ShengjiCard, ShengjiMode, ShengjiPlayerId, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    build_deck_for,
};

#[test]
fn six_player_lobby_deals_private_hands_and_settles_every_participant() {
    let rules = ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer);
    let mut deck = build_deck_for(3);
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(5, index);
    let mut session = ShengjiSession::new(ROOM, 52300, rules, deck).unwrap();
    let connections = (0..6)
        .map(|index| ConnectionId(10 * (index + 1)))
        .collect::<Vec<_>>();
    for (index, connection) in connections.iter().copied().enumerate() {
        let index = index as u8;
        session.handle(connection, message(index, 1, join_command(index)));
        session.handle(
            connection,
            message(
                index,
                2,
                ClientCommand::SelectSeat {
                    seat: SeatId(index),
                },
            ),
        );
        session.handle(
            connection,
            message(index, 3, ClientCommand::SetReady { ready: true }),
        );
    }
    assert_eq!(session.room.players.len(), 6);
    session.handle(connections[0], message(0, 4, ClientCommand::StartGame));
    session.advance_time(DEAL_INTERVAL * 6);
    let declared = session.handle(
        connections[5],
        message(
            5,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![target],
            })),
        ),
    );
    assert_eq!(
        game_snapshot(&declared, connections[5]).dealer,
        Some(PlayerId(5))
    );
    assert_eq!(
        game_snapshot(&declared, connections[5]).your_exposed_cards,
        vec![target]
    );
    assert!(
        game_snapshot(&declared, connections[0])
            .your_exposed_cards
            .is_empty()
    );
    session.advance_time(DEAL_INTERVAL * 150);
    for (index, connection) in connections.iter().copied().enumerate() {
        session.handle(
            connection,
            message(
                index as u8,
                10,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
            ),
        );
    }
    let game = session.game.as_ref().unwrap();
    assert_eq!(game.players().len(), 6);
    assert_eq!(game.players()[5].hand.len(), 32);
    let buried = game.players()[5].hand[..6].to_vec();
    let deliveries = session.handle(
        connections[5],
        message(
            5,
            11,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury {
                cards: buried.clone(),
            })),
        ),
    );
    for (index, connection) in connections.iter().copied().enumerate() {
        let snapshot = game_snapshot(&deliveries, connection);
        assert_eq!(snapshot.your_hand.len(), 26);
        assert_eq!(snapshot.players.len(), 6);
        assert_eq!(snapshot.phase, ShengjiPhaseView::Playing);
        if index == 5 {
            assert_eq!(snapshot.your_buried, buried);
        } else {
            assert!(snapshot.your_buried.is_empty());
        }
        assert_eq!(
            session.game.as_ref().unwrap().players()[index].id,
            ShengjiPlayerId(index as u8)
        );
    }
    session.apply_finished_reference_points(&result_for_reference_points(false, 1, 120));
    let changes = session
        .statistics
        .hand
        .finished_reference_changes
        .as_ref()
        .unwrap();
    assert_eq!(changes.len(), 6);
    assert!(
        session
            .room
            .players
            .iter()
            .all(|player| player.completed_games == 1 && player.game_profiles.shengji.is_some())
    );
}
