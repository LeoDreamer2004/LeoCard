//! 顶层页面导航状态。

use super::*;
use leocard_protocol::{GameKind, GameSnapshot, PlayerId, UnoPendingSwapView};

#[derive(Default)]
pub struct NavigationUiState {
    pub settings_open: bool,
    pub profile_open: bool,
    pub player_profile: Option<PlayerProfilePage>,
    pub profile_game_tab: ProfileGameTab,
    pub host_game_picker_open: bool,
}

impl UiState {
    pub(super) fn reconcile_screen_state(&mut self, client: Option<&ClientResource>) {
        let model = client.map(|client| client.0.model());
        let in_uno_lobby = model
            .as_ref()
            .and_then(|model| model.lobby())
            .is_some_and(|lobby| lobby.game == GameKind::Uno);
        if !in_uno_lobby {
            self.uno.mode_menu_open = false;
            self.uno.expansion_settings_open = false;
        }

        match model.as_ref().and_then(|model| model.game_snapshot()) {
            Some(GameSnapshot::QiGui523(game)) => {
                self.qigui523
                    .selected
                    .retain(|card| game.your_hand.contains(card));
                self.qigui523
                    .card_animations
                    .retain(|card, _| game.your_hand.contains(card));
                self.retain_interaction_target(|target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            Some(GameSnapshot::TexasHoldem(game)) => {
                self.clear_qigui523_hand_state();
                self.retain_interaction_target(|target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            Some(GameSnapshot::Shengji(game)) => {
                self.clear_qigui523_hand_state();
                self.shengji
                    .selected
                    .retain(|card| game.your_hand.contains(card));
                self.retain_interaction_target(|target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            Some(GameSnapshot::Uno(game)) => {
                self.clear_qigui523_hand_state();
                self.clear_shengji_hand_state();
                self.reconcile_uno_state(game);
                self.retain_interaction_target(|target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            Some(GameSnapshot::Mahjong(game)) => {
                self.clear_qigui523_hand_state();
                self.clear_shengji_hand_state();
                self.clear_uno_hand_state();
                self.retain_interaction_target(|target| {
                    game.players.iter().any(|player| player.id == target)
                });
            }
            None => self.reset_outside_game(),
        }
    }

    fn reconcile_uno_state(&mut self, game: &leocard_protocol::UnoSnapshot) {
        self.uno
            .selected
            .retain(|card| game.your_hand.contains(card));
        if let Some(card) = game.your_jump_in_card {
            self.uno.selected.clear();
            self.uno.selected.insert(card);
        } else if game.current_player != Some(game.you) {
            self.uno.selected.clear();
        }
        self.uno
            .card_animations
            .retain(|card, _| game.your_hand.contains(card));

        let selecting_swap_targets = matches!(
            game.pending_swap,
            Some(UnoPendingSwapView::SwapOneTarget { player })
                | Some(UnoPendingSwapView::ForceTrade { player })
                | Some(UnoPendingSwapView::SevenSwap { player }) if player == game.you
        );
        if selecting_swap_targets {
            self.social.interaction_menu_open = None;
            self.uno.swap_targets.retain(|target| {
                game.players
                    .iter()
                    .any(|player| player.id == *target && !player.eliminated)
            });
        } else {
            self.uno.swap_targets.clear();
        }
        if self
            .uno
            .color_choice
            .is_some_and(|card| !game.your_hand.contains(&card))
        {
            self.uno.color_choice = None;
        }
    }

    fn retain_interaction_target(&mut self, contains: impl FnOnce(PlayerId) -> bool) {
        if self
            .social
            .interaction_menu_open
            .is_some_and(|target| !contains(target))
        {
            self.social.interaction_menu_open = None;
        }
    }

    fn clear_qigui523_hand_state(&mut self) {
        self.qigui523.selected.clear();
        self.qigui523.card_animations.clear();
        self.qigui523.observed_hand.clear();
        self.qigui523.greedy_hint.reset();
    }

    fn clear_shengji_hand_state(&mut self) {
        self.shengji.selected.clear();
        self.shengji.card_animations.clear();
        self.shengji.observed_hand.clear();
    }

    fn clear_uno_hand_state(&mut self) {
        self.uno.selected.clear();
        self.uno.card_animations.clear();
    }

    fn reset_outside_game(&mut self) {
        self.clear_qigui523_hand_state();
        self.clear_shengji_hand_state();
        self.clear_uno_hand_state();
        self.social.interaction_menu_open = None;
        self.texas_holdem.observed_match = None;
        self.texas_holdem.observed_hand_number = 0;
        self.texas_holdem.observed_community_len = 0;
        self.shengji.observed_match = None;
        self.shengji.observed_hand_number = 0;
        self.shengji.buried_open = false;
        self.uno.swap_targets.clear();
        self.uno.color_choice = None;
    }
}
