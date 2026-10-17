use bevy::prelude::*;
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
