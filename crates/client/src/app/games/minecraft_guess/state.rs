use bevy::prelude::*;
use leocard_minecraft_guess::GameError;
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub(crate) struct MinecraftGuessUiState {
    pub input: String,
    pub mode_menu_open: bool,
    pub scroll_y: f32,
    pub scroll_to_latest: bool,
    pub match_id: Option<MatchId>,
    pub game_index: u8,
    pub guess_count: usize,
}

#[derive(Component)]
pub(super) struct GuessInput;

#[derive(Component)]
pub(super) struct GuessScroll;

pub(crate) fn violation_label(error: GameError) -> &'static str {
    match error {
        GameError::NotPlayersTurn => "还没有轮到你猜测",
        GameError::CannotSkip => "抢答模式不能跳过",
        GameError::Finished => "本轮竞猜已经结束",
        GameError::UnknownEntry => "没有找到这个名称，请输入完整的方块或物品名称",
        GameError::AlreadyGuessed => "此条目已猜过",
        GameError::InvalidGuess => "请输入有效的方块或物品名称",
        GameError::GuessTooLong => "名称最多输入 10 个字",
    }
}
