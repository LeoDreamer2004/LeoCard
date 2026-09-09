use crate::{
    ConnectionId, Delivery, HostError, MahjongSession, QiGui523Session, ShengjiSession,
    TexasHoldemSession, UnoSession,
};
use leocard_mahjong::{MahjongRuleSet, MahjongTile};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameKind, GameRules, PROTOCOL_VERSION, Revision, RoomId,
    ServerEvent, ServerMessage,
};
use leocard_qigui523::{QiGuiCard, QiGuiRuleSet, build_deck};
use leocard_shengji::{ShengjiCard, ShengjiRuleSet, build_deck_for};
use leocard_texas_holdem::{TexasHoldemCard, TexasHoldemRuleSet};
use leocard_uno::{UnoCard, UnoRuleSet, build_deck_for_rules};
use std::time::Duration;

/// 创建统一房主会话所需的具体游戏后端配置。
#[derive(Clone, Debug)]
pub enum GameSetup {
    QiGui523 {
        host_port: u16,
        rules: QiGuiRuleSet,
        shuffled_deck: Vec<QiGuiCard>,
    },
    TexasHoldem {
        host_port: u16,
        rules: TexasHoldemRuleSet,
        shuffled_deck: Vec<TexasHoldemCard>,
    },
    Shengji {
        host_port: u16,
        rules: ShengjiRuleSet,
        shuffled_deck: Vec<ShengjiCard>,
    },
    Uno {
        host_port: u16,
        rules: UnoRuleSet,
        shuffled_deck: Vec<UnoCard>,
    },
    Mahjong {
        host_port: u16,
        rules: MahjongRuleSet,
        shuffled_deck: Vec<MahjongTile>,
    },
}

impl GameSetup {
    pub fn shuffled(host_port: u16, rules: GameRules) -> Self {
        match rules {
            GameRules::QiGui523(rules) => {
                let mut shuffled_deck = build_deck(rules.deck_count);
                fastrand::shuffle(&mut shuffled_deck);
                Self::QiGui523 {
                    host_port,
                    rules,
                    shuffled_deck,
                }
            }
            GameRules::TexasHoldem(rules) => {
                let mut shuffled_deck = leocard_texas_holdem::build_deck(rules.short_deck);
                fastrand::shuffle(&mut shuffled_deck);
                Self::TexasHoldem {
                    host_port,
                    rules,
                    shuffled_deck,
                }
            }
            GameRules::Shengji(rules) => {
                let mut shuffled_deck = build_deck_for(rules.deck_count);
                fastrand::shuffle(&mut shuffled_deck);
                Self::Shengji {
                    host_port,
                    rules,
                    shuffled_deck,
                }
            }
            GameRules::Uno(rules) => {
                let mut shuffled_deck = build_deck_for_rules(rules);
                fastrand::shuffle(&mut shuffled_deck);
                Self::Uno {
                    host_port,
                    rules,
                    shuffled_deck,
                }
            }
            GameRules::Mahjong(rules) => {
                let mut shuffled_deck = leocard_mahjong::build_deck();
                fastrand::shuffle(&mut shuffled_deck);
                Self::Mahjong {
                    host_port,
                    rules,
                    shuffled_deck,
                }
            }
        }
    }

    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523 { .. } => GameKind::QiGui523,
            Self::TexasHoldem { .. } => GameKind::TexasHoldem,
            Self::Shengji { .. } => GameKind::Shengji,
            Self::Uno { .. } => GameKind::Uno,
            Self::Mahjong { .. } => GameKind::Mahjong,
        }
    }
}

/// TCP 层面对的统一游戏会话路由。
#[derive(Clone, Debug)]
pub enum HostSession {
    QiGui523(Box<QiGui523Session>),
    TexasHoldem(Box<TexasHoldemSession>),
    Shengji(Box<ShengjiSession>),
    Uno(Box<UnoSession>),
    Mahjong(Box<MahjongSession>),
}

trait HostBackend {
    fn game_kind(&self) -> GameKind;
    fn room_id(&self) -> RoomId;
    fn revision(&self) -> Revision;
    fn is_current_connection(&self, connection: ConnectionId) -> bool;
    fn is_closed(&self) -> bool;
    fn heartbeat(&self) -> Vec<Delivery>;
    fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery>;
    fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery>;
    fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery>;
}

macro_rules! impl_host_backend {
    ($session:ty, $kind:ident) => {
        impl HostBackend for $session {
            fn game_kind(&self) -> GameKind {
                GameKind::$kind
            }

            fn room_id(&self) -> RoomId {
                <$session>::room_id(self)
            }

            fn revision(&self) -> Revision {
                <$session>::revision(self)
            }

            fn is_current_connection(&self, connection: ConnectionId) -> bool {
                <$session>::is_current_connection(self, connection)
            }

            fn is_closed(&self) -> bool {
                <$session>::is_closed(self)
            }

            fn heartbeat(&self) -> Vec<Delivery> {
                <$session>::heartbeat(self)
            }

            fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
                <$session>::advance_time(self, elapsed)
            }

            fn handle(
                &mut self,
                connection: ConnectionId,
                message: ClientMessage,
            ) -> Vec<Delivery> {
                <$session>::handle(self, connection, message)
            }

            fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
                <$session>::disconnect(self, connection)
            }
        }
    };
}

impl_host_backend!(QiGui523Session, QiGui523);
impl_host_backend!(TexasHoldemSession, TexasHoldem);
impl_host_backend!(ShengjiSession, Shengji);
impl_host_backend!(UnoSession, Uno);
impl_host_backend!(MahjongSession, Mahjong);

impl HostSession {
    pub fn new(room_id: RoomId, setup: GameSetup) -> Result<Self, HostError> {
        match setup {
            GameSetup::QiGui523 {
                host_port,
                rules,
                shuffled_deck,
            } => QiGui523Session::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Box::new)
                .map(Self::QiGui523),
            GameSetup::TexasHoldem {
                host_port,
                rules,
                shuffled_deck,
            } => TexasHoldemSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Box::new)
                .map(Self::TexasHoldem),
            GameSetup::Shengji {
                host_port,
                rules,
                shuffled_deck,
            } => ShengjiSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Box::new)
                .map(Self::Shengji),
            GameSetup::Uno {
                host_port,
                rules,
                shuffled_deck,
            } => UnoSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Box::new)
                .map(Self::Uno),
            GameSetup::Mahjong {
                host_port,
                rules,
                shuffled_deck,
            } => MahjongSession::new_with_host_port(room_id, host_port, rules, shuffled_deck)
                .map(Box::new)
                .map(Self::Mahjong),
        }
    }

    fn backend(&self) -> &dyn HostBackend {
        match self {
            Self::QiGui523(session) => session.as_ref(),
            Self::TexasHoldem(session) => session.as_ref(),
            Self::Shengji(session) => session.as_ref(),
            Self::Uno(session) => session.as_ref(),
            Self::Mahjong(session) => session.as_ref(),
        }
    }

    fn backend_mut(&mut self) -> &mut dyn HostBackend {
        match self {
            Self::QiGui523(session) => session.as_mut(),
            Self::TexasHoldem(session) => session.as_mut(),
            Self::Shengji(session) => session.as_mut(),
            Self::Uno(session) => session.as_mut(),
            Self::Mahjong(session) => session.as_mut(),
        }
    }

    pub fn game_kind(&self) -> GameKind {
        self.backend().game_kind()
    }

    pub fn room_id(&self) -> RoomId {
        self.backend().room_id()
    }

    pub fn revision(&self) -> Revision {
        self.backend().revision()
    }

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        self.backend().is_current_connection(connection)
    }

    pub fn is_closed(&self) -> bool {
        self.backend().is_closed()
    }

    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.backend().heartbeat()
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        self.backend_mut().advance_time(elapsed)
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        if matches!(&message.command, ClientCommand::Ping) {
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
        self.backend_mut().handle(connection, message)
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        self.backend_mut().disconnect(connection)
    }
}
