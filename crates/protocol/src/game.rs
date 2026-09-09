use crate::PlayerId;
use leocard_mahjong::{MahjongClaim, MahjongRuleSet, MahjongTile, MahjongTileKind};
use leocard_qigui523::{QiGuiCard, QiGuiRuleSet};
use leocard_shengji::{ShengjiCard, ShengjiRuleSet};
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemRuleSet};
use leocard_uno::{UnoCard, UnoColor, UnoRuleSet};
use serde::{Deserialize, Serialize};

/// LeoCard 支持的游戏。房间、身份与聊天协议不依赖具体游戏；新增游戏时在这里
/// 注册对应的规则、命令、快照和事件后端。
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum GameKind {
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
    Mahjong,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameRules {
    QiGui523(QiGuiRuleSet),
    TexasHoldem(TexasHoldemRuleSet),
    Shengji(ShengjiRuleSet),
    Uno(UnoRuleSet),
    Mahjong(MahjongRuleSet),
}

macro_rules! game_rule_projections {
    ($($method:ident => $variant:ident($rules:ty)),+ $(,)?) => {
        $(
            pub const fn $method(&self) -> Option<&$rules> {
                match self {
                    Self::$variant(rules) => Some(rules),
                    _ => None,
                }
            }
        )+
    };
}

impl GameRules {
    pub const fn kind(&self) -> GameKind {
        match self {
            Self::QiGui523(_) => GameKind::QiGui523,
            Self::TexasHoldem(_) => GameKind::TexasHoldem,
            Self::Shengji(_) => GameKind::Shengji,
            Self::Uno(_) => GameKind::Uno,
            Self::Mahjong(_) => GameKind::Mahjong,
        }
    }

    game_rule_projections! {
        qigui523 => QiGui523(QiGuiRuleSet),
        texas_holdem => TexasHoldem(TexasHoldemRuleSet),
        shengji => Shengji(ShengjiRuleSet),
        uno => Uno(UnoRuleSet),
        mahjong => Mahjong(MahjongRuleSet),
    }
}

impl From<QiGuiRuleSet> for GameRules {
    fn from(value: QiGuiRuleSet) -> Self {
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

impl From<MahjongRuleSet> for GameRules {
    fn from(value: MahjongRuleSet) -> Self {
        Self::Mahjong(value)
    }
}

/// 具体游戏的操作。通用房间命令保留在 [`crate::ClientCommand`] 中。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameCommand {
    QiGui523(QiGui523Command),
    TexasHoldem(TexasHoldemCommand),
    Shengji(ShengjiCommand),
    Uno(UnoCommand),
    Mahjong(MahjongCommand),
}

macro_rules! impl_game_command_from {
    ($command:ty, $variant:ident) => {
        impl From<$command> for GameCommand {
            fn from(value: $command) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_game_command_from!(QiGui523Command, QiGui523);
impl_game_command_from!(TexasHoldemCommand, TexasHoldem);
impl_game_command_from!(ShengjiCommand, Shengji);
impl_game_command_from!(UnoCommand, Uno);
impl_game_command_from!(MahjongCommand, Mahjong);

impl GameCommand {
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
pub enum MahjongCommand {
    UpdateRules { rules: MahjongRuleSet },
    SetDeveloperHand { tiles: Vec<MahjongTileKind> },
    Discard { tile: MahjongTile },
    RespondToClaim { claim: MahjongClaim },
    DeclareSelfDraw,
    DeclareConcealedKong { tile: MahjongTileKind },
    DeclareAddedKong { tile: MahjongTile },
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
    ChooseSwapOneTarget {
        target: PlayerId,
    },
    ChooseSevenSwapTarget {
        target: PlayerId,
    },
    GiveSwapOneCard {
        card: UnoCard,
    },
    ForceTradeHands {
        first: PlayerId,
        second: PlayerId,
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
    UpdateRules { rules: QiGuiRuleSet },
    PlayCards { cards: Vec<QiGuiCard> },
    SetDeveloperHand { cards: Vec<QiGuiCard> },
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
