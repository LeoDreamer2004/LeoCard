use super::*;
use leocard_ai::{DevicePreference, ModelId, ModelRegistry, Tensor, TensorMap};
use leocard_protocol::RoomViolation;

#[test]
fn only_host_selects_ai_and_active_match_retains_its_model() {
    let id = ModelId::Douzero;
    let weights = TensorMap::from([("test".into(), Tensor::new(vec![1], vec![1.0]).unwrap())]);
    ModelRegistry::install(id, vec![weights; 3]).unwrap();
    let mut session = DoudizhuSession::new(
        RoomId(99),
        52300,
        DoudizhuRuleSet::default(),
        build_deck(DoudizhuMode::Three),
    )
    .unwrap();
    for connection in 1..=2 {
        assert_accepted(&send(&mut session, connection, join(connection)));
    }
    assert_accepted(&send(
        &mut session,
        1,
        ClientCommand::SelectSeat { seat: SeatId(0) },
    ));
    let configure = ClientCommand::ConfigureBotAi {
        model: Some(id),
        device: DevicePreference::Cpu,
    };
    let rejected = send(&mut session, 2, configure.clone());
    assert!(rejected.iter().any(|d| matches!(
        d.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Room(RoomViolation::OnlyHostCanConfigure)
        }
    )));
    assert_accepted(&send(&mut session, 2, ClientCommand::LeaveRoom));
    assert_accepted(&send(
        &mut session,
        1,
        ClientCommand::ConfigureBotSeat {
            seat: SeatId(1),
            occupied: true,
        },
    ));
    assert_accepted(&send(&mut session, 1, configure));
    assert_accepted(&send(
        &mut session,
        1,
        ClientCommand::ConfigureBotSeat {
            seat: SeatId(2),
            occupied: true,
        },
    ));
    let names: Vec<_> = session
        .room
        .players
        .iter()
        .filter(|p| p.is_bot)
        .map(|p| p.name.as_str())
        .collect();
    assert_eq!(names, ["绝悟人机1", "绝悟人机2"]);
    assert_accepted(&send(
        &mut session,
        1,
        ClientCommand::SetReady { ready: true },
    ));
    assert_accepted(&send(&mut session, 1, ClientCommand::StartGame));
    let original = session.room.bot_ai.clone().unwrap();
    ModelRegistry::remove(id);
    assert!(original.tensors(0).is_ok());
    let rejected = send(
        &mut session,
        1,
        ClientCommand::ConfigureBotAi {
            model: None,
            device: DevicePreference::Auto,
        },
    );
    assert!(rejected.iter().any(|d| matches!(
        d.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::GameAlreadyStarted)
        }
    )));
    assert_eq!(session.room.bot_ai.as_ref(), Some(&original));
}
