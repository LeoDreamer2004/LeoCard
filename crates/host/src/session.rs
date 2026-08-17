use std::time::Duration;

use leocard_protocol::{
    ClientMessage, GameKind, PROTOCOL_VERSION, Revision, RoomId, ServerEvent, ServerMessage,
};
use leocard_qigui523::{Card, RuleSet};

use crate::{
    ConnectionId, Delivery, HostError, QiGui523Session, ShengjiSession, TexasHoldemSession,
    UnoSession,
};

/// 创建统一房主会话所需的具体游戏后端配置。
#[derive(Clone, Debug)]
pub enum GameSetup {
    QiGui523 {
        host_port: u16,
        rules: RuleSet,
        shuffled_deck: Vec<Card>,
    },
    TexasHoldem {
        host_port: u16,
        rules: leocard_texas_holdem::RuleSet,
        shuffled_deck: Vec<leocard_texas_holdem::Card>,
    },
    Shengji {
        host_port: u16,
        rules: leocard_shengji::RuleSet,
        shuffled_deck: Vec<leocard_shengji::Card>,
    },
    Uno {
        host_port: u16,
        rules: leocard_uno::RuleSet,
        shuffled_deck: Vec<leocard_uno::Card>,
    },
}

impl GameSetup {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523 { .. } => GameKind::QiGui523,
            Self::TexasHoldem { .. } => GameKind::TexasHoldem,
            Self::Shengji { .. } => GameKind::Shengji,
            Self::Uno { .. } => GameKind::Uno,
        }
    }
}

/// TCP 层面对的统一游戏会话路由。
#[derive(Clone, Debug)]
pub enum HostSession {
    QiGui523(QiGui523Session),
    TexasHoldem(TexasHoldemSession),
    Shengji(ShengjiSession),
    Uno(UnoSession),
}

impl HostSession {
    pub fn new(room_id: RoomId, setup: GameSetup) -> Result<Self, HostError> {
        match setup {
            GameSetup::QiGui523 {
                host_port,
                rules,
                shuffled_deck,
            } => QiGui523Session::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Self::QiGui523),
            GameSetup::TexasHoldem {
                host_port,
                rules,
                shuffled_deck,
            } => TexasHoldemSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Self::TexasHoldem),
            GameSetup::Shengji {
                host_port,
                rules,
                shuffled_deck,
            } => ShengjiSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Self::Shengji),
            GameSetup::Uno {
                host_port,
                rules,
                shuffled_deck,
            } => UnoSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Self::Uno),
        }
    }

    pub fn qigui523(
        room_id: RoomId,
        host_port: u16,
        rules: RuleSet,
        shuffled_deck: Vec<Card>,
    ) -> Result<Self, HostError> {
        Self::new(
            room_id,
            GameSetup::QiGui523 {
                host_port,
                rules,
                shuffled_deck,
            },
        )
    }

    pub fn texas_holdem(
        room_id: RoomId,
        host_port: u16,
        rules: leocard_texas_holdem::RuleSet,
        shuffled_deck: Vec<leocard_texas_holdem::Card>,
    ) -> Result<Self, HostError> {
        Self::new(
            room_id,
            GameSetup::TexasHoldem {
                host_port,
                rules,
                shuffled_deck,
            },
        )
    }

    pub fn shengji(
        room_id: RoomId,
        host_port: u16,
        rules: leocard_shengji::RuleSet,
        shuffled_deck: Vec<leocard_shengji::Card>,
    ) -> Result<Self, HostError> {
        Self::new(
            room_id,
            GameSetup::Shengji {
                host_port,
                rules,
                shuffled_deck,
            },
        )
    }

    pub fn uno(
        room_id: RoomId,
        host_port: u16,
        rules: leocard_uno::RuleSet,
        shuffled_deck: Vec<leocard_uno::Card>,
    ) -> Result<Self, HostError> {
        Self::new(
            room_id,
            GameSetup::Uno {
                host_port,
                rules,
                shuffled_deck,
            },
        )
    }

    pub const fn game_kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
        }
    }

    pub const fn qigui523_backend(&self) -> Option<&QiGui523Session> {
        match self {
            Self::QiGui523(session) => Some(session),
            Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub const fn texas_holdem_backend(&self) -> Option<&TexasHoldemSession> {
        match self {
            Self::TexasHoldem(session) => Some(session),
            Self::QiGui523(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub const fn shengji_backend(&self) -> Option<&ShengjiSession> {
        match self {
            Self::Shengji(session) => Some(session),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Uno(_) => None,
        }
    }

    pub const fn uno_backend(&self) -> Option<&UnoSession> {
        match self {
            Self::Uno(session) => Some(session),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) => None,
        }
    }

    pub fn room_id(&self) -> RoomId {
        match self {
            Self::QiGui523(session) => session.room_id(),
            Self::TexasHoldem(session) => session.room_id(),
            Self::Shengji(session) => session.room_id(),
            Self::Uno(session) => session.room_id(),
        }
    }

    pub fn revision(&self) -> Revision {
        match self {
            Self::QiGui523(session) => session.revision(),
            Self::TexasHoldem(session) => session.revision(),
            Self::Shengji(session) => session.revision(),
            Self::Uno(session) => session.revision(),
        }
    }

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        match self {
            Self::QiGui523(session) => session.is_current_connection(connection),
            Self::TexasHoldem(session) => session.is_current_connection(connection),
            Self::Shengji(session) => session.is_current_connection(connection),
            Self::Uno(session) => session.is_current_connection(connection),
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::QiGui523(session) => session.is_closed(),
            Self::TexasHoldem(session) => session.is_closed(),
            Self::Shengji(session) => session.is_closed(),
            Self::Uno(session) => session.is_closed(),
        }
    }

    pub fn heartbeat(&self) -> Vec<Delivery> {
        match self {
            Self::QiGui523(session) => session.heartbeat(),
            Self::TexasHoldem(session) => session.heartbeat(),
            Self::Shengji(session) => session.heartbeat(),
            Self::Uno(session) => session.heartbeat(),
        }
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        match self {
            Self::QiGui523(session) => session.advance_time(elapsed),
            Self::TexasHoldem(session) => session.advance_time(elapsed),
            Self::Shengji(session) => session.advance_time(elapsed),
            Self::Uno(session) => session.advance_time(elapsed),
        }
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        if matches!(&message.command, leocard_protocol::ClientCommand::Ping) {
            return vec![Delivery {
                recipient: connection,
                message: ServerMessage {
                    protocol_version: PROTOCOL_VERSION,
                    room_id: self.room_id(),
                    revision: self.revision(),
                    in_reply_to: None,
                    event: ServerEvent::Heartbeat,
                },
            }];
        }
        match self {
            Self::QiGui523(session) => session.handle(connection, message),
            Self::TexasHoldem(session) => session.handle(connection, message),
            Self::Shengji(session) => session.handle(connection, message),
            Self::Uno(session) => session.handle(connection, message),
        }
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        match self {
            Self::QiGui523(session) => session.disconnect(connection),
            Self::TexasHoldem(session) => session.disconnect(connection),
            Self::Shengji(session) => session.disconnect(connection),
            Self::Uno(session) => session.disconnect(connection),
        }
    }
}
