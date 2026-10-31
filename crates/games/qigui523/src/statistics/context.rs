use crate::{QiGuiCard, QiGuiGameState, QiGuiPhase, QiGuiTrickState};
use leocard_game_common::PlayerId;

/// Capture before an action and submit only when that action succeeds.
/// Hands never leave this component; reports expose only derived facts.
pub struct QiGuiActionContext {
    pub(super) actor: PlayerId,
    pub(super) hands: Vec<Vec<QiGuiCard>>,
    pub(super) scores: Vec<u32>,
    pub(super) draw_pile_empty: bool,
    pub(super) trick: QiGuiTrickState,
}

impl QiGuiActionContext {
    pub fn capture(game: &QiGuiGameState, actor: PlayerId) -> Option<Self> {
        if !matches!(game.phase(), QiGuiPhase::Playing) || game.player(actor).is_none() {
            return None;
        }
        Some(Self {
            actor,
            hands: game.players().iter().map(|p| p.hand().to_vec()).collect(),
            scores: game.players().iter().map(|p| p.score()).collect(),
            draw_pile_empty: game.draw_pile_len() == 0,
            trick: game.trick()?.clone(),
        })
    }
}
