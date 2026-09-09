use crate::{
    GameKind, MahjongHandResultView, PlayerId, PublicPlay, ShengjiBottomFlipRevealView,
    ShengjiDeclarationView, ShengjiHandResultView, ShengjiPublicPlay,
};
use leocard_mahjong::{MahjongClaim, MahjongDrawOrigin, MahjongTile, MahjongTileKind};
use leocard_shengji::{ShengjiCard, ShengjiRank};
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemCard, TexasHoldemStreet};
use leocard_uno::{UnoCard, UnoChallengeResult, UnoColor, UnoDirection, UnoFlipSide};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameEvent {
    QiGui523(QiGui523Event),
    TexasHoldem(TexasHoldemEvent),
    Shengji(ShengjiEvent),
    Uno(UnoEvent),
    Mahjong(MahjongEvent),
}

macro_rules! impl_game_event_from {
    ($event:ty, $variant:ident) => {
        impl From<$event> for GameEvent {
            fn from(value: $event) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_game_event_from!(QiGui523Event, QiGui523);
impl_game_event_from!(TexasHoldemEvent, TexasHoldem);
impl_game_event_from!(ShengjiEvent, Shengji);
impl_game_event_from!(UnoEvent, Uno);
impl_game_event_from!(MahjongEvent, Mahjong);

impl GameEvent {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
            Self::Mahjong(_) => GameKind::Mahjong,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MahjongEvent {
    TileDiscarded {
        player: PlayerId,
        tile: MahjongTile,
    },
    TileDrawn {
        player: PlayerId,
        origin: MahjongDrawOrigin,
    },
    FlowerReplaced {
        player: PlayerId,
    },
    ClaimResolved {
        player: PlayerId,
        source: PlayerId,
        tile: MahjongTile,
        claim: MahjongClaim,
    },
    KongDeclared {
        player: PlayerId,
        tile: MahjongTileKind,
        added: bool,
    },
    FalseWin {
        player: PlayerId,
        deltas: [i32; 4],
    },
    HandFinished {
        result: MahjongHandResultView,
    },
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
        /// 同一次原子出牌中的零基下标；一次打出对子时依次为 0、1。
        play_index: u8,
        /// 同一次原子出牌包含的牌数；客户端据此只播放一次组合音效。
        play_count: u8,
    },
    CardsDrawn {
        player: PlayerId,
        count: u16,
        penalty: bool,
        card_backs: Vec<UnoCard>,
    },
    ChallengeResolved {
        challenger: PlayerId,
        offender: PlayerId,
        result: UnoChallengeResult,
        penalized: PlayerId,
        count: u16,
        card_backs: Vec<UnoCard>,
    },
    UnoCalled {
        player: PlayerId,
    },
    UnoReported {
        reporter: PlayerId,
        target: PlayerId,
        card_backs: Vec<UnoCard>,
    },
    SkipResolved {
        player: PlayerId,
        remaining: u16,
        drew_card: bool,
        card_back: Option<UnoCard>,
    },
    HandRefreshed {
        player: PlayerId,
        count: u16,
    },
    SwapOneCardTaken {
        player: PlayerId,
        target: PlayerId,
    },
    SwapOneCompleted {
        player: PlayerId,
        target: PlayerId,
    },
    HandsTraded {
        player: PlayerId,
        first: PlayerId,
        second: PlayerId,
    },
    HandsPassed {
        player: PlayerId,
        direction: UnoDirection,
    },
    DrawPenaltyReflected {
        player: PlayerId,
        target: PlayerId,
        count: u16,
        card_backs: Vec<UnoCard>,
    },
    StackNumberRevealed {
        player: PlayerId,
        cards: Vec<UnoCard>,
        value: u8,
    },
    CardsDiscarded {
        player: PlayerId,
        cards: Vec<UnoCard>,
    },
    Flipped {
        side: UnoFlipSide,
    },
    ColorRouletteResolved {
        player: PlayerId,
        color: UnoColor,
        count: u16,
        card_backs: Vec<UnoCard>,
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
