use super::from_core_player;
use leocard_mahjong::{
    GameState, MahjongPlayerId, MahjongTile, MahjongWinAvailability, Phase, WinSource,
};
use leocard_protocol::MahjongEvent;

#[derive(Clone, Debug, Default)]
pub(super) struct MahjongWinFeedback {
    last_draw: Option<(MahjongPlayerId, MahjongTile)>,
}
impl MahjongWinFeedback {
    pub(super) fn after_action(&mut self, game: &GameState, events: &mut Vec<MahjongEvent>) {
        let draw = matches!(game.phase(), Phase::Playing)
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
                player: from_core_player(player),
                points_without_flowers: points,
            });
        }
    }

    pub(super) fn before_discard(
        game: &GameState,
        event: Option<&MahjongEvent>,
    ) -> Vec<MahjongEvent> {
        let Some(MahjongEvent::TileDiscarded {
            player: source,
            tile,
        }) = event
        else {
            return Vec::new();
        };
        let source = MahjongPlayerId(usize::from(source.0));
        (0..4)
            .map(MahjongPlayerId)
            .filter(|player| *player != source)
            .filter_map(|player| {
                let MahjongWinAvailability::InsufficientFan { points } = game
                    .claim_win_availability(player, tile.kind(), WinSource::Discard(source))
                    .ok()?
                else {
                    return None;
                };
                Some(MahjongEvent::WinUnavailable {
                    player: from_core_player(player),
                    points_without_flowers: points,
                })
            })
            .collect()
    }
}
