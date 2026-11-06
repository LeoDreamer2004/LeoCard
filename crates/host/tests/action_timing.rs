use ed25519_dalek::{Signer, SigningKey};
use leocard_blackjack::{BlackjackAction, BlackjackRuleSet};
use leocard_game_common::{ActionTimeStats, PlayerId};
use leocard_host::{ConnectionId, Delivery, GameSetup, HostSession};
use leocard_protocol::{
    ActionTimingSnapshot, BlackjackCommand, ClientCommand, ClientMessage, GameKind, GameRules,
    GameSnapshot, JoinRequest, PlayerGameProfiles, PlayerInteractionKind, ProfileId,
    ReconnectToken, RequestId, RoomId, SeatId, ServerEvent,
};
use std::collections::HashMap;
use std::time::Duration;

#[path = "action_timing/games.rs"]
mod games;
#[path = "action_timing/presence.rs"]
mod presence;
#[path = "action_timing/texas.rs"]
mod texas;

const ROOM: RoomId = RoomId(411);

struct Table {
    session: HostSession,
    requests: HashMap<u64, u64>,
}

impl Table {
    fn new() -> Self {
        let rules = BlackjackRuleSet {
            turn_seconds: 15,
            ..Default::default()
        };
        Self::new_for(rules.into())
    }

    fn new_for(rules: GameRules) -> Self {
        fastrand::seed(411);
        let mut table = Self {
            session: HostSession::new(ROOM, GameSetup::shuffled(52300, rules)).unwrap(),
            requests: HashMap::new(),
        };
        for connection in 1..=4 {
            let key = SigningKey::from_bytes(&[connection as u8; 32]);
            let mut join = JoinRequest {
                name: format!("玩家{connection}"),
                reconnect_token: ReconnectToken(connection),
                profile_id: ProfileId(key.verifying_key().to_bytes()),
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
                identity_signature: Vec::new(),
                avatar_png: None,
            };
            join.identity_signature = key.sign(&join.identity_payload(ROOM)).to_bytes().to_vec();
            table.send(connection, ClientCommand::join(join));
            table.send(
                connection,
                ClientCommand::SelectSeat {
                    seat: SeatId(connection as u8 - 1),
                },
            );
            table.send(connection, ClientCommand::SetReady { ready: true });
        }
        let started = table.send(1, ClientCommand::StartGame);
        assert!(
            started
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::GameStarted { .. })),
            "{started:?}"
        );
        table
    }

    fn send(&mut self, connection: u64, command: ClientCommand) -> Vec<Delivery> {
        let request = self.requests.entry(connection).or_default();
        *request += 1;
        self.session.handle(
            ConnectionId(connection),
            ClientMessage::new(ROOM, RequestId(*request), command),
        )
    }

    fn view(&mut self) -> (GameSnapshot, ActionTimingSnapshot) {
        self.view_for(1)
    }

    fn view_for(&mut self, connection: u64) -> (GameSnapshot, ActionTimingSnapshot) {
        self.send(connection, ClientCommand::RequestSnapshot)
            .into_iter()
            .find_map(|delivery| match delivery.message.event {
                ServerEvent::GameSnapshot(snapshot) => {
                    Some((snapshot, delivery.message.action_timing.unwrap()))
                }
                _ => None,
            })
            .unwrap()
    }
}

trait ProfileTiming {
    fn action_time(&self, game: GameKind) -> ActionTimeStats;
}

impl ProfileTiming for PlayerGameProfiles {
    fn action_time(&self, game: GameKind) -> ActionTimeStats {
        match game {
            GameKind::QiGui523 => self.qigui523.as_ref().map(|stats| stats.action_time),
            GameKind::Texas => self.texas.as_ref().map(|stats| stats.action_time),
            GameKind::Shengji => self.shengji.as_ref().map(|stats| stats.action_time),
            GameKind::Uno => self.uno.as_ref().map(|stats| stats.action_time),
            GameKind::Mahjong => self.mahjong.as_ref().map(|stats| stats.action_time),
            GameKind::McGuess => self.mc_guess.as_ref().map(|stats| stats.action_time),
            GameKind::Blackjack => self.blackjack.as_ref().map(|stats| stats.action_time),
            GameKind::Doudizhu => self.doudizhu.as_ref().map(|stats| stats.action_time),
            GameKind::Guandan => self.guandan.as_ref().map(|stats| stats.action_time),
            GameKind::MahjongLink => self.mahjong_link.as_ref().map(|stats| stats.action_time),
        }
        .unwrap_or_default()
    }
}

fn bet(amount: u32) -> ClientCommand {
    ClientCommand::Game(
        BlackjackCommand::Act {
            action: BlackjackAction::Bet { amount },
        }
        .into(),
    )
}

#[test]
fn only_an_accepted_human_decision_records_elapsed_time_and_replays_do_not_duplicate_it() {
    let mut table = Table::new();
    let (_, initial) = table.view();
    assert_eq!(initial.game, GameKind::Blackjack);
    assert_eq!(initial.clocks.len(), 1);
    assert_eq!(initial.clocks[0].elapsed_millis, 0);
    let player = initial.clocks[0].player;
    let connection = u64::from(player.0) + 1;
    table.session.advance_time(Duration::from_millis(1200));
    let rejected = table.send(connection, bet(0));
    assert!(
        rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    assert_eq!(table.view().1.clocks[0].elapsed_millis, 1200);
    table.session.advance_time(Duration::from_millis(300));
    table.send(
        connection,
        ClientCommand::Interact {
            target: PlayerId((player.0 + 1) % 4),
            kind: PlayerInteractionKind::Egg,
        },
    );
    assert_eq!(table.view().1.clocks[0].elapsed_millis, 1500);
    table.session.advance_time(Duration::from_millis(500));
    let accepted = table.send(connection, bet(2));
    let accepted_request = table.requests[&connection];
    assert!(
        !accepted
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    let (snapshot, next) = table.view();
    let stats = snapshot
        .player(player)
        .unwrap()
        .game_profiles
        .action_time(GameKind::Blackjack);
    assert_eq!((stats.total_millis, stats.completed_actions), (2000, 1));
    assert_eq!(
        snapshot
            .player(player)
            .unwrap()
            .game_profiles
            .action_time(GameKind::Texas)
            .completed_actions,
        0
    );
    assert_eq!(next.clocks[0].elapsed_millis, 0);
    let replay = table.session.handle(
        ConnectionId(connection),
        ClientMessage::new(ROOM, RequestId(accepted_request), bet(2)),
    );
    assert!(
        replay
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    assert_eq!(
        table
            .view()
            .0
            .player(player)
            .unwrap()
            .game_profiles
            .action_time(GameKind::Blackjack),
        stats
    );
}

#[test]
fn auto_play_and_timeout_do_not_count_as_human_thinking_samples() {
    for auto_play in [false, true] {
        let mut table = Table::new();
        let player = table.view().1.clocks[0].player;
        if auto_play {
            table.send(
                u64::from(player.0) + 1,
                ClientCommand::Game(BlackjackCommand::SetAutoPlay { enabled: true }.into()),
            );
        }
        table
            .session
            .advance_time(Duration::from_secs(if auto_play { 1 } else { 15 }));
        let (snapshot, _) = table.view();
        assert_eq!(
            snapshot
                .player(player)
                .unwrap()
                .game_profiles
                .action_time(GameKind::Blackjack),
            Default::default()
        );
    }
}
