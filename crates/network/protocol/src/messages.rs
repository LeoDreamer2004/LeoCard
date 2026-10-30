use crate::{
    AchievementAnnouncement, AchievementCounts, AvatarId, BlackjackSnapshot, ChatContent,
    ChatMessage, DoudizhuSnapshot, GameCommand, GameEvent, GameKind, GamePhaseView, GameRules,
    MahjongPhaseView, MahjongSnapshot, MatchId, MinecraftGuessSnapshot, PROTOCOL_VERSION,
    PlayerGameProfiles, PlayerId, PlayerInteraction, PlayerInteractionKind, ProfileId,
    QiGui523Snapshot, ReconnectToken, RejectReason, RequestId, Revision, RoomId, RoomInfo, SeatId,
    ShengjiSnapshot, TexasHoldemPhaseView, TexasHoldemSnapshot, UnoPhaseView, UnoSnapshot,
};
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
    SpectatePlayer {
        player: PlayerId,
    },
    PublishCoins {
        coins: u32,
    },
    /// Aggregate profile synchronization; IDs announce only newly earned awards.
    PublishAchievements {
        counts: AchievementCounts,
        unlocked: Vec<String>,
    },
    Join(Box<JoinRequest>),
    SetAvatar {
        png: Vec<u8>,
    },
    SelectSeat {
        seat: SeatId,
    },
    /// 由房主在准备大厅的指定座位添加或移除机器人。
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
    /// Read-only public metadata; does not require a Join request.
    RequestRoomInfo,
}

impl ClientCommand {
    pub fn join(request: JoinRequest) -> Self {
        Self::Join(Box::new(request))
    }
}

/// Stable identity of a live fact within a match. Assigned by the game adapter.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameEventContext {
    /// The host applies the same eligibility policy to profiles, coins and live achievements.
    pub records_progress: bool,
    pub match_id: MatchId,
    pub hand_index: Option<u32>,
    pub sequence: u128,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServerMessage {
    pub protocol_version: u16,
    pub room_id: RoomId,
    pub revision: Revision,
    /// Present only on live game/session events, never on snapshots.
    pub game_context: Option<GameEventContext>,
    /// 对主动请求者设置；其他客户端收到同一次广播时为 `None`。
    pub in_reply_to: Option<RequestId>,
    pub event: ServerEvent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// 快照是高频协议主体；保持内联可避免为单个大游戏改变既有线协议形状。
#[allow(clippy::large_enum_variant)]
pub enum ServerEvent {
    AchievementUnlocked(AchievementAnnouncement),
    Heartbeat,
    Joined {
        you: PlayerId,
    },
    AvatarData {
        id: AvatarId,
        png: Vec<u8>,
    },
    LobbySnapshot(LobbySnapshot),
    GameSnapshot(GameSnapshot),
    GameEvent(GameEvent),
    PlayerInteraction(PlayerInteraction),
    ChatMessage(ChatMessage),
    PlayerLeft {
        name: String,
    },
    SpectatorJoined {
        name: String,
    },
    LeftRoom,
    RoomClosed,
    Rejected {
        reason: RejectReason,
    },
    GameStarted {
        host: PlayerId,
        game: GameKind,
        match_id: MatchId,
    },
    /// Bootstrap rules without temporarily moving a midgame joiner into the lobby.
    RulesSnapshot(GameRules),
    RoomInfo(Option<RoomInfo>),
    Notice {
        title: String,
        message: String,
    },
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
    MinecraftGuess(MinecraftGuessSnapshot),
    Blackjack(BlackjackSnapshot),
    Doudizhu(DoudizhuSnapshot),
}

macro_rules! game_snapshot_projections {
    ($($borrowed:ident, $owned:ident => $variant:ident($snapshot:ty)),+ $(,)?) => {
        $(
            pub const fn $borrowed(&self) -> Option<&$snapshot> {
                match self {
                    Self::$variant(snapshot) => Some(snapshot),
                    _ => None,
                }
            }

            pub fn $owned(self) -> Option<$snapshot> {
                match self {
                    Self::$variant(snapshot) => Some(snapshot),
                    _ => None,
                }
            }
        )+
    };
}

macro_rules! impl_game_snapshot_from {
    ($snapshot:ty, $variant:ident) => {
        impl From<$snapshot> for GameSnapshot {
            fn from(value: $snapshot) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_game_snapshot_from!(QiGui523Snapshot, QiGui523);
impl_game_snapshot_from!(TexasHoldemSnapshot, TexasHoldem);
impl_game_snapshot_from!(ShengjiSnapshot, Shengji);
impl_game_snapshot_from!(UnoSnapshot, Uno);
impl_game_snapshot_from!(MahjongSnapshot, Mahjong);
impl_game_snapshot_from!(MinecraftGuessSnapshot, MinecraftGuess);
impl_game_snapshot_from!(BlackjackSnapshot, Blackjack);
impl_game_snapshot_from!(DoudizhuSnapshot, Doudizhu);

impl GameSnapshot {
    /// A rematch can change the roster only after the whole match, never between hands.
    pub fn match_complete(&self) -> bool {
        match self {
            Self::QiGui523(snapshot) => matches!(snapshot.phase, GamePhaseView::Finished { .. }),
            Self::TexasHoldem(snapshot) => matches!(
                snapshot.phase,
                TexasHoldemPhaseView::HandComplete {
                    tournament_complete: true,
                    ..
                }
            ),
            // Shengji runs continuously; changing its four seats requires returning to the lobby.
            Self::Shengji(_) => false,
            Self::Uno(snapshot) => matches!(snapshot.phase, UnoPhaseView::Finished { .. }),
            Self::Mahjong(snapshot) => {
                matches!(&snapshot.phase, MahjongPhaseView::Finished { result } if result.match_complete)
            }
            Self::MinecraftGuess(snapshot) => {
                snapshot.progress.is_last_game() && snapshot.round.finished()
            }
            Self::Blackjack(snapshot) => snapshot.match_complete,
            Self::Doudizhu(snapshot) => snapshot.match_complete,
        }
    }

    /// The player whose private view is represented; spectators retain their Joined identity.
    pub const fn you(&self) -> PlayerId {
        match self {
            Self::QiGui523(snapshot) => snapshot.you,
            Self::TexasHoldem(snapshot) => snapshot.you,
            Self::Shengji(snapshot) => snapshot.you,
            Self::Uno(snapshot) => snapshot.you,
            Self::Mahjong(snapshot) => snapshot.you,
            Self::MinecraftGuess(snapshot) => snapshot.you,
            Self::Blackjack(snapshot) => snapshot.you,
            Self::Doudizhu(snapshot) => snapshot.you,
        }
    }

    pub const fn match_id(&self) -> MatchId {
        match self {
            Self::QiGui523(snapshot) => snapshot.match_id,
            Self::TexasHoldem(snapshot) => snapshot.match_id,
            Self::Shengji(snapshot) => snapshot.match_id,
            Self::Uno(snapshot) => snapshot.match_id,
            Self::Mahjong(snapshot) => snapshot.match_id,
            Self::MinecraftGuess(snapshot) => snapshot.match_id,
            Self::Blackjack(snapshot) => snapshot.match_id,
            Self::Doudizhu(snapshot) => snapshot.match_id,
        }
    }

    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
            Self::Mahjong(_) => GameKind::Mahjong,
            Self::MinecraftGuess(_) => GameKind::MinecraftGuess,
            Self::Blackjack(_) => GameKind::Blackjack,
            Self::Doudizhu(_) => GameKind::Doudizhu,
        }
    }

    game_snapshot_projections! {
        qigui523, into_qigui523 => QiGui523(QiGui523Snapshot),
        texas_holdem, into_texas_holdem => TexasHoldem(TexasHoldemSnapshot),
        shengji, into_shengji => Shengji(ShengjiSnapshot),
        uno, into_uno => Uno(UnoSnapshot),
        mahjong, into_mahjong => Mahjong(MahjongSnapshot),
        minecraft_guess, into_minecraft_guess => MinecraftGuess(MinecraftGuessSnapshot),
        blackjack, into_blackjack => Blackjack(BlackjackSnapshot),
        doudizhu, into_doudizhu => Doudizhu(DoudizhuSnapshot),
    }
}
