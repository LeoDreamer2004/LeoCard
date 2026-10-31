use super::super::uno_card_is_playable;
use bevy::picking::pointer::PointerId;
use leocard_game_common::PlayerId;
use leocard_protocol::{MatchId, UnoPhaseView, UnoSnapshot};
use leocard_uno::UnoCard;

use bevy::prelude::*;

#[derive(Component)]
pub(in super::super) struct UnoDraggableCard(pub UnoCard);

#[derive(Resource, Default)]
pub(in super::super) struct UnoDragState {
    pub(super) gesture: Option<UnoDragGesture>,
}

pub(super) struct UnoDragGesture {
    pub card: UnoCard,
    pub match_id: MatchId,
    pub discard: UnoCard,
    pub current_player: Option<PlayerId>,
    pub pointer: PointerId,
    pub window: Entity,
    pub scale: f32,
    pub press: Vec2,
    pub cursor: Vec2,
    pub origin: Vec2,
    pub target: Vec2,
    pub dragging: bool,
}

impl UnoDragGesture {
    pub fn valid(&self, game: &UnoSnapshot) -> bool {
        self.match_id == game.match_id
            && self.discard == game.discard_top
            && self.current_player == game.current_player
            && Self::can_drag(game, self.card)
    }

    pub fn can_drag(game: &UnoSnapshot, card: UnoCard) -> bool {
        matches!(game.phase, UnoPhaseView::Playing)
            && game.your_hand.contains(&card)
            && game.players.iter().any(|player| {
                player.player.id == game.you
                    && player.connected
                    && !player.auto_play
                    && !player.eliminated
            })
            && (uno_card_is_playable(game, card) || game.your_jump_in_card == Some(card))
    }

    pub fn ready(&self) -> bool {
        ((self.cursor - self.target) / Self::target_radius()).length_squared() <= 1.0
    }

    pub fn target_radius() -> Vec2 {
        Vec2::new(220.0, 165.0)
    }
}
