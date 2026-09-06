//! UNO 牌桌交互、飞牌与规则特效的状态类型。

use super::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct UnoUiState {
    pub selected: HashSet<UnoCard>,
    pub swap_targets: Vec<PlayerId>,
    pub card_animations: HashMap<UnoCard, CardAnimationState>,
    pub mode_menu_open: bool,
    pub expansion_settings_open: bool,
    pub color_choice: Option<UnoCard>,
}

#[derive(Component)]
pub struct UnoSwapTargetPanel {
    pub selected: bool,
}

#[derive(Component)]
pub struct UnoExpansionStatus;

#[derive(Component)]
pub struct UnoExpansionStatusFrame;

#[derive(Component)]
pub struct UnoHandCardVisual {
    pub button: Entity,
    pub card: UnoCard,
    pub selected: bool,
    pub hover_amount: f32,
    pub selected_amount: f32,
}

#[derive(Component)]
pub struct UnoHandCardButton;

#[derive(Component)]
pub struct UnoExtensionCardHelp {
    pub title: &'static str,
    pub description: &'static str,
}

#[derive(Component)]
pub struct UnoExtensionCardHelpOverlay {
    pub title: Entity,
    pub description: Entity,
}

#[derive(Resource, Default)]
pub struct UnoPresentationState {
    pub events: VecDeque<UnoEvent>,
    pub last_snapshot: Option<UnoSnapshot>,
}

#[derive(Component)]
pub struct UnoDrawPileAnchor;

#[derive(Component)]
pub struct UnoDiscardPileAnchor;

#[derive(Component)]
pub struct UnoDiscardCard(pub UnoCard);

#[derive(Component, Clone, Copy)]
pub enum UnoFlipTarget {
    Own(usize),
    Opponent { player: PlayerId, index: usize },
    DrawPile(usize),
    DiscardPile(usize),
}

#[derive(Component)]
pub struct UnoFlyingCard {
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
pub struct UnoFlipOverlay {
    pub elapsed: f32,
}

#[derive(Component)]
pub struct UnoFlipCard {
    pub elapsed: f32,
    pub delay: f32,
    pub old_face: Handle<Image>,
    pub new_face: Handle<Image>,
    pub swapped: bool,
    pub base_transform: UiTransform,
    pub pile: bool,
}

#[derive(Component)]
pub struct UnoPaletteEffect {
    pub elapsed: f32,
}

#[derive(Component)]
pub struct UnoPaletteSelectedSector {
    pub elapsed: f32,
}

#[derive(Component)]
pub struct UnoPaletteColorRing {
    pub elapsed: f32,
    pub delay: f32,
    pub color: Color,
    pub start_scale: f32,
    pub end_scale: f32,
    pub max_alpha: f32,
}

#[derive(Component)]
pub struct UnoPaletteParticle {
    pub elapsed: f32,
    pub delay: f32,
    pub origin: Vec2,
    pub direction: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub rotation: f32,
}

#[derive(Component)]
pub struct UnoReverseArrow {
    pub elapsed: f32,
    pub delay: f32,
    pub color: Color,
    pub max_alpha: f32,
    pub shadow_alpha: f32,
}

#[derive(Component)]
pub struct UnoModeDropdownPanel;
