use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId, UnoPhaseView, UnoSnapshot};
use leocard_uno::UnoCard;

#[derive(Resource, Default)]
pub(crate) struct UnoJumpInDevice {
    pub enabled: bool,
    pub drawer_open: bool,
    pub available: bool,
    pub(super) last_attempt: Option<JumpInOpportunity>,
}

/// 使用牌局状态识别抢出机会，聊天等消息引起的版本变化不会再次触发。
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct JumpInOpportunity {
    match_id: MatchId,
    player: PlayerId,
    current_player: PlayerId,
    discard: UnoCard,
    pub card: UnoCard,
}

impl JumpInOpportunity {
    pub fn from_snapshot(game: &UnoSnapshot) -> Option<Self> {
        if !matches!(game.phase, UnoPhaseView::Playing) {
            return None;
        }
        let own = game.players.iter().find(|player| player.id == game.you)?;
        if own.auto_play || own.eliminated || !own.connected {
            return None;
        }
        Some(Self {
            match_id: game.match_id,
            player: game.you,
            current_player: game.current_player?,
            discard: game.discard_top,
            card: game.your_jump_in_card?,
        })
    }
}
