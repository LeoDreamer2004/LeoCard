//! UNO 牌桌交互、飞牌与规则特效的状态类型。

use crate::app::presentation::CardAnimationState;
use crate::app::shell::SocialUiState;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, UnoEvent, UnoPendingSwapView, UnoSnapshot};
use leocard_uno::UnoCard;
use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};

#[derive(Resource, Default)]
pub(crate) struct UnoUiState {
    pub selected: HashSet<UnoCard>,
    pub swap_targets: Vec<PlayerId>,
    pub card_animations: HashMap<UnoCard, CardAnimationState>,
    pub mode_menu_open: bool,
    pub expansion_settings_open: bool,
    pub color_choice: Option<UnoCard>,
}

impl UnoUiState {
    pub(crate) fn reconcile(&mut self, game: &UnoSnapshot, social: &mut SocialUiState) {
        self.selected.retain(|card| game.your_hand.contains(card));
        if let Some(card) = game.your_jump_in_card {
            self.selected.clear();
            self.selected.insert(card);
        } else if game.current_player != Some(game.you) {
            self.selected.clear();
        }
        self.card_animations
            .retain(|card, _| game.your_hand.contains(card));
        let selecting_targets = matches!(
            game.pending_swap,
            Some(UnoPendingSwapView::SwapOneTarget { player })
                | Some(UnoPendingSwapView::ForceTrade { player })
                | Some(UnoPendingSwapView::SevenSwap { player }) if player == game.you
        );
        if selecting_targets {
            social.interaction_menu_open = None;
            self.swap_targets.retain(|target| {
                game.players
                    .iter()
                    .any(|player| player.id == *target && !player.eliminated)
            });
        } else {
            self.swap_targets.clear();
        }
        if self
            .color_choice
            .is_some_and(|card| !game.your_hand.contains(&card))
        {
            self.color_choice = None;
        }
    }

    pub(crate) fn clear(&mut self) {
        self.selected.clear();
        self.card_animations.clear();
        self.swap_targets.clear();
        self.color_choice = None;
        self.mode_menu_open = false;
        self.expansion_settings_open = false;
    }
}

#[derive(Component)]
pub(super) struct UnoSwapTargetPanel {
    pub selected: bool,
}

#[derive(Component)]
pub(crate) struct UnoExpansionStatus;

#[derive(Component)]
pub(crate) struct UnoExpansionStatusFrame;

#[derive(Component)]
pub(crate) struct UnoHandCardVisual {
    pub button: Entity,
    pub card: UnoCard,
    pub selected: bool,
    pub hover_amount: f32,
    pub selected_amount: f32,
}

#[derive(Component)]
pub(crate) struct UnoHandCardButton;

#[derive(Component)]
pub(super) struct UnoExtensionCardHelp {
    pub title: &'static str,
    pub description: &'static str,
}

#[derive(Component)]
pub(super) struct UnoExtensionCardHelpOverlay {
    pub title: Entity,
    pub description: Entity,
}

#[derive(Resource, Default)]
pub(crate) struct UnoPresentationState {
    pub events: VecDeque<UnoEvent>,
    pub last_snapshot: Option<UnoSnapshot>,
}

#[derive(Component)]
pub(crate) struct UnoDrawPileAnchor;

#[derive(Component)]
pub(crate) struct UnoDiscardPileAnchor;

#[derive(Component)]
pub(crate) struct UnoDiscardCard(pub UnoCard);

#[derive(Component, Clone, Copy)]
pub(crate) enum UnoFlipTarget {
    Own(usize),
    Opponent { player: PlayerId, index: usize },
    DrawPile(usize),
    DiscardPile(usize),
}

#[derive(Component)]
pub(crate) struct UnoFlyingCard {
    pub elapsed: f32,
    pub delay: f32,
    pub start: Vec2,
    pub staging: Vec2,
    pub control: Vec2,
    pub target: Vec2,
    pub duration: f32,
    pub draw_animation: bool,
    pub played_card: Option<UnoCard>,
    pub start_angle: f32,
    pub end_angle: f32,
}

#[derive(Component)]
pub(crate) struct UnoFlipOverlay {
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct UnoFlipCard {
    pub elapsed: f32,
    pub delay: f32,
    pub old_face: Handle<Image>,
    pub new_face: Handle<Image>,
    pub swapped: bool,
    pub base_transform: UiTransform,
    pub pile: bool,
}

#[derive(Component)]
pub(crate) struct UnoPaletteEffect {
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct UnoPaletteSelectedSector {
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct UnoPaletteColorRing {
    pub elapsed: f32,
    pub delay: f32,
    pub color: Color,
    pub start_scale: f32,
    pub end_scale: f32,
    pub max_alpha: f32,
}

#[derive(Component)]
pub(crate) struct UnoPaletteParticle {
    pub elapsed: f32,
    pub delay: f32,
    pub origin: Vec2,
    pub direction: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub rotation: f32,
}

#[derive(Component)]
pub(crate) struct UnoReverseArrow {
    pub elapsed: f32,
    pub delay: f32,
    pub color: Color,
    pub max_alpha: f32,
    pub shadow_alpha: f32,
}

#[derive(Component)]
pub(crate) struct UnoModeDropdownPanel;
