use crate::{
    AvatarId, ChatContent, ChatMessage, GameCommand, GameEvent, GameKind, GameRules,
    MahjongSnapshot, PROTOCOL_VERSION, PlayerGameProfiles, PlayerId, PlayerInteraction,
    PlayerInteractionKind, ProfileId, QiGui523Snapshot, ReconnectToken, RejectReason, RequestId,
    Revision, RoomId, SeatId, ShengjiSnapshot, TexasHoldemSnapshot, UnoSnapshot,
};
use leocard_mahjong::MahjongRuleSet;
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientMessage {
    pub protocol_version: u16,
    pub room_id: RoomId,
    pub request_id: RequestId,
    pub command: ClientCommand,
}

impl ClientMessage {
    pub fn new(room_id: RoomId, request_id: RequestId, command: ClientCommand) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            room_id,
            request_id,
            command,
        }
    }
}

/// 构造加入房间身份签名的规范字节串。名称必须传入去除首尾空白后的形式。
pub fn join_identity_payload(
    room_id: RoomId,
    reconnect_token: ReconnectToken,
    name: &str,
    reference_points: i32,
    completed_games: u32,
    game_profiles: &PlayerGameProfiles,
) -> Vec<u8> {
    const DOMAIN: &[u8] = b"leocard/join-identity/v2";
    let name = name.as_bytes();
    let profiles = postcard::to_allocvec(game_profiles)
        .expect("player game profiles always have a canonical postcard encoding");
    let mut payload =
        Vec::with_capacity(DOMAIN.len() + 8 + 8 + 4 + 4 + 4 + name.len() + 4 + profiles.len());
    payload.extend_from_slice(DOMAIN);
    payload.extend_from_slice(&room_id.0.to_be_bytes());
    payload.extend_from_slice(&reconnect_token.0.to_be_bytes());
    payload.extend_from_slice(&reference_points.to_be_bytes());
    payload.extend_from_slice(&completed_games.to_be_bytes());
    payload.extend_from_slice(&(name.len() as u32).to_be_bytes());
    payload.extend_from_slice(name);
    payload.extend_from_slice(&(profiles.len() as u32).to_be_bytes());
    payload.extend_from_slice(&profiles);
    payload
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JoinRequest {
    pub name: String,
    pub reconnect_token: ReconnectToken,
    pub profile_id: ProfileId,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
    pub identity_signature: Vec<u8>,
}

impl JoinRequest {
    pub fn identity_payload(&self, room_id: RoomId) -> Vec<u8> {
        join_identity_payload(
            room_id,
            self.reconnect_token,
            self.name.trim(),
            self.reference_points,
            self.completed_games,
            &self.game_profiles,
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ClientCommand {
    Join(Box<JoinRequest>),
    SetAvatar {
        png: Vec<u8>,
    },
    SelectSeat {
        seat: SeatId,
    },
    /// 开发者模式下由房主在指定座位添加或移除默认机器人。
    ConfigureBotSeat {
        seat: SeatId,
        occupied: bool,
    },
    SetReady {
        ready: bool,
    },
    Game(GameCommand),
    StartGame,
    ReturnToLobby,
    PlayAgain,
    LeaveRoom,
    CloseRoom,
    Interact {
        target: PlayerId,
        kind: PlayerInteractionKind,
    },
    Chat {
        content: ChatContent,
    },
    /// 传输层存活探测，不参与房间请求序号和游戏状态。
    Ping,
    RequestSnapshot,
}

impl ClientCommand {
    pub fn join(request: JoinRequest) -> Self {
        Self::Join(Box::new(request))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServerMessage {
    pub protocol_version: u16,
    pub room_id: RoomId,
    pub revision: Revision,
    /// 对主动请求者设置；其他客户端收到同一次广播时为 `None`。
    pub in_reply_to: Option<RequestId>,
    pub event: ServerEvent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// 快照是高频协议主体；保持内联可避免为单个大游戏改变既有线协议形状。
#[allow(clippy::large_enum_variant)]
pub enum ServerEvent {
    Heartbeat,
    Joined { you: PlayerId },
    AvatarData { id: AvatarId, png: Vec<u8> },
    LobbySnapshot(LobbySnapshot),
    GameSnapshot(GameSnapshot),
    GameEvent(GameEvent),
    PlayerInteraction(PlayerInteraction),
    ChatMessage(ChatMessage),
    PlayerLeft { name: String },
    LeftRoom,
    RoomClosed,
    Rejected { reason: RejectReason },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LobbySnapshot {
    pub game: GameKind,
    pub rules: GameRules,
    /// 房主实际监听的 TCP 端口；由房主广播，不能从客户端使用的映射地址推断。
    pub host_port: u16,
    pub host: Option<PlayerId>,
    pub players: Vec<LobbyPlayer>,
}

impl LobbySnapshot {
    pub const fn qigui523_rules(&self) -> Option<&QiGuiRuleSet> {
        self.rules.qigui523()
    }

    pub const fn texas_holdem_rules(&self) -> Option<&TexasHoldemRuleSet> {
        self.rules.texas_holdem()
    }

    pub const fn shengji_rules(&self) -> Option<&ShengjiRuleSet> {
        self.rules.shengji()
    }

    pub const fn uno_rules(&self) -> Option<&UnoRuleSet> {
        self.rules.uno()
    }

    pub const fn mahjong_rules(&self) -> Option<&MahjongRuleSet> {
        self.rules.mahjong()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LobbyPlayer {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: Option<SeatId>,
    pub ready: bool,
    pub connected: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameSnapshot {
    QiGui523(QiGui523Snapshot),
    TexasHoldem(TexasHoldemSnapshot),
    Shengji(ShengjiSnapshot),
    Uno(UnoSnapshot),
    Mahjong(MahjongSnapshot),
}

impl GameSnapshot {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
            Self::Mahjong(_) => GameKind::Mahjong,
        }
    }

    pub const fn qigui523(&self) -> Option<&QiGui523Snapshot> {
        match self {
            Self::QiGui523(snapshot) => Some(snapshot),
            Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) | Self::Mahjong(_) => None,
        }
    }

    pub fn into_qigui523(self) -> Option<QiGui523Snapshot> {
        match self {
            Self::QiGui523(snapshot) => Some(snapshot),
            Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) | Self::Mahjong(_) => None,
        }
    }

    pub const fn texas_holdem(&self) -> Option<&TexasHoldemSnapshot> {
        match self {
            Self::TexasHoldem(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::Shengji(_) | Self::Uno(_) | Self::Mahjong(_) => None,
        }
    }

    pub fn into_texas_holdem(self) -> Option<TexasHoldemSnapshot> {
        match self {
            Self::TexasHoldem(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::Shengji(_) | Self::Uno(_) | Self::Mahjong(_) => None,
        }
    }

    pub const fn shengji(&self) -> Option<&ShengjiSnapshot> {
        match self {
            Self::Shengji(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Uno(_) | Self::Mahjong(_) => None,
        }
    }

    pub fn into_shengji(self) -> Option<ShengjiSnapshot> {
        match self {
            Self::Shengji(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Uno(_) | Self::Mahjong(_) => None,
        }
    }

    pub const fn uno(&self) -> Option<&UnoSnapshot> {
        match self {
            Self::Uno(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) | Self::Mahjong(_) => None,
        }
    }

    pub fn into_uno(self) -> Option<UnoSnapshot> {
        match self {
            Self::Uno(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) | Self::Mahjong(_) => None,
        }
    }

    pub const fn mahjong(&self) -> Option<&MahjongSnapshot> {
        match self {
            Self::Mahjong(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub fn into_mahjong(self) -> Option<MahjongSnapshot> {
        match self {
            Self::Mahjong(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }
}
