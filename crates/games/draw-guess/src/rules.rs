use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DrawGuessRuleSet {
    pub rounds: u8,
    pub drawing_seconds: u16,
}

impl DrawGuessRuleSet {
    pub const MIN_PLAYERS: u8 = 2;
    pub const MAX_PLAYERS: u8 = 8;
    pub const CHOICE_SECONDS: u16 = 15;
    pub const REVEAL_SECONDS: u16 = 5;

    pub fn validate(self) -> Result<Self, DrawGuessError> {
        if !(1..=5).contains(&self.rounds) || !(30..=180).contains(&self.drawing_seconds) {
            return Err(DrawGuessError::InvalidRules);
        }
        Ok(self)
    }
}

impl Default for DrawGuessRuleSet {
    fn default() -> Self {
        Self {
            rounds: 2,
            drawing_seconds: 90,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DrawGuessError {
    InvalidRules,
    InvalidPlayers,
    InvalidWords,
    WrongPhase,
    NotArtist,
    CannotGuess,
    InvalidGuess,
    GuessTooSoon,
    InvalidDrawing,
    DrawingLimit,
    StrokeInProgress,
    NoStroke,
    NothingToUndo,
    NothingToRedo,
    InvalidWordLibrary,
    WordLibraryOfferPending,
    WordLibraryOfferUnavailable,
}

impl fmt::Display for DrawGuessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidWordLibrary => {
                "词库标题最多 40 字，内容最多 1024 个词且不超过 64 KiB，每词最多 20 字"
            }
            Self::WordLibraryOfferPending => "请等待房主处理上一次上传",
            Self::WordLibraryOfferUnavailable => "该词库上传已失效",
            Self::InvalidRules => "无效的你画我猜规则",
            Self::InvalidPlayers => "需要 2 至 8 名玩家",
            Self::InvalidWords => "词库至少需要三个不同的词",
            Self::WrongPhase => "当前阶段无法进行此操作",
            Self::NotArtist => "只有本轮画家可以绘画",
            Self::CannotGuess => "当前无法猜词",
            Self::InvalidGuess => "发言不能为空或超过 20 字",
            Self::GuessTooSoon => "发言太快了",
            Self::InvalidDrawing => "无效的绘图操作",
            Self::DrawingLimit => "本轮绘图已达到上限",
            Self::StrokeInProgress => "请先结束当前笔画",
            Self::NoStroke => "当前没有这条笔画",
            Self::NothingToUndo => "没有可撤销的操作",
            Self::NothingToRedo => "没有可重做的操作",
        })
    }
}

impl Error for DrawGuessError {}
