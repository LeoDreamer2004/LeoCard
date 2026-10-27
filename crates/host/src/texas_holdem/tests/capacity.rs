use super::super::*;
use super::*;
use crate::ConnectionId;
use leocard_protocol::{
    ClientCommand, GameCommand, GameViolation, RejectReason, ServerEvent, TexasHoldemCommand,
};
use leocard_texas_holdem::{TexasHoldemRuleSet, build_deck};

fn joined_session(count: u8, rules: TexasHoldemRuleSet) -> TexasHoldemSession {
    let mut session =
        TexasHoldemSession::new(ROOM, 52300, rules, build_deck(rules.short_deck)).unwrap();
    for index in 1..=count {
        let connection = ConnectionId(u64::from(index) * 10);
        session.handle(
            connection,
            message(1, join_command(&format!("玩家{index}"), u64::from(index))),
        );
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    session
}

#[test]
fn eight_players_can_join_and_start_standard_short_deck_or_omaha_tables() {
    for (short_deck, omaha) in [(false, false), (true, false), (false, true)] {
        let mut session = joined_session(
            8,
            TexasHoldemRuleSet {
                short_deck,
                omaha,
                ..TexasHoldemRuleSet::default()
            },
        );
        assert_eq!(session.room.seat_count, 8);
        let deliveries = session.handle(HOST_CONNECTION, message(3, ClientCommand::StartGame));
        assert!(
            deliveries
                .iter()
                .any(|delivery| matches!(delivery.message.event, ServerEvent::GameSnapshot(_)))
        );
        let game = session.game().unwrap();
        assert_eq!(game.players().len(), 8);
        for player in game.players() {
            let snapshot = game.snapshot(player.id).unwrap();
            assert_eq!(snapshot.your_hole_cards.len(), if omaha { 4 } else { 2 });
            assert!(snapshot.revealed_hands.is_empty());
        }
    }
}

#[test]
fn short_deck_omaha_rejects_seven_players_until_the_rules_are_changed() {
    let rules = TexasHoldemRuleSet {
        short_deck: true,
        omaha: true,
        ..TexasHoldemRuleSet::default()
    };
    let mut six = joined_session(6, rules);
    six.handle(HOST_CONNECTION, message(3, ClientCommand::StartGame));
    assert!(six.game().is_some());

    let mut seven = joined_session(7, rules);
    let deliveries = seven.handle(HOST_CONNECTION, message(3, ClientCommand::StartGame));
    assert!(deliveries.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            ..
        }
    )));
    assert!(seven.game().is_none());
    assert_eq!(seven.room.players.len(), 7);
    let invalid = seven.handle(
        HOST_CONNECTION,
        message(
            4,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::UpdateRules {
                rules,
            })),
        ),
    );
    assert!(
        invalid
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    seven.handle(
        HOST_CONNECTION,
        message(
            5,
            ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::UpdateRules {
                rules: TexasHoldemRuleSet {
                    short_deck: false,
                    ..rules
                },
            })),
        ),
    );
    for index in 1..=7 {
        seven.handle(
            ConnectionId(index * 10),
            message(6, ClientCommand::SetReady { ready: true }),
        );
    }
    seven.handle(HOST_CONNECTION, message(7, ClientCommand::StartGame));
    assert_eq!(seven.game().unwrap().players().len(), 7);
}

#[test]
fn exact_equity_handles_seven_and_eight_way_split_pots() {
    use super::super::equity::EquityCache;
    use leocard_texas_holdem::{
        GameState, TexasHoldemAction, TexasHoldemCard, TexasHoldemPlayerId, TexasHoldemRank,
        TexasHoldemSuit,
    };
    use std::thread;
    use std::time::{Duration, Instant};

    for player_count in [7, 8] {
        let mut deck = build_deck(false);
        let dealt = usize::from(player_count) * 2;
        for (index, rank) in (dealt..dealt + 5).zip([
            TexasHoldemRank::Ten,
            TexasHoldemRank::Jack,
            TexasHoldemRank::Queen,
            TexasHoldemRank::King,
            TexasHoldemRank::Ace,
        ]) {
            let card = TexasHoldemCard::new(TexasHoldemSuit::Spade, rank);
            let source = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(index, source);
        }
        let mut game = GameState::new_with_deck(
            TexasHoldemRuleSet {
                player_count,
                ..TexasHoldemRuleSet::default()
            },
            TexasHoldemPlayerId(0),
            deck,
        )
        .unwrap();
        while let Some(player) = game.current_player() {
            let action = if game.blind_to_post().is_some() {
                TexasHoldemAction::PostBlind
            } else {
                TexasHoldemAction::AllIn
            };
            game.act(player, action).unwrap();
        }
        let cache = EquityCache::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        let values = loop {
            if let Some(values) = cache.values(&game) {
                break values;
            }
            assert!(
                Instant::now() < deadline,
                "equity calculation did not finish"
            );
            thread::sleep(Duration::from_millis(1));
        };
        let expected = (10_000.0 / f64::from(player_count)).round() as u16;
        assert_eq!(values, vec![expected; usize::from(player_count)]);
    }
}
