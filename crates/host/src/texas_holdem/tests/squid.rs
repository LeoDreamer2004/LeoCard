use super::super::TexasHoldemSession;
use super::fixtures::{
    HOST_CONNECTION, ROOM, SECOND_CONNECTION, THIRD_CONNECTION, join_command, message,
};
use leocard_protocol::{
    ClientCommand, GameViolation, RejectReason, ServerEvent, TexasHoldemCommand,
};
use leocard_texas_holdem::{TexasHoldemRuleSet, build_deck};

#[test]
fn lobby_enforces_the_current_player_count_instead_of_table_capacity() {
    let mut session = TexasHoldemSession::new(
        ROOM,
        52300,
        TexasHoldemRuleSet::default(),
        build_deck(false),
    )
    .unwrap();
    for (connection, token) in [
        (HOST_CONNECTION, 1),
        (SECOND_CONNECTION, 2),
        (THIRD_CONNECTION, 3),
    ] {
        session.handle(connection, message(1, join_command("玩家", token)));
    }
    for (request, count) in [(2, 3), (3, 2)] {
        let rules = TexasHoldemRuleSet {
            squid_markers: count,
            ..Default::default()
        };
        let deliveries = session.handle(
            HOST_CONNECTION,
            message(
                request,
                ClientCommand::Game(TexasHoldemCommand::UpdateRules { rules }.into()),
            ),
        );
        assert_eq!(
            deliveries.iter().any(|delivery| matches!(
                delivery.message.event,
                ServerEvent::Rejected {
                    reason: RejectReason::Game(GameViolation::InvalidRuleConfiguration)
                }
            )),
            count == 3
        );
    }
}
