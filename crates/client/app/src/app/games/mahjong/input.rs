use super::{MahjongUiAction, MahjongUiState};
use crate::app::runtime::{AppearancePreferences, ClientResource};
use crate::app::shell::{
    ConfirmationDialog, ModalInputScope, PageMotion, PressedUiAction, UiAction, UiState,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{GameSnapshot, MahjongPhaseView, MahjongSnapshot};

#[derive(SystemParam)]
pub(super) struct MahjongShortcutInput<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    preferences: Res<'w, AppearancePreferences>,
    ui: Res<'w, UiState>,
    mahjong: Res<'w, MahjongUiState>,
    modal_scope: ModalInputScope<'w, 's>,
    confirmation: Res<'w, ConfirmationDialog>,
    page: Res<'w, PageMotion>,
}

pub(super) fn handle_right_click_draw_discard(
    input: MahjongShortcutInput,
    client: Option<Res<ClientResource>>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if !input.preferences.controls.mahjong_right_click_draw_discard
        || !input.mouse.just_pressed(MouseButton::Right)
        || input.page.active()
        || input.confirmation.is_busy()
        || input.modal_scope.top().is_some()
        || input.ui.settings.open
        || input.ui.profile.open
        || input.ui.shop.open
        || input.ui.achievements.open
        || input.ui.leaving_room
        || input.mahjong.fan_guide_open
        || input
            .windows
            .single()
            .ok()
            .is_none_or(|window| !window.focused || window.cursor_position().is_none())
    {
        return;
    }
    let Some(model) = client.as_deref().map(|client| client.0.model()) else {
        return;
    };
    let Some(game) = model.game_snapshot().and_then(GameSnapshot::mahjong) else {
        return;
    };
    if model.you() != Some(game.you) {
        return;
    }
    if let Some(tile) = drawn_tile_for_shortcut(game, &input.mahjong) {
        actions.write(PressedUiAction(UiAction::Mahjong(
            MahjongUiAction::DiscardDrawnTile(tile),
        )));
    }
}

fn drawn_tile_for_shortcut(game: &MahjongSnapshot, ui: &MahjongUiState) -> Option<MahjongTile> {
    if !matches!(game.phase, MahjongPhaseView::Playing)
        || game.current_player != game.you
        || game.pending_claim.is_some()
        || ui.declaring_ready
        || (ui.auto_win && !game.rules.false_win && game.can_legal_self_draw)
        || !game
            .players
            .iter()
            .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
    {
        return None;
    }
    game.your_drawn_tile
        .filter(|tile| game.your_hand.contains(tile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_game_common::PlayerId;
    use leocard_mahjong::{MahjongWallBreak, MahjongWind, build_deck};
    use leocard_protocol::{MahjongPlayerView, MatchId, PlayerPublicInfo, ProfileId, SeatId};

    fn snapshot() -> MahjongSnapshot {
        let hand = build_deck()[..14].to_vec();
        let you = PlayerId(0);
        MahjongSnapshot {
            match_id: MatchId([1; 16]),
            host_port: 52300,
            you,
            host: you,
            rules: default(),
            players: vec![MahjongPlayerView {
                player: PlayerPublicInfo {
                    id: you,
                    profile_id: ProfileId([0; 32]),
                    name: "玩家".to_owned(),
                    avatar: None,
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles: default(),
                },
                seat: SeatId(0),
                seat_wind: MahjongWind::East,
                concealed_count: 14,
                revealed_hand: None,
                melds: Vec::new(),
                flowers: Vec::new(),
                dead_hand: false,
                declared_ready: false,
                auto_play: false,
                ready: false,
                connected: true,
            }],
            your_drawn_tile: hand.last().copied(),
            your_hand: hand,
            your_auto_flower: true,
            discards: Vec::new(),
            dealer: you,
            prevalent_wind: MahjongWind::East,
            sequence_index: 0,
            current_player: you,
            wall_len: 100,
            wall_break: MahjongWallBreak::new([1, 1]).unwrap(),
            wall_tail_drawn: 0,
            match_scores: [0; 4],
            pending_claim: None,
            can_self_draw: false,
            can_legal_self_draw: false,
            concealed_kong_options: Vec::new(),
            added_kong_options: Vec::new(),
            ready_discards: Vec::new(),
            phase: MahjongPhaseView::Playing,
        }
    }

    #[test]
    fn shortcut_discards_only_the_actual_draw_including_after_declaring_ready() {
        let mut game = snapshot();
        let ui = MahjongUiState::default();
        assert_eq!(drawn_tile_for_shortcut(&game, &ui), game.your_drawn_tile);
        game.players[0].declared_ready = true;
        assert_eq!(drawn_tile_for_shortcut(&game, &ui), game.your_drawn_tile);
        game.your_hand.pop();
        assert!(drawn_tile_for_shortcut(&game, &ui).is_none());
        game.your_drawn_tile = None;
        assert!(drawn_tile_for_shortcut(&game, &ui).is_none());
    }

    #[test]
    fn shortcut_respects_turn_control_and_automatic_win_priority() {
        let game = snapshot();
        let mut ui = MahjongUiState::default();
        let mut other_turn = game.clone();
        other_turn.current_player = PlayerId(1);
        assert!(drawn_tile_for_shortcut(&other_turn, &ui).is_none());
        let mut dealing = game.clone();
        dealing.phase = MahjongPhaseView::Dealing { batch: 0 };
        assert!(drawn_tile_for_shortcut(&dealing, &ui).is_none());
        let mut automated = game.clone();
        automated.players[0].auto_play = true;
        assert!(drawn_tile_for_shortcut(&automated, &ui).is_none());
        let mut winning = game;
        winning.can_legal_self_draw = true;
        ui.auto_win = true;
        assert!(drawn_tile_for_shortcut(&winning, &ui).is_none());
        ui.auto_win = false;
        assert_eq!(
            drawn_tile_for_shortcut(&winning, &ui),
            winning.your_drawn_tile
        );
    }
}
