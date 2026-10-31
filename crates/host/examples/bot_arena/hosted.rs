use ed25519_dalek::{Signer, SigningKey};
use leocard_host::{ConnectionId, GameSetup, HostSession};
use leocard_mahjong::{MahjongRuleSet, build_deck as mahjong_deck};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameEvent, JoinRequest, MahjongEvent, PlayerGameProfiles,
    ProfileId, ReconnectToken, RequestId, RoomId, SeatId, ServerEvent, ShengjiEvent, TexasCommand,
    TexasEvent, join_identity_payload,
};
use leocard_shengji::{ShengjiRuleSet, build_deck as shengji_deck};
use leocard_texas_holdem::{TexasAction, TexasRuleSet, build_deck as texas_deck};
use std::{
    thread,
    time::{Duration, Instant},
};

pub fn run() {
    let mut mahjong = mahjong_deck();
    let mut shengji = shengji_deck();
    let mut texas = texas_deck(false);
    let mut random = fastrand::Rng::with_seed(9);
    random.shuffle(&mut mahjong);
    random.shuffle(&mut shengji);
    random.shuffle(&mut texas);
    let setups = [
        GameSetup::Mahjong {
            host_port: 52300,
            rules: MahjongRuleSet::default(),
            shuffled_deck: mahjong,
        },
        GameSetup::Texas {
            host_port: 52300,
            rules: TexasRuleSet {
                player_count: 3,
                ..TexasRuleSet::default()
            },
            shuffled_deck: texas,
        },
        GameSetup::Shengji {
            host_port: 52300,
            rules: ShengjiRuleSet::default(),
            shuffled_deck: shengji,
        },
    ];
    for setup in setups {
        let seats = if matches!(setup, GameSetup::Texas { .. }) {
            3
        } else {
            4
        };
        let room = RoomId(400);
        let mut session = HostSession::new(room, setup).unwrap();
        let mut request = 0;
        let mut send = |session: &mut HostSession, command| {
            request += 1;
            let deliveries = session.handle(
                ConnectionId(1),
                ClientMessage::new(room, RequestId(request), command),
            );
            for delivery in deliveries {
                if let ServerEvent::Rejected { reason, .. } = delivery.message.event {
                    panic!("host command rejected: {reason:?}");
                }
            }
        };
        send(&mut session, join(room));
        send(
            &mut session,
            ClientCommand::SelectSeat {
                seat: SeatId(seats - 1),
            },
        );
        for seat in 0..seats - 1 {
            send(
                &mut session,
                ClientCommand::ConfigureBotSeat {
                    seat: SeatId(seat),
                    occupied: true,
                },
            );
        }
        send(&mut session, ClientCommand::StartGame);
        if seats == 3 {
            // The random dealer can make the human host act before either bot.
            send(
                &mut session,
                ClientCommand::Game(TexasCommand::SetAutoPlay { enabled: true }.into()),
            );
        }
        let start = Instant::now();
        let mut tick_max = Duration::ZERO;
        let mut acted = false;
        while start.elapsed() < Duration::from_secs(12) {
            let tick = Instant::now();
            let deliveries = session.advance_time(Duration::from_secs(1));
            tick_max = tick_max.max(tick.elapsed());
            acted = deliveries
                .iter()
                .any(|delivery| match &delivery.message.event {
                    ServerEvent::GameEvent(GameEvent::Mahjong(
                        MahjongEvent::TileDiscarded { .. } | MahjongEvent::KongDeclared { .. },
                    )) => true,
                    ServerEvent::GameEvent(GameEvent::Texas(TexasEvent::ActionApplied {
                        player,
                        action,
                        ..
                    })) => player.0 != 0 && *action != TexasAction::PostBlind,
                    ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsBuried {
                        ..
                    })) => true,
                    _ => false,
                });
            if acted {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(acted, "background bot failed to return an action");
        println!(
            "hosted {:?}: first bot action in {:?}, largest host tick {:?}",
            session.game_kind(),
            start.elapsed(),
            tick_max
        );
    }
}

fn join(room: RoomId) -> ClientCommand {
    let key = SigningKey::from_bytes(&[17; 32]);
    let name = "模拟房主".to_owned();
    let token = ReconnectToken(1);
    let profiles = PlayerGameProfiles::default();
    let signature = key
        .sign(&join_identity_payload(room, token, &name, 0, 0, &profiles))
        .to_bytes()
        .to_vec();
    ClientCommand::join(JoinRequest {
        name,
        reconnect_token: token,
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: profiles,
        identity_signature: signature,
    })
}
