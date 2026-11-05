use super::AchievementSession;
use super::processing::process_action_record;
use crate::app::runtime::sync_local_profile;
use bevy::prelude::*;
use leocard_client::{
    ClientModel, LocalPlayerProfile, PlayerAchievements, PlayerIdentity, PlayerRatingProfile,
};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ACHIEVEMENT_REGISTRY, ActionClock, ActionTimingSnapshot, GameActionTimes, GameKind, GameRules,
    GameViolation, LobbyPlayerView, LobbySnapshot, MatchId, PROTOCOL_VERSION, PlayerGameProfiles,
    PlayerPublicInfo, ProfileId, RejectReason, Revision, RoomId, ServerEvent, ServerMessage,
    UnoViolation,
};
use std::collections::HashSet;

const ID: &str = "leocard:personal/slow_action";

fn profiles(millis: &[u64]) -> PlayerGameProfiles {
    let mut action_times = GameActionTimes::default();
    for millis in millis {
        action_times.record(GameKind::Uno, *millis);
    }
    PlayerGameProfiles {
        action_times,
        ..Default::default()
    }
}

fn app(games: PlayerGameProfiles) -> App {
    let identity = PlayerIdentity::generate().unwrap();
    let mut app = App::new();
    app.insert_resource(PlayerAchievements::new(identity.profile_id()))
        .insert_resource(LocalPlayerProfile {
            identity,
            rating: PlayerRatingProfile {
                reference_points: 0,
                completed_games: 0,
                applied_matches: HashSet::new(),
                last_change: None,
            },
            game_profiles: games,
        })
        .insert_resource(AchievementSession {
            enabled: true,
            ..Default::default()
        })
        .add_systems(Update, process_action_record);
    app
}

fn message(event: ServerEvent, revision: u64) -> ServerMessage {
    ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(1),
        revision: Revision(revision),
        in_reply_to: None,
        game_context: None,
        action_timing: None,
        event,
    }
}

fn model() -> ClientModel {
    let mut model = ClientModel::new(RoomId(1));
    assert!(model.apply(message(ServerEvent::Joined { you: PlayerId(0) }, 1)));
    model
}

fn snapshot(own: PlayerGameProfiles, other: PlayerGameProfiles, revision: u64) -> ServerMessage {
    message(
        ServerEvent::LobbySnapshot(LobbySnapshot {
            game: GameKind::Uno,
            rules: GameRules::Uno(Default::default()),
            host_port: 52300,
            host: Some(PlayerId(0)),
            players: [own, other]
                .into_iter()
                .enumerate()
                .rev()
                .map(|(id, game_profiles)| LobbyPlayerView {
                    player: PlayerPublicInfo {
                        id: PlayerId(id as u8),
                        profile_id: ProfileId([id as u8; 32]),
                        name: format!("player{id}"),
                        avatar: None,
                        reference_points: 0,
                        completed_games: 0,
                        game_profiles,
                    },
                    seat: None,
                    ready: false,
                    connected: true,
                })
                .collect(),
            entrance: None,
        }),
        revision,
    )
}

fn synchronize(app: &mut App, model: &ClientModel) {
    sync_local_profile(
        model,
        &mut app.world_mut().resource_mut::<LocalPlayerProfile>(),
    );
    app.update();
}

fn achieved(app: &App) -> bool {
    app.world()
        .resource::<PlayerAchievements>()
        .achieved(ACHIEVEMENT_REGISTRY.by_id(ID).unwrap())
}

#[test]
fn slow_action_accepted_snapshot_bridge_uses_inclusive_single_action_maximum() {
    for (millis, expected) in [
        (vec![59_999], false),
        (vec![60_000], true),
        (vec![60_001], true),
        (vec![30_000, 30_001, 40_000], false),
    ] {
        let mut app = app(Default::default());
        let mut model = model();
        assert!(model.apply(snapshot(profiles(&millis), Default::default(), 2)));
        synchronize(&mut app, &model);
        assert_eq!(achieved(&app), expected, "{millis:?}");
    }
}

#[test]
fn slow_action_ignores_other_players_and_rejected_snapshots() {
    let mut app = app(Default::default());
    let mut model = model();
    assert!(model.apply(snapshot(profiles(&[59_999]), profiles(&[90_000]), 3)));
    synchronize(&mut app, &model);
    assert!(!achieved(&app));
    let valid = snapshot(profiles(&[60_000]), Default::default(), 4);
    let mut wrong_room = valid.clone();
    wrong_room.room_id = RoomId(2);
    let mut wrong_version = valid.clone();
    wrong_version.protocol_version = PROTOCOL_VERSION.wrapping_add(1);
    let mut stale = valid;
    stale.revision = Revision(2);
    for rejected in [wrong_room, wrong_version, stale] {
        assert!(!model.apply(rejected));
        synchronize(&mut app, &model);
        assert!(!achieved(&app));
    }
}

#[test]
fn slow_action_running_clock_and_failed_operation_do_not_complete_an_action() {
    let mut app = app(Default::default());
    let mut model = model();
    assert!(model.apply(snapshot(profiles(&[59_999]), Default::default(), 2)));
    let mut clock = message(ServerEvent::Heartbeat, 3);
    clock.action_timing = Some(ActionTimingSnapshot {
        game: GameKind::Uno,
        match_id: MatchId([1; 16]),
        clocks: vec![ActionClock {
            player: PlayerId(0),
            elapsed_millis: 90_000,
        }],
    });
    assert!(model.apply(clock));
    synchronize(&mut app, &model);
    assert!(!achieved(&app));
    // A failed operation leaves the accepted profile maximum unchanged.
    assert!(model.apply(message(
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::Uno(UnoViolation::NotPlayersTurn)),
        },
        4,
    )));
    synchronize(&mut app, &model);
    assert!(!achieved(&app));
}

#[test]
fn slow_action_retained_record_unlocks_without_callback_and_ledger_deduplicates_reconnect() {
    let mut app = app(profiles(&[60_000]));
    app.update();
    assert!(achieved(&app));
    {
        let mut session = app.world_mut().resource_mut::<AchievementSession>();
        assert_eq!(session.pending.len(), 1);
        assert_eq!(session.pending[0].0.id, ID);
        assert!(session.dirty);
        session.pending.clear();
        session.dirty = false;
    }
    let mut model = model();
    // Reobserve the retained record after a different snapshot, bypassing the cache.
    for (revision, millis) in [(2, 59_999), (3, 60_000), (4, 60_000)] {
        assert!(model.apply(snapshot(profiles(&[millis]), Default::default(), revision)));
        synchronize(&mut app, &model);
        let session = app.world().resource::<AchievementSession>();
        assert!(session.pending.is_empty());
        assert!(!session.dirty);
    }
    assert_eq!(
        app.world().resource::<PlayerAchievements>().counts().bronze,
        1
    );
}
