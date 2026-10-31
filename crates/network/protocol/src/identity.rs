use leocard_achievements::TrophyCounts;
use leocard_blackjack::BlackjackProfileStats;
use leocard_doudizhu::DoudizhuProfileStats;
use leocard_game_common::PlayerId;
use leocard_mahjong::MahjongProfileStats;
use leocard_mc_guess::McGuessProfileStats;
use leocard_qigui523::QiGui523ProfileStats;
use leocard_shengji::ShengjiProfileStats;
use leocard_texas_holdem::TexasProfileStats;
use leocard_uno::UnoProfileStats;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RoomId(pub u64);

/// 跨房间稳定的本地玩家身份；其字节同时是 Ed25519 验证公钥。
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProfileId(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MatchId(pub [u8; 16]);

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlayerGender {
    #[default]
    Male,
    Female,
}

/// 可随玩家身份公开的各游戏长期档案。`None` 表示旧版本没有采集过该游戏的
/// 明细，展示层必须与真实的零次记录区分开来。
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerGameProfiles {
    /// 仅公开各等级已解锁成就数量，不公开成就条目及日期。
    pub achievements: TrophyCounts,
    /// 由本地钱包公开的金币余额，不随游戏档案存档。
    pub coins: u32,
    /// Public profile attribute shared with other players and future voice lines.
    pub gender: PlayerGender,
    pub qigui523: Option<QiGui523ProfileStats>,
    pub texas: Option<TexasProfileStats>,
    pub shengji: Option<ShengjiProfileStats>,
    pub uno: Option<UnoProfileStats>,
    /// 玩家收到互动时累计的鲜花与鸡蛋数量。
    pub interactions: Option<PlayerInteractionStats>,
    pub mahjong: Option<MahjongProfileStats>,
    pub mc_guess: Option<McGuessProfileStats>,
    pub blackjack: Option<BlackjackProfileStats>,
    pub doudizhu: Option<DoudizhuProfileStats>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerInteractionStats {
    pub flowers_received: u32,
    pub eggs_received: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SeatId(pub u8);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PlayerInteractionKind {
    Flower,
    Egg,
    Wine,
    Shoe,
}

impl PlayerInteractionKind {
    pub const fn flower_count(self) -> u32 {
        match self {
            Self::Flower => 1,
            Self::Wine => 10,
            _ => 0,
        }
    }

    pub const fn egg_count(self) -> u32 {
        match self {
            Self::Egg => 1,
            Self::Shoe => 10,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerInteraction {
    pub source: PlayerId,
    pub target: PlayerId,
    pub kind: PlayerInteractionKind,
    /// 由房主生成，使所有客户端选择相同音效和命中偏移。
    pub seed: u32,
    /// 房主记录的房间存续时间；互动窗口不依赖客户端收包时间。
    pub elapsed_millis: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ChatContent {
    Text(String),
    QuickVoice(u8),
    Emoji(ChatEmoji),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum ChatEmoji {
    Laugh,
    Angry,
    Surprised,
    Pleading,
    Party,
    Heart,
    Grinning,
    RollingLaugh,
    Smile,
    Wink,
    HeartEyes,
    HeartsFace,
    Kiss,
    Sunglasses,
    StarStruck,
    Cry,
    LoudCry,
    AngryHorns,
    Flushed,
    Thinking,
    RollingEyes,
    Unamused,
    Expressionless,
    Tongue,
    Fearful,
    Fire,
    SparklingHeart,
    ThumbsUp,
    Clap,
    Hundred,
    TiebaSmile,
    TiebaGrinning,
    TiebaTongue,
    TiebaAstonished,
    TiebaSunglasses,
    TiebaAngry,
    TiebaSmileEyes,
    TiebaSweat,
    TiebaCry,
    TiebaAwkward,
    TiebaUnamused,
    TiebaDisdain,
    TiebaRage,
    TiebaMoney,
    TiebaKiss,
    TiebaRedFace,
    TiebaVomit,
    TiebaEyeRoll,
    TiebaAggrieved,
    TiebaHeartEyes,
    TiebaPuzzled,
    TiebaLaugh,
    TiebaSmirk,
    TiebaCoverEyes,
    TiebaFunny,
    TiebaShady,
    TiebaTears,
    TiebaPleading,
    TiebaSleep,
    TiebaStartled,
    TiebaFurious,
    TiebaShock,
    TiebaSpit,
    TiebaHeart,
    TiebaBrokenHeart,
    TiebaRose,
    TiebaGift,
    TiebaRainbow,
    TiebaMoon,
    TiebaSun,
    TiebaCoin,
    TiebaLightbulb,
    TiebaCoffee,
    TiebaCake,
    TiebaMusic,
    TiebaRock,
    TiebaPeace,
    TiebaThumbsUp,
    TiebaThumbsDown,
    TiebaOk,
}

impl ChatEmoji {
    pub const ALL: [Self; 80] = [
        Self::TiebaSmile,
        Self::TiebaGrinning,
        Self::TiebaTongue,
        Self::TiebaAstonished,
        Self::TiebaSunglasses,
        Self::TiebaAngry,
        Self::TiebaSmileEyes,
        Self::TiebaSweat,
        Self::TiebaCry,
        Self::TiebaAwkward,
        Self::TiebaUnamused,
        Self::TiebaDisdain,
        Self::TiebaRage,
        Self::TiebaMoney,
        Self::TiebaKiss,
        Self::TiebaRedFace,
        Self::TiebaVomit,
        Self::TiebaEyeRoll,
        Self::TiebaAggrieved,
        Self::TiebaHeartEyes,
        Self::TiebaPuzzled,
        Self::TiebaLaugh,
        Self::TiebaSmirk,
        Self::TiebaCoverEyes,
        Self::TiebaFunny,
        Self::TiebaShady,
        Self::TiebaTears,
        Self::TiebaPleading,
        Self::TiebaSleep,
        Self::TiebaStartled,
        Self::TiebaFurious,
        Self::TiebaShock,
        Self::TiebaSpit,
        Self::TiebaHeart,
        Self::TiebaBrokenHeart,
        Self::TiebaRose,
        Self::TiebaGift,
        Self::TiebaRainbow,
        Self::TiebaMoon,
        Self::TiebaSun,
        Self::TiebaCoin,
        Self::TiebaLightbulb,
        Self::TiebaCoffee,
        Self::TiebaCake,
        Self::TiebaMusic,
        Self::TiebaRock,
        Self::TiebaPeace,
        Self::TiebaThumbsUp,
        Self::TiebaThumbsDown,
        Self::TiebaOk,
        Self::Grinning,
        Self::Laugh,
        Self::RollingLaugh,
        Self::Smile,
        Self::Wink,
        Self::HeartEyes,
        Self::HeartsFace,
        Self::Kiss,
        Self::Sunglasses,
        Self::StarStruck,
        Self::Party,
        Self::Cry,
        Self::LoudCry,
        Self::Pleading,
        Self::Angry,
        Self::AngryHorns,
        Self::Surprised,
        Self::Flushed,
        Self::Thinking,
        Self::RollingEyes,
        Self::Unamused,
        Self::Expressionless,
        Self::Tongue,
        Self::Fearful,
        Self::Fire,
        Self::Heart,
        Self::SparklingHeart,
        Self::ThumbsUp,
        Self::Clap,
        Self::Hundred,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }
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
