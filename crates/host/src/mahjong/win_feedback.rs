use leocard_game_common::PlayerId;
use leocard_mahjong::{
    MahjongGameState, MahjongPhase, MahjongTile, MahjongWinAvailability, MahjongWinSource,
};
use leocard_protocol::MahjongEvent;

#[derive(Clone, Debug, Default)]
pub(super) struct MahjongWinFeedback {
    last_draw: Option<(PlayerId, MahjongTile)>,
}
impl MahjongWinFeedback {
    pub(super) fn after_action(&mut self, game: &MahjongGameState, events: &mut Vec<MahjongEvent>) {
        let draw = matches!(game.phase(), MahjongPhase::Playing)
            .then(|| game.last_drawn().map(|tile| (game.current_player(), tile)))
            .flatten();
        if draw == self.last_draw {
            return;
        }
        self.last_draw = draw;
        if let Some((player, _)) = draw
            && let Ok(MahjongWinAvailability::InsufficientFan { points }) =
                game.self_draw_win_availability(player)
        {
            events.push(MahjongEvent::WinUnavailable {
                player,
                points_without_flowers: points,
            });
        }
    }

    pub(super) fn before_discard(
        game: &MahjongGameState,
        event: Option<&MahjongEvent>,
    ) -> Vec<MahjongEvent> {
        let Some(MahjongEvent::TileDiscarded {
            player: source,
            tile,
        }) = event
        else {
            return Vec::new();
        };
        (0..4)
            .map(PlayerId)
            .filter(|player| *player != *source)
            .filter_map(|player| {
                let MahjongWinAvailability::InsufficientFan { points } = game
                    .claim_win_availability(player, tile.kind(), MahjongWinSource::Discard(*source))
                    .ok()?
                else {
                    return None;
                };
                Some(MahjongEvent::WinUnavailable {
                    player,
                    points_without_flowers: points,
                })
            })
            .collect()
    }
}
