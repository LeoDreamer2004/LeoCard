//! 与传输方式无关的联机协议。
//!
//! TCP 层只需读取 4 字节大端长度，再读取对应的 Postcard 负载；业务层始终处理
//! [`ClientMessage`] 和 [`ServerMessage`]。

use std::fmt;

use leocard_qigui523::{Card, PlayKind, RuleSet};
use leocard_shengji::{
    BidKind as ShengjiBidKind, BidTrump as ShengjiBidTrump, Card as ShengjiCard,
    ClassifiedPlay as ShengjiClassifiedPlay, Rank as ShengjiRank, RuleSet as ShengjiRuleSet,
    TeamId as ShengjiTeamId, Trump as ShengjiTrump,
};
use leocard_texas_holdem::{
    Action as TexasHoldemAction, BlindKind as TexasHoldemBlindKind, Card as TexasHoldemCard,
    EvaluatedHand, HandCategory as TexasHoldemHandCategory, RuleSet as TexasHoldemRuleSet,
    Street as TexasHoldemStreet,
};
use leocard_uno::{
    Card as UnoCard, ChallengeResult as UnoChallengeResult, Color as UnoColor,
    Direction as UnoDirection, PendingDrawKind as UnoPendingDrawKind, RuleSet as UnoRuleSet,
};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 23;
pub const MAX_FRAME_PAYLOAD: usize = 1024 * 1024;
pub const MAX_PLAYER_NAME_CHARS: usize = 7;
pub const AVATAR_DIMENSION: u32 = 64;
pub const MAX_AVATAR_BYTES: usize = 32 * 1024;
pub const MAX_CHAT_MESSAGE_CHARS: usize = 120;
pub const QUICK_VOICE_COUNT: u8 = 23;
pub const TABLE_SEAT_COUNT: u8 = 6;

/// LeoCard 支持的游戏。房间、身份与聊天协议不依赖具体游戏；新增游戏时在这里
/// 注册对应的规则、命令、快照和事件后端。
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum GameKind {
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameRules {
    QiGui523(RuleSet),
    TexasHoldem(TexasHoldemRuleSet),
    Shengji(ShengjiRuleSet),
    Uno(UnoRuleSet),
}

impl GameRules {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
        }
    }

    pub const fn qigui523(&self) -> Option<&RuleSet> {
        match self {
            Self::QiGui523(rules) => Some(rules),
            Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub const fn texas_holdem(&self) -> Option<&TexasHoldemRuleSet> {
        match self {
            Self::TexasHoldem(rules) => Some(rules),
            Self::QiGui523(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub const fn shengji(&self) -> Option<&ShengjiRuleSet> {
        match self {
            Self::Shengji(rules) => Some(rules),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Uno(_) => None,
        }
    }

    pub const fn uno(&self) -> Option<&UnoRuleSet> {
        match self {
            Self::Uno(rules) => Some(rules),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) => None,
        }
    }
}

impl From<RuleSet> for GameRules {
    fn from(value: RuleSet) -> Self {
        Self::QiGui523(value)
    }
}

impl From<TexasHoldemRuleSet> for GameRules {
    fn from(value: TexasHoldemRuleSet) -> Self {
        Self::TexasHoldem(value)
    }
}

impl From<ShengjiRuleSet> for GameRules {
    fn from(value: ShengjiRuleSet) -> Self {
        Self::Shengji(value)
    }
}

impl From<UnoRuleSet> for GameRules {
    fn from(value: UnoRuleSet) -> Self {
        Self::Uno(value)
    }
}

/// 具体游戏的操作。通用房间命令保留在 [`ClientCommand`] 中。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameCommand {
    QiGui523(QiGui523Command),
    TexasHoldem(TexasHoldemCommand),
    Shengji(ShengjiCommand),
    Uno(UnoCommand),
}

impl GameCommand {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoCommand {
    SetAutoPlay {
        enabled: bool,
    },
    UpdateRules {
        rules: UnoRuleSet,
    },
    ChooseInitialColor {
        color: UnoColor,
    },
    PlayCard {
        card: UnoCard,
        chosen_color: Option<UnoColor>,
    },
    PlayCards {
        cards: Vec<UnoCard>,
        chosen_color: Option<UnoColor>,
    },
    JumpIn {
        card: UnoCard,
    },
    DrawCard,
    PassAfterDraw,
    AcceptDrawPenalty,
    ChallengeDrawFour,
    ResolveSkip,
    CallUno,
    ReportUno {
        target: PlayerId,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QiGui523Command {
    SetAutoPlay { enabled: bool },
    UpdateRules { rules: RuleSet },
    PlayCards { cards: Vec<Card> },
    SetDeveloperHand { cards: Vec<Card> },
    Pass,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasHoldemCommand {
    SetAutoPlay { enabled: bool },
    UpdateRules { rules: TexasHoldemRuleSet },
    Act { action: TexasHoldemAction },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiCommand {
    SetAutoPlay {
        enabled: bool,
    },
    UpdateRules {
        rules: ShengjiRuleSet,
    },
    /// 发牌中或发牌结束后的五秒窗口内亮主、加亮自保或反主。
    Declare {
        cards: Vec<ShengjiCard>,
    },
    /// 发牌完成后确认在当前亮主结果下不再亮主、反主或自保。任何新的声明
    /// 都会清空全桌确认；四人全部确认后可立即结束亮主阶段。
    ConfirmBidPass,
    /// 庄家拿到底牌后重新埋下规则要求的牌数（两/四副牌八张、三副牌六张）。
    Bury {
        cards: Vec<ShengjiCard>,
    },
    /// 当前被询问者提交更强的反主牌抄底；`None` 表示放弃本次机会。
    ChooseBottomCopy {
        cards: Option<Vec<ShengjiCard>>,
    },
    /// 符合资格者提交五张过江牌；`None` 表示明确不过江。
    ChooseFiveTrumpCrossing {
        cards: Option<Vec<ShengjiCard>>,
    },
    /// 对家看过收到的牌后选择任意五张归还。
    ReturnFiveTrumpCrossing {
        cards: Vec<ShengjiCard>,
    },
    PlayCards {
        cards: Vec<ShengjiCard>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RoomId(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlayerId(pub u8);

/// 跨房间稳定的本地玩家身份；其字节同时是 Ed25519 验证公钥。
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProfileId(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MatchId(pub [u8; 16]);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SeatId(pub u8);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PlayerInteractionKind {
    Flower,
    Egg,
    Wine,
    Shoe,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerInteraction {
    pub source: PlayerId,
    pub target: PlayerId,
    pub kind: PlayerInteractionKind,
    /// 由房主生成，使所有客户端选择相同音效和命中偏移。
    pub seed: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ChatContent {
    Text(String),
    QuickVoice(u8),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChatMessage {
    pub source: PlayerId,
    pub content: ChatContent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AvatarId(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ReconnectToken(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RequestId(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Revision(pub u64);

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
) -> Vec<u8> {
    const DOMAIN: &[u8] = b"leocard/join-identity/v1";
    let name = name.as_bytes();
    let mut payload = Vec::with_capacity(DOMAIN.len() + 8 + 8 + 4 + 4 + 4 + name.len());
    payload.extend_from_slice(DOMAIN);
    payload.extend_from_slice(&room_id.0.to_be_bytes());
    payload.extend_from_slice(&reconnect_token.0.to_be_bytes());
    payload.extend_from_slice(&reference_points.to_be_bytes());
    payload.extend_from_slice(&completed_games.to_be_bytes());
    payload.extend_from_slice(&(name.len() as u32).to_be_bytes());
    payload.extend_from_slice(name);
    payload
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ClientCommand {
    Join {
        name: String,
        reconnect_token: ReconnectToken,
        profile_id: ProfileId,
        reference_points: i32,
        completed_games: u32,
        identity_signature: Vec<u8>,
    },
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
    RequestSnapshot,
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
    pub const fn qigui523_rules(&self) -> Option<&RuleSet> {
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameSnapshot {
    QiGui523(QiGui523Snapshot),
    TexasHoldem(TexasHoldemSnapshot),
    Shengji(ShengjiSnapshot),
    Uno(UnoSnapshot),
}

impl GameSnapshot {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
        }
    }

    pub const fn qigui523(&self) -> Option<&QiGui523Snapshot> {
        match self {
            Self::QiGui523(snapshot) => Some(snapshot),
            Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub fn into_qigui523(self) -> Option<QiGui523Snapshot> {
        match self {
            Self::QiGui523(snapshot) => Some(snapshot),
            Self::TexasHoldem(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub const fn texas_holdem(&self) -> Option<&TexasHoldemSnapshot> {
        match self {
            Self::TexasHoldem(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub fn into_texas_holdem(self) -> Option<TexasHoldemSnapshot> {
        match self {
            Self::TexasHoldem(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::Shengji(_) | Self::Uno(_) => None,
        }
    }

    pub const fn shengji(&self) -> Option<&ShengjiSnapshot> {
        match self {
            Self::Shengji(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Uno(_) => None,
        }
    }

    pub fn into_shengji(self) -> Option<ShengjiSnapshot> {
        match self {
            Self::Shengji(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Uno(_) => None,
        }
    }

    pub const fn uno(&self) -> Option<&UnoSnapshot> {
        match self {
            Self::Uno(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) => None,
        }
    }

    pub fn into_uno(self) -> Option<UnoSnapshot> {
        match self {
            Self::Uno(snapshot) => Some(snapshot),
            Self::QiGui523(_) | Self::TexasHoldem(_) | Self::Shengji(_) => None,
        }
    }
}

/// 面向单个 UNO 客户端的私有快照。进行中只公开接收者的手牌；终局公开所有
/// 剩余手牌，以便核对排名分数。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: UnoRuleSet,
    pub players: Vec<UnoPlayerState>,
    pub your_hand: Vec<UnoCard>,
    pub draw_pile_len: u16,
    pub discard_top: UnoCard,
    /// 从旧到新排列的弃牌堆末尾，用于客户端绘制有轻微错位的牌堆。
    pub discard_pile: Vec<UnoCard>,
    pub current_color: Option<UnoColor>,
    pub current_player: Option<PlayerId>,
    pub direction: UnoDirection,
    pub pending_draw: u16,
    pub pending_kind: Option<UnoPendingDrawKind>,
    pub challenge_offender: Option<PlayerId>,
    pub pending_skip: u16,
    /// 只有接收者本人摸到可出的牌并仍在本回合时才为 `Some`。
    pub your_drawn_card: Option<UnoCard>,
    /// 抢出窗口内，只有实际持有匹配牌的非下家会收到这张私有候选牌。
    pub your_jump_in_card: Option<UnoCard>,
    pub uno_exposed: Vec<PlayerId>,
    pub uno_declared: Vec<PlayerId>,
    pub phase: UnoPhaseView,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub hand_len: u8,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub skipped_turns: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoPhaseView {
    Playing,
    Finished {
        winner: PlayerId,
        results: Vec<UnoPlayerResult>,
        remaining_hands: Vec<UnoRevealedHand>,
        reference_changes: Vec<PlayerReferenceChange>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoPlayerResult {
    pub player: PlayerId,
    pub hand_score: u16,
    pub placement: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoRevealedHand {
    pub player: PlayerId,
    pub cards: Vec<UnoCard>,
}

/// 面向单个七鬼五二三客户端生成的状态。对局中只含 `your_hand`；终局后才会在
/// [`GamePhaseView::Finished`] 中公开各玩家剩余的手牌。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523Snapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<PlayerPublicState>,
    pub your_hand: Vec<Card>,
    pub draw_pile_len: u16,
    pub starting_card: StartingCardView,
    pub trick: Option<TrickView>,
    pub turn_timer: Option<TurnTimerView>,
    pub phase: GamePhaseView,
}

/// 面向单个升级客户端生成的私有快照。进行中只发送接收者自己的手牌；其他玩家
/// 仅公开手牌张数。当接收者是最后一位埋底者时，[`ShengjiSnapshot::your_buried`]
/// 会私下发送他自己埋下的牌；终局才通过 [`ShengjiPhaseView::Finished`] 对所有人公开。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiSnapshot {
    pub match_id: MatchId,
    pub hand_number: u32,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: ShengjiRuleSet,
    pub players: Vec<ShengjiPlayerState>,
    pub your_hand: Vec<ShengjiCard>,
    /// 仅向接收者公开其本人此前亮过的物理牌；王在带王亮规则下可以复用，
    /// 级牌不可复用。用于重连后继续生成正确的亮主候选。
    pub your_exposed_cards: Vec<ShengjiCard>,
    pub levels: [ShengjiRank; 2],
    /// 本局抢亮所使用的级牌。首局无人亮主前庄家尚未公开；首局亮主后随当前
    /// 最高声明实时转移，后续小局则从发牌开始公开上一局已经确定的庄家。
    pub bidding_level: ShengjiRank,
    pub dealer: Option<PlayerId>,
    pub trump: Option<ShengjiTrump>,
    pub declaration: Option<ShengjiDeclarationView>,
    pub current_player: Option<PlayerId>,
    pub trick: Option<ShengjiTrickView>,
    /// 甩牌失败的公开演示状态；存在时先展示原甩牌，再收回并显示强制小牌。
    pub throw_failure: Option<ShengjiThrowFailureView>,
    /// 闲家当前总得分，已计入甩牌罚分，但尚未计入未发生的抠底。
    pub collecting_score: u32,
    /// 已埋底的张数；进行中只公开数量，不公开牌面。
    pub buried_count: u8,
    /// 仅最后一位埋底者可见的当前底牌。成功抄底取走旧底后，
    /// 该字段会在新的埋底操作完成前变回空。
    pub your_buried: Vec<ShengjiCard>,
    pub phase: ShengjiPhaseView,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub hand_len: u8,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiDeclarationView {
    pub player: PlayerId,
    pub trump: ShengjiBidTrump,
    pub kind: ShengjiBidKind,
    pub protected: bool,
    /// 亮出的实体牌；同牌面的不同副牌仍具有不同的牌标识。
    pub cards: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiTrickView {
    pub leader: PlayerId,
    pub current_player: PlayerId,
    pub winning_player: PlayerId,
    pub plays: Vec<ShengjiPublicPlay>,
    pub table_points: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiPublicPlay {
    pub player: PlayerId,
    /// 甩牌失败时这里只包含被强制打出的最小可失败牌型。
    pub play: ShengjiClassifiedPlay,
    pub throw_penalty: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiThrowFailureStage {
    Showing,
    Returning,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiThrowFailureView {
    pub player: PlayerId,
    /// 玩家最初尝试甩出的全部实体牌。
    pub attempted: Vec<ShengjiCard>,
    /// 展示结束后实际强制打出的最小可失败牌型。
    pub forced: ShengjiClassifiedPlay,
    pub penalty_points: u16,
    pub stage: ShengjiThrowFailureStage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiPhaseView {
    Dealing {
        cards_remaining: u8,
    },
    BiddingGrace {
        milliseconds_remaining: u16,
        power_outage: bool,
        confirmed_count: u8,
        you_confirmed: bool,
    },
    BottomFlipping {
        reveal: Option<ShengjiBottomFlipRevealView>,
    },
    Burying,
    BottomCopying {
        player: PlayerId,
        milliseconds_remaining: u16,
    },
    BottomCopyBurying {
        player: PlayerId,
    },
    FiveTrumpCrossing {
        stage: ShengjiFiveTrumpCrossingStage,
        eligible: Vec<PlayerId>,
        decided: Vec<PlayerId>,
        crossing: Vec<PlayerId>,
        returned: Vec<PlayerId>,
        /// 决定阶段的剩余时间；无时限的返还阶段固定为 0。
        milliseconds_remaining: u16,
    },
    Playing,
    Finished {
        result: ShengjiHandResultView,
        /// 本局结束后公开，用于核对抠底得分。
        buried: Vec<ShengjiCard>,
    },
    Redealing,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiFiveTrumpCrossingStage {
    Deciding,
    Returning,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiBottomFlipMatchView {
    pub player: PlayerId,
    pub cards: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiBottomFlipRevealView {
    pub card: ShengjiCard,
    pub matches: Vec<ShengjiBottomFlipMatchView>,
    pub dealer: Option<PlayerId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiHandResultView {
    /// 每小局独立的结算标识；同一场升级可连续进行多小局。
    pub settlement_id: MatchId,
    pub dealer: PlayerId,
    pub dealer_team: ShengjiTeamId,
    pub collecting_team: ShengjiTeamId,
    pub trick_points: u32,
    pub penalty_adjustment: i32,
    pub kitty_points: u16,
    pub kitty_multiplier: u32,
    pub collecting_score: u32,
    pub promoted_team: ShengjiTeamId,
    pub promoted_steps: u8,
    pub next_dealer: PlayerId,
    pub levels: [ShengjiRank; 2],
    /// 本小局对四名玩家平台积分的实际变动。
    pub reference_changes: Vec<PlayerReferenceChange>,
}

/// 面向单个德州扑克客户端的私有快照。牌局进行中只有 `your_hole_cards` 包含
/// 底牌；只有进入摊牌的牌局完成后，玩家底牌才会出现在 `revealed_hands`。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemSnapshot {
    pub match_id: MatchId,
    pub hand_number: u32,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<TexasHoldemPlayerState>,
    pub your_hole_cards: Vec<TexasHoldemCard>,
    pub revealed_hands: Vec<TexasHoldemRevealedHand>,
    pub community: Vec<TexasHoldemCard>,
    /// 尚未从权威牌堆发出的牌数；客户端可扣除仍扣置在桌面的公共牌数量来绘制牌堆。
    pub draw_pile_len: u16,
    pub dealer: PlayerId,
    pub small_blind: PlayerId,
    pub big_blind: PlayerId,
    pub current_player: Option<PlayerId>,
    pub blind_to_post: Option<TexasHoldemBlindView>,
    pub current_bet: u32,
    pub minimum_raise_to: u32,
    pub amount_to_call: u32,
    pub raise_allowed: bool,
    pub pot: u32,
    pub phase: TexasHoldemPhaseView,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemBlindView {
    pub player: PlayerId,
    pub kind: TexasHoldemBlindKind,
    pub amount: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemPlayerState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub stack: u32,
    /// 本手开始时的筹码，用于在客户端显示本手盈亏。
    pub hand_start_stack: u32,
    pub committed_street: u32,
    pub committed_total: u32,
    pub folded: bool,
    pub all_in: bool,
    pub connected: bool,
    /// 由房主执行保守策略的托管状态；开发者模式补入的机器人始终为 `true`。
    pub auto_play: bool,
    /// 本手结算窗口中的下一手准备状态；房主不享有默认准备。
    pub ready: bool,
    pub reference_points: i32,
    pub completed_games: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemRevealedHand {
    pub player: PlayerId,
    /// 标准德州为两张，奥马哈为四张。
    pub cards: Vec<TexasHoldemCard>,
    /// 未发满五张公共牌便因其他玩家弃牌结束时，可能不足以组成五张牌型。
    pub best: Option<EvaluatedHand>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasHoldemPhaseView {
    Betting {
        street: TexasHoldemStreet,
    },
    HandComplete {
        showdown: bool,
        awards: Vec<TexasHoldemPotAward>,
        table_winner: Option<PlayerId>,
        /// 任意玩家筹码归零时，本场比赛立即结束。
        tournament_complete: bool,
        /// 仅整场结束时包含共享参考积分变化。
        reference_changes: Vec<PlayerReferenceChange>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemPotAward {
    pub amount: u32,
    pub winners: Vec<PlayerId>,
    pub winning_category: Option<TexasHoldemHandCategory>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameEvent {
    QiGui523(QiGui523Event),
    TexasHoldem(TexasHoldemEvent),
    Shengji(ShengjiEvent),
    Uno(UnoEvent),
}

impl GameEvent {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoEvent {
    ColorChosen {
        player: PlayerId,
        color: UnoColor,
    },
    CardPlayed {
        player: PlayerId,
        card: UnoCard,
        chosen_color: Option<UnoColor>,
    },
    CardsDrawn {
        player: PlayerId,
        count: u16,
        penalty: bool,
    },
    ChallengeResolved {
        challenger: PlayerId,
        offender: PlayerId,
        result: UnoChallengeResult,
        penalized: PlayerId,
        count: u16,
    },
    UnoCalled {
        player: PlayerId,
    },
    UnoReported {
        reporter: PlayerId,
        target: PlayerId,
    },
    SkipResolved {
        player: PlayerId,
        remaining: u16,
        drew_card: bool,
    },
    GameFinished {
        winner: PlayerId,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QiGui523Event {
    PlayEffect { player: PlayerId, play: PublicPlay },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasHoldemEvent {
    ActionApplied {
        player: PlayerId,
        action: TexasHoldemAction,
        /// 此动作实际从玩家剩余筹码中投入的数量；过牌和弃牌为 0。
        amount: u32,
    },
    StreetAdvanced {
        street: TexasHoldemStreet,
        dealt: Vec<TexasHoldemCard>,
    },
    HandFinished {
        showdown: bool,
    },
}

/// 升级的公开增量事件。私有发牌与底牌牌面只经由接收者专属快照发送。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiEvent {
    /// 此事件按接收者生成：只有被发牌玩家本人会收到 `card: Some(_)`。
    CardDealt {
        player: PlayerId,
        hand_len: u8,
        card: Option<ShengjiCard>,
    },
    DeclarationChanged {
        declaration: ShengjiDeclarationView,
    },
    /// 当前亮主结果已经锁定；无声明表示无人亮主并进入相应特殊规则。
    BiddingLocked {
        declaration: Option<ShengjiDeclarationView>,
    },
    PowerOutageDealerChanged {
        dealer: PlayerId,
        level: ShengjiRank,
    },
    BottomCardRevealed {
        reveal: ShengjiBottomFlipRevealView,
    },
    BottomCopied {
        declaration: ShengjiDeclarationView,
    },
    FiveTrumpCrossingStarted {
        players: Vec<PlayerId>,
    },
    FiveTrumpCrossingReturned {
        player: PlayerId,
        complete: bool,
    },
    DealCompleted,
    DealerTookKitty {
        dealer: PlayerId,
    },
    CardsBuried {
        dealer: PlayerId,
    },
    CardsPlayed {
        play: ShengjiPublicPlay,
        /// 是否为本墩首家出牌；跟牌的混合结构不能据此冒充甩牌。
        is_lead: bool,
    },
    TrickFinished {
        winner: PlayerId,
        points: u16,
        collecting_score: u32,
    },
    HandFinished {
        result: ShengjiHandResultView,
        buried: Vec<ShengjiCard>,
    },
    RedealRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnTimerView {
    pub player: PlayerId,
    pub base_seconds: u16,
    pub reserve_seconds: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerPublicState {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub hand_len: u16,
    pub score: u32,
    pub ready: bool,
    pub connected: bool,
    /// 由房主执行贪心策略的托管状态；所有客户端都可见。
    pub auto_play: bool,
    pub reference_points: i32,
    pub completed_games: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StartingCardView {
    pub player: PlayerId,
    pub card: Card,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrickView {
    pub leader: PlayerId,
    pub current_player: PlayerId,
    pub winning_player: Option<PlayerId>,
    pub winning_play: Option<PublicPlay>,
    pub records: Vec<PublicPlayRecord>,
    pub table_points: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicPlay {
    pub kind: PlayKind,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PublicPlayRecord {
    Played { player: PlayerId, play: PublicPlay },
    Passed { player: PlayerId },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GamePhaseView {
    Playing,
    Finished {
        match_id: MatchId,
        finisher: PlayerId,
        scores: Vec<PlayerScore>,
        remaining_hands: Vec<RevealedHand>,
        reference_changes: Vec<PlayerReferenceChange>,
        captured_hand_points: u32,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RevealedHand {
    pub player: PlayerId,
    pub cards: Vec<Card>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerScore {
    pub player: PlayerId,
    pub score: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerReferenceChange {
    pub player: PlayerId,
    pub profile_id: ProfileId,
    pub delta: i16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RejectReason {
    ProtocolMismatch {
        expected: u16,
        received: u16,
    },
    RoomMismatch,
    DuplicateRequest {
        last_seen: RequestId,
    },
    StaleRequest {
        last_seen: RequestId,
    },
    AlreadyJoined,
    NotJoined,
    NameEmpty,
    NameTooLong {
        max_chars: u16,
    },
    InvalidIdentityProof,
    RoomFull,
    InvalidAvatar,
    AvatarAlreadySet,
    InvalidSeat,
    SeatTaken,
    MustSelectSeat,
    OnlyHostCanConfigure,
    OnlyHostCanStart,
    OnlyHostCanReturnToLobby,
    OnlyHostCanCloseRoom,
    NotEnoughPlayers {
        minimum: u8,
        actual: u8,
    },
    WaitingForPlayers {
        expected: u8,
        actual: u8,
    },
    PlayersNotReady {
        players: Vec<PlayerId>,
    },
    InvalidRuleConfiguration,
    DeveloperFeatureUnavailable,
    InvalidDeveloperHand,
    InvalidChatMessage,
    WrongGame {
        expected: GameKind,
        received: GameKind,
    },
    GameNotStarted,
    GameNotFinished,
    GameAlreadyStarted,
    GameViolation(GameViolation),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameViolation {
    QiGui523(RuleViolation),
    TexasHoldem(TexasHoldemViolation),
    Shengji(ShengjiViolation),
    Uno(UnoViolation),
}

impl From<RuleViolation> for GameViolation {
    fn from(value: RuleViolation) -> Self {
        Self::QiGui523(value)
    }
}

impl From<TexasHoldemViolation> for GameViolation {
    fn from(value: TexasHoldemViolation) -> Self {
        Self::TexasHoldem(value)
    }
}

impl From<ShengjiViolation> for GameViolation {
    fn from(value: ShengjiViolation) -> Self {
        Self::Shengji(value)
    }
}

impl From<UnoViolation> for GameViolation {
    fn from(value: UnoViolation) -> Self {
        Self::Uno(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoViolation {
    InvalidPlayer,
    NotPlayersTurn,
    GameAlreadyFinished,
    InitialColorChoiceRequired,
    InitialColorAlreadyChosen,
    CardNotInHand,
    CardDoesNotMatch,
    ColorRequired,
    UnexpectedColor,
    MustPlayDrawnCard,
    MustResolveDrawPenalty,
    NoDrawPenalty,
    CannotStack,
    CannotChallenge,
    MustDrawBeforePassing,
    MustResolveSkip,
    NoSkipToResolve,
    UnoCalloutDisabled,
    CannotCallUno,
    MustPlayAfterUno,
    CannotReportSelf,
    PlayerNotReportable,
    CannotPlayTogether,
    CannotJumpIn,
    DrawPileExhausted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RuleViolation {
    InvalidPlayer,
    NotPlayersTurn,
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand,
    InvalidPattern,
    PlayDoesNotBeatCurrent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasHoldemViolation {
    InvalidPlayer,
    NotPlayersTurn,
    HandAlreadyComplete,
    PlayerCannotAct,
    MustPostBlind,
    NoBlindToPost,
    CannotCheckWhileFacingBet { amount_to_call: u32 },
    NothingToCall,
    RaiseMustExceedCurrentBet { current_bet: u32, target: u32 },
    RaiseBelowMinimum { minimum_target: u32, target: u32 },
    RaiseExceedsStack { maximum_target: u32, target: u32 },
    RaiseNotReopened,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiViolation {
    InvalidPlayer,
    WrongPhase,
    InvalidDeclaration,
    DeclarationCardsNotOwned,
    CounterRequiresPair,
    CounterNotStronger,
    ProtectedSuitCanOnlyBeCounteredByNoTrump,
    DeclarationRequiresJoker,
    NoTrumpCannotOpen,
    NotDealer,
    WrongBuryCount { expected: u8, actual: u8 },
    CrossingNotEligible,
    CrossingAlreadyDecided,
    WrongCrossingCount { expected: u8, actual: u8 },
    CrossingMustIncludeAllTrumps,
    CrossingReturnNotRequired,
    CrossingAlreadyReturned,
    NotBottomCopyPlayer,
    CardsNotOwned,
    NotPlayersTurn,
    MustLeadWithCards,
    ThrowDisabled,
    InvalidPattern,
    WrongCardCount { expected: u8, actual: u8 },
    MustFollowCategory,
    MustFollowStructure,
}

#[derive(Debug)]
pub enum FrameError {
    Serialize(postcard::Error),
    Deserialize(postcard::Error),
    FrameTooShort,
    PayloadTooLarge { actual: usize, maximum: usize },
    LengthMismatch { declared: usize, actual: usize },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(f, "failed to serialize frame: {error}"),
            Self::Deserialize(error) => write!(f, "failed to deserialize frame: {error}"),
            Self::FrameTooShort => f.write_str("frame is shorter than its 4-byte length prefix"),
            Self::PayloadTooLarge { actual, maximum } => {
                write!(f, "frame payload is {actual} bytes; maximum is {maximum}")
            }
            Self::LengthMismatch { declared, actual } => {
                write!(
                    f,
                    "frame declares {declared} payload bytes but contains {actual}"
                )
            }
        }
    }
}

impl std::error::Error for FrameError {}

/// 编码为可直接写入 TCP 的一帧：4 字节大端负载长度 + Postcard 负载。
pub fn encode_frame<T: Serialize>(value: &T) -> Result<Vec<u8>, FrameError> {
    let payload = postcard::to_allocvec(value).map_err(FrameError::Serialize)?;
    if payload.len() > MAX_FRAME_PAYLOAD {
        return Err(FrameError::PayloadTooLarge {
            actual: payload.len(),
            maximum: MAX_FRAME_PAYLOAD,
        });
    }
    let payload_len = u32::try_from(payload.len()).expect("maximum payload fits in u32");
    let mut frame = Vec::with_capacity(payload.len() + 4);
    frame.extend_from_slice(&payload_len.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// 解码一整帧。TCP 实现应先 `read_exact(4)`，校验长度，再 `read_exact(length)`。
pub fn decode_frame<'a, T: Deserialize<'a>>(frame: &'a [u8]) -> Result<T, FrameError> {
    let prefix: [u8; 4] = frame
        .get(..4)
        .ok_or(FrameError::FrameTooShort)?
        .try_into()
        .expect("slice length was checked");
    let declared = u32::from_be_bytes(prefix) as usize;
    if declared > MAX_FRAME_PAYLOAD {
        return Err(FrameError::PayloadTooLarge {
            actual: declared,
            maximum: MAX_FRAME_PAYLOAD,
        });
    }
    let payload = &frame[4..];
    if payload.len() != declared {
        return Err(FrameError::LengthMismatch {
            declared,
            actual: payload.len(),
        });
    }
    postcard::from_bytes(payload).map_err(FrameError::Deserialize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_deck_spaceship_component_round_trips_through_protocol_codec() {
        let cards = [leocard_shengji::Rank::Three, leocard_shengji::Rank::Four]
            .into_iter()
            .flat_map(|rank| {
                (0..4)
                    .map(move |deck| ShengjiCard::suited(deck, leocard_shengji::Suit::Spade, rank))
            })
            .collect::<Vec<_>>();
        let component = leocard_shengji::Component::Spaceship {
            cards,
            quad_count: 2,
            top_strength: 2,
        };
        let decoded: leocard_shengji::Component =
            decode_frame(&encode_frame(&component).unwrap()).unwrap();
        assert_eq!(decoded, component);
    }

    #[test]
    fn client_message_round_trips_through_tcp_frame() {
        let message = ClientMessage::new(
            RoomId(42),
            RequestId(7),
            ClientCommand::Join {
                name: "玩家一".to_owned(),
                reconnect_token: ReconnectToken(123),
                profile_id: ProfileId([7; 32]),
                reference_points: 12,
                completed_games: 8,
                identity_signature: vec![9; 64],
            },
        );
        let frame = encode_frame(&message).unwrap();
        let decoded: ClientMessage = decode_frame(&frame).unwrap();
        assert_eq!(decoded, message);
        assert_eq!(
            u32::from_be_bytes(frame[..4].try_into().unwrap()) as usize,
            frame.len() - 4
        );
    }

    #[test]
    fn interaction_event_round_trips_with_its_shared_animation_seed() {
        let message = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(42),
            revision: Revision(9),
            in_reply_to: Some(RequestId(7)),
            event: ServerEvent::PlayerInteraction(PlayerInteraction {
                source: PlayerId(1),
                target: PlayerId(3),
                kind: PlayerInteractionKind::Wine,
                seed: 523,
            }),
        };

        let frame = encode_frame(&message).unwrap();
        let decoded: ServerMessage = decode_frame(&frame).unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn chat_events_round_trip_for_text_and_quick_voice() {
        for content in [
            ChatContent::Text("大家好".to_owned()),
            ChatContent::QuickVoice(7),
        ] {
            let message = ServerMessage {
                protocol_version: PROTOCOL_VERSION,
                room_id: RoomId(42),
                revision: Revision(9),
                in_reply_to: Some(RequestId(7)),
                event: ServerEvent::ChatMessage(ChatMessage {
                    source: PlayerId(1),
                    content,
                }),
            };
            let frame = encode_frame(&message).unwrap();
            let decoded: ServerMessage = decode_frame(&frame).unwrap();
            assert_eq!(decoded, message);
        }
    }

    #[test]
    fn concrete_game_commands_round_trip_through_the_common_protocol() {
        let command = ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
            enabled: true,
        }));
        let message = ClientMessage::new(RoomId(42), RequestId(8), command.clone());
        let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();

        assert_eq!(decoded.command, command);
        let ClientCommand::Game(game_command) = decoded.command else {
            panic!("game command should retain its common wrapper");
        };
        assert_eq!(game_command.kind(), GameKind::QiGui523);
    }

    #[test]
    fn developer_bot_seat_command_round_trips_through_the_common_protocol() {
        for occupied in [true, false] {
            let command = ClientCommand::ConfigureBotSeat {
                seat: SeatId(3),
                occupied,
            };
            let message = ClientMessage::new(RoomId(42), RequestId(8), command.clone());
            let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();

            assert_eq!(decoded.command, command);
        }
    }

    #[test]
    fn texas_holdem_commands_round_trip_through_the_common_protocol() {
        let rules = TexasHoldemRuleSet {
            player_count: 6,
            starting_chips: 40,
            short_deck: true,
            ignore_kickers: true,
            omaha: true,
        };
        for command in [
            GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay { enabled: true }),
            GameCommand::TexasHoldem(TexasHoldemCommand::UpdateRules { rules }),
            GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                action: TexasHoldemAction::RaiseTo(12),
            }),
        ] {
            let message = ClientMessage::new(
                RoomId(42),
                RequestId(8),
                ClientCommand::Game(command.clone()),
            );
            let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
            assert_eq!(decoded.command, ClientCommand::Game(command));
        }
    }

    #[test]
    fn shengji_commands_round_trip_through_the_common_protocol() {
        let first_ten = ShengjiCard::suited(0, leocard_shengji::Suit::Heart, ShengjiRank::Ten);
        let second_ten = ShengjiCard::suited(1, leocard_shengji::Suit::Heart, ShengjiRank::Ten);
        for command in [
            GameCommand::Shengji(ShengjiCommand::SetAutoPlay { enabled: true }),
            GameCommand::Shengji(ShengjiCommand::UpdateRules {
                rules: ShengjiRuleSet::default(),
            }),
            GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![first_ten, second_ten],
            }),
            GameCommand::Shengji(ShengjiCommand::ConfirmBidPass),
            GameCommand::Shengji(ShengjiCommand::Bury {
                cards: leocard_shengji::build_deck()[..8].to_vec(),
            }),
            GameCommand::Shengji(ShengjiCommand::ChooseBottomCopy {
                cards: Some(vec![first_ten, second_ten]),
            }),
            GameCommand::Shengji(ShengjiCommand::ChooseBottomCopy { cards: None }),
            GameCommand::Shengji(ShengjiCommand::ChooseFiveTrumpCrossing {
                cards: Some(vec![first_ten, second_ten]),
            }),
            GameCommand::Shengji(ShengjiCommand::ChooseFiveTrumpCrossing { cards: None }),
            GameCommand::Shengji(ShengjiCommand::ReturnFiveTrumpCrossing {
                cards: vec![first_ten, second_ten],
            }),
            GameCommand::Shengji(ShengjiCommand::PlayCards {
                cards: vec![first_ten],
            }),
        ] {
            let message = ClientMessage::new(
                RoomId(42),
                RequestId(9),
                ClientCommand::Game(command.clone()),
            );
            let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
            assert_eq!(decoded.command, ClientCommand::Game(command));
        }
    }

    #[test]
    fn uno_pair_and_jump_in_commands_round_trip_through_the_common_protocol() {
        let first = UnoCard::number(UnoColor::Red, 7, 0);
        let second = UnoCard::number(UnoColor::Red, 7, 1);
        for command in [
            GameCommand::Uno(UnoCommand::PlayCards {
                cards: vec![first, second],
                chosen_color: None,
            }),
            GameCommand::Uno(UnoCommand::JumpIn { card: second }),
        ] {
            let message = ClientMessage::new(
                RoomId(42),
                RequestId(10),
                ClientCommand::Game(command.clone()),
            );
            let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
            assert_eq!(decoded.command, ClientCommand::Game(command));
        }
    }

    #[test]
    fn active_shengji_snapshot_round_trips_private_hand_and_own_bottom() {
        let your_card = ShengjiCard::suited(0, leocard_shengji::Suit::Spade, ShengjiRank::Ace);
        let exposed = ShengjiCard::suited(0, leocard_shengji::Suit::Heart, ShengjiRank::Ten);
        let players = (0..4)
            .map(|id| ShengjiPlayerState {
                id: PlayerId(id),
                profile_id: ProfileId([id; 32]),
                name: format!("玩家{id}"),
                avatar: None,
                seat: SeatId(id),
                hand_len: 25,
                ready: false,
                connected: true,
                auto_play: false,
                reference_points: 0,
                completed_games: 0,
            })
            .collect();
        let snapshot = GameSnapshot::Shengji(ShengjiSnapshot {
            match_id: MatchId([8; 16]),
            hand_number: 1,
            host_port: 52300,
            you: PlayerId(2),
            host: PlayerId(0),
            rules: ShengjiRuleSet {
                deck_count: 3,
                bid_with_joker: true,
                constant_trump: true,
                ..ShengjiRuleSet::default()
            },
            players,
            your_hand: vec![your_card],
            your_exposed_cards: vec![exposed],
            levels: [ShengjiRank::Ten, ShengjiRank::Nine],
            bidding_level: ShengjiRank::Ten,
            dealer: Some(PlayerId(0)),
            trump: Some(
                ShengjiTrump::new(ShengjiRank::Ten, Some(leocard_shengji::Suit::Heart))
                    .unwrap()
                    .with_constant_trump(true),
            ),
            declaration: Some(ShengjiDeclarationView {
                player: PlayerId(0),
                trump: ShengjiBidTrump::Suit(leocard_shengji::Suit::Heart),
                kind: ShengjiBidKind::Initial,
                protected: false,
                cards: vec![exposed],
            }),
            current_player: Some(PlayerId(0)),
            trick: None,
            throw_failure: None,
            collecting_score: 15,
            buried_count: 8,
            your_buried: vec![exposed],
            phase: ShengjiPhaseView::Playing,
        });

        let decoded: GameSnapshot = decode_frame(&encode_frame(&snapshot).unwrap()).unwrap();
        let decoded = decoded.into_shengji().unwrap();
        assert_eq!(decoded.you, PlayerId(2));
        assert_eq!(decoded.your_hand, vec![your_card]);
        assert_eq!(decoded.your_exposed_cards, vec![exposed]);
        assert_eq!(decoded.your_buried, vec![exposed]);
        assert_eq!(
            decoded
                .players
                .iter()
                .map(|player| player.hand_len)
                .sum::<u8>(),
            100
        );
        assert_eq!(decoded.buried_count, 8);
        assert!(decoded.rules.constant_trump);
        assert!(decoded.rules.bid_with_joker);
        assert_eq!(decoded.rules.deck_count, 3);
        assert!(decoded.trump.unwrap().constant_trump);
        assert!(matches!(decoded.phase, ShengjiPhaseView::Playing));
    }

    #[test]
    fn lobby_game_kind_and_rules_remain_consistent() {
        let rules = RuleSet::default();
        let lobby = LobbySnapshot {
            game: GameKind::QiGui523,
            rules: GameRules::QiGui523(rules),
            host_port: 52300,
            host: None,
            players: Vec::new(),
        };

        assert_eq!(lobby.game, lobby.rules.kind());
        assert_eq!(lobby.qigui523_rules(), Some(&rules));

        let texas_rules = TexasHoldemRuleSet::default();
        let texas_lobby = LobbySnapshot {
            game: GameKind::TexasHoldem,
            rules: GameRules::TexasHoldem(texas_rules),
            host_port: 52301,
            host: None,
            players: Vec::new(),
        };
        assert_eq!(texas_lobby.game, texas_lobby.rules.kind());
        assert_eq!(texas_lobby.texas_holdem_rules(), Some(&texas_rules));

        let shengji_rules = ShengjiRuleSet::default();
        let shengji_lobby = LobbySnapshot {
            game: GameKind::Shengji,
            rules: GameRules::Shengji(shengji_rules),
            host_port: 52302,
            host: None,
            players: Vec::new(),
        };
        assert_eq!(shengji_lobby.game, shengji_lobby.rules.kind());
        assert_eq!(shengji_lobby.shengji_rules(), Some(&shengji_rules));
    }

    #[test]
    fn rejects_truncated_and_oversized_frames_before_deserialization() {
        assert!(matches!(
            decode_frame::<ClientMessage>(&[0, 1, 2]),
            Err(FrameError::FrameTooShort)
        ));
        assert!(matches!(
            decode_frame::<ClientMessage>(&[0, 0, 0, 2, 1]),
            Err(FrameError::LengthMismatch {
                declared: 2,
                actual: 1
            })
        ));

        let too_large = u32::try_from(MAX_FRAME_PAYLOAD + 1).unwrap().to_be_bytes();
        assert!(matches!(
            decode_frame::<ClientMessage>(&too_large),
            Err(FrameError::PayloadTooLarge { .. })
        ));
    }
}
