use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RoomId(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlayerId(pub u8);

/// 跨房间稳定的本地玩家身份；其字节同时是 Ed25519 验证公钥。
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProfileId(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MatchId(pub [u8; 16]);

/// 可随玩家身份公开的各游戏长期档案。`None` 表示旧版本没有采集过该游戏的
/// 明细，展示层必须与真实的零次记录区分开来。
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerGameProfiles {
    pub qigui523: Option<QiGui523ProfileStats>,
    pub texas_holdem: Option<TexasHoldemProfileStats>,
    pub shengji: Option<ShengjiProfileStats>,
    pub uno: Option<UnoProfileStats>,
    /// 玩家收到互动时累计的鲜花与鸡蛋数量。
    pub interactions: Option<PlayerInteractionStats>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerInteractionStats {
    pub flowers_received: u32,
    pub eggs_received: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct QiGui523ProfileStats {
    pub completed_games: u32,
    pub total_score: u64,
    pub total_reference_delta: i64,
    pub placement_counts: [u32; 6],
    pub straight_plays: u32,
    pub consecutive_pair_plays: u32,
    pub airplane_plays: u32,
    pub bomb_plays: u32,
    pub heaven_bomb_plays: u32,
    pub longest_straight: u16,
    pub longest_consecutive_pairs: u16,
    pub longest_airplane: u16,
}

/// 普通德州扑克与奥马哈共用的长期档案。
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TexasHoldemProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_final_chips: u64,
    pub placement_counts: [u32; 6],
    /// 跟注、加注和全下实际投入的筹码总和；不含盲注。
    pub wagered_chips: u64,
    pub wager_actions: u32,
    /// 不含盲注的弃牌、过牌、跟注、加注及全下动作总数。
    pub voluntary_actions: u32,
    pub check_actions: u32,
    pub raise_actions: u32,
    pub all_in_actions: u32,
    pub hands_played: u32,
    pub hands_folded: u32,
    /// 顺序依次为高牌、一对、两对、三条、顺子、同花、葫芦、四条、
    /// 同花顺及皇家同花顺。
    pub hand_category_counts: [u32; 10],
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShengjiProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub dealer_team_games: u32,
    pub dealer_team_score: u64,
    pub collecting_team_games: u32,
    pub collecting_team_score: u64,
    pub dealer_games: u32,
    pub declaration_games: u32,
    pub counter_games: u32,
    pub defended_kitty_games: u32,
    pub captured_kitty_games: u32,
    pub buried_games: u32,
    pub buried_points: u64,
    pub plays: u32,
    pub winning_plays: u32,
    pub crossing_games: u32,
    /// 拖拉机、泰坦尼克、炸弹、太空堡垒、甩牌。
    pub play_category_counts: [u32; 5],
    pub longest_tractor: u16,
    pub longest_titanic: u16,
    pub longest_space_fortress: u16,
    pub longest_throw: u16,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnoProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_remaining_score: u64,
    pub placement_counts: [u32; 6],
    pub max_hand_cards: u16,
    pub max_penalty_cards: u16,
    pub max_skipped_turns: u16,
    pub uno_calls: u32,
    pub uno_penalties: u32,
    pub challenges: u32,
    pub successful_challenges: u32,
    pub challenges_received: u32,
    /// 别的玩家成功质疑自己的次数。
    pub successful_challenges_received: u32,
    /// 玩家实际收到抢出候选牌的次数，而非发送抢出命令的次数。
    #[serde(alias = "jump_in_attempts")]
    pub jump_in_opportunities: u32,
    pub successful_jump_ins: u32,
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
}

impl ChatEmoji {
    pub const ALL: [Self; 30] = [
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
