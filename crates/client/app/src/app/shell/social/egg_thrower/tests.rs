use super::AutoEggThrower;
use super::state::EggBurst;
use super::systems::tick_auto_egg_thrower;
use crate::app::runtime::{ClientResource, PageErrorState};
use bevy::prelude::*;
use leocard_client::{
    ClientModel, LocalPlayerConnection, LocalPlayerProfile, PlayerEconomy, PlayerIdentity,
    PlayerRatingProfile, TcpGameClient,
};
use leocard_game_common::PlayerId;
use leocard_protocol::{PlayerInteractionKind, ProfileId, RoomId};
use leocard_qigui523::QiGuiRuleSet;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

fn burst() -> EggBurst {
    EggBurst {
        room: RoomId(1),
        source: PlayerId(0),
        target: PlayerId(1),
        target_profile: ProfileId([2; 32]),
        last_sent_at: Duration::from_secs(20),
        sent: 0,
    }
}

#[test]
fn eggs_are_spaced_at_one_hundred_milliseconds_and_stop_at_one_hundred() {
    let mut burst = burst();
    let start = burst.last_sent_at;
    assert!(!burst.ready(Duration::ZERO));
    for egg in 1..=100 {
        let now = start + Duration::from_millis(egg * 100);
        assert!(!burst.ready(now - Duration::from_millis(1)));
        assert!(burst.ready(now));
        burst.record_sent(now);
        assert!(!burst.ready(now));
        assert_eq!(burst.finished(), egg == 100);
    }
    assert!(!burst.ready(start + Duration::from_secs(60)));
}

#[test]
fn delayed_frames_do_not_dump_catch_up_eggs() {
    let mut burst = burst();
    let now = burst.last_sent_at + Duration::from_secs(60);
    assert!(burst.ready(now));
    burst.record_sent(now);
    assert_eq!(burst.sent, 1);
    assert!(!burst.ready(now));
    assert!(!burst.ready(now + Duration::from_millis(99)));
    assert!(burst.ready(now + Duration::from_millis(100)));
}

fn wait_for(client: &mut TcpGameClient, ready: impl Fn(&ClientModel) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        client.poll();
        if ready(client.model()) {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!("timed out waiting for local room: {:?}", client.state());
}

#[test]
fn a_real_tcp_target_receives_exactly_one_hundred_host_validated_eggs() {
    let port = TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut host = TcpGameClient::host_with_profile(
        port,
        QiGuiRuleSet::default().into(),
        LocalPlayerConnection::from_profile(
            "砸蛋器房主",
            None,
            &LocalPlayerProfile {
                identity: PlayerIdentity::generate().unwrap(),
                rating: PlayerRatingProfile {
                    reference_points: 0,
                    completed_games: 0,
                    applied_matches: default(),
                    last_change: None,
                },
                game_profiles: default(),
            },
            default(),
            default(),
        ),
    )
    .unwrap();
    wait_for(&mut host, |model| model.lobby().is_some());
    let mut guest = TcpGameClient::join("目标", &format!("127.0.0.1:{port}")).unwrap();
    wait_for(&mut guest, |model| model.lobby().is_some());
    let target = guest.model().you().unwrap();
    wait_for(&mut host, |model| model.player(target).is_some());
    let mut active = EggBurst {
        room: host.model().room_id(),
        source: host.model().you().unwrap(),
        target,
        target_profile: host.model().player(target).unwrap().profile_id,
        last_sent_at: Duration::ZERO,
        sent: 0,
    };
    assert!(active.still_valid(&host));
    active.target_profile = ProfileId([99; 32]);
    assert!(!active.still_valid(&host));
    active.target_profile = host.model().player(target).unwrap().profile_id;
    active.room = RoomId(u64::MAX);
    assert!(!active.still_valid(&host));
    active.room = host.model().room_id();
    active.source = target;
    assert!(!active.still_valid(&host));
    active.source = host.model().you().unwrap();
    let mut unavailable = PlayerEconomy::unavailable(ProfileId([1; 32]), "test wallet".to_owned());
    let mut thrower = AutoEggThrower {
        active: Some(active),
    };
    assert!(
        thrower
            .activate(&host, target, Duration::ZERO, &mut unavailable)
            .unwrap_err()
            .contains("正在连发")
    );
    let mut app = App::new();
    app.insert_resource(Time::<Real>::default())
        .insert_resource(ClientResource(host))
        .insert_resource(PageErrorState::default())
        .insert_resource(thrower)
        .add_systems(Update, tick_auto_egg_thrower);
    let mut received = 0;
    for expected in 1..=100 {
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        app.update();
        let deadline = Instant::now() + Duration::from_secs(5);
        while received < expected && Instant::now() < deadline {
            guest.poll();
            for interaction in guest.take_player_interactions() {
                assert_eq!(interaction.target, target);
                assert_eq!(interaction.kind, PlayerInteractionKind::Egg);
                received += 1;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(received, expected);
    }
    assert!(app.world().resource::<AutoEggThrower>().active.is_none());
    assert!(app.world().resource::<PageErrorState>().error.is_none());
    assert!(guest.model().last_rejection().is_none());

    // A departing target cancels remaining eggs without needing an economy
    // resource: cancellation cannot refund or write any inventory.
    let host = &app.world().resource::<ClientResource>().0;
    let departing = EggBurst {
        room: host.model().room_id(),
        source: host.model().you().unwrap(),
        target,
        target_profile: host.model().player(target).unwrap().profile_id,
        last_sent_at: Duration::from_secs(10),
        sent: 1,
    };
    app.world_mut().resource_mut::<AutoEggThrower>().active = Some(departing);
    drop(guest);
    wait_for(
        &mut app.world_mut().resource_mut::<ClientResource>().0,
        |model| model.player(target).is_none(),
    );
    app.update();
    assert!(app.world().resource::<AutoEggThrower>().active.is_none());
}

#[test]
fn leaving_the_room_cancels_the_burst_instead_of_leaking_it_to_another_session() {
    let mut app = App::new();
    app.insert_resource(Time::<Real>::default())
        .insert_resource(PageErrorState::default())
        .insert_resource(AutoEggThrower {
            active: Some(burst()),
        })
        .add_systems(Update, tick_auto_egg_thrower);
    app.update();
    assert!(app.world().resource::<AutoEggThrower>().active.is_none());
}
