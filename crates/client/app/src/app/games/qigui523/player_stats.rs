use crate::app::presentation::{SeatSide, add_hand_count_score};
use crate::app::runtime::UiAssets;
use crate::app::shell::PlayerGameScoreText;
use leocard_game_common::PlayerId;

use bevy::prelude::*;

use leocard_ui::UiComposer;

pub(super) fn add_qigui_player_stats(
    commands: &mut UiComposer,
    portrait: Entity,
    player: PlayerId,
    side: SeatSide,
    score: u32,
    hand_len: u16,
    assets: &UiAssets,
) {
    let value = add_hand_count_score(commands, portrait, hand_len, i64::from(score), assets);
    commands
        .entity(value)
        .insert(PlayerGameScoreText::Opponent { player, side });
}
