//! 手牌悬停、拖选和预览输入。

use super::{DESIGN_WIDTH, HAND_CARD_REVEAL};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use std::collections::HashSet;
use std::hash::Hash;

const HAND_CARD_SELECTED_LIFT: f32 = 24.0;

#[derive(Resource, Default)]
pub(crate) struct CardDragSelection {
    pub active: bool,
    pub anchor: usize,
    pub current: usize,
    pub select: bool,
}

impl CardDragSelection {
    pub(crate) fn contains(&self, index: usize) -> bool {
        self.active
            && (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index)
    }
}

#[derive(Component)]
pub(crate) struct HandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum CardSize {
    Hand,
    Seat,
    Score,
    TableScore,
    FinishedHand,
}

impl CardSize {
    pub(crate) fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Hand => (76.0, 103.0),
            Self::Seat => (72.0, 98.0),
            Self::Score => (36.0, 49.0),
            Self::TableScore => (28.0, 38.0),
            Self::FinishedHand => (43.2, 58.8),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HandCardPose {
    pub translation: Val2,
    pub rotation: Rot2,
}

pub(crate) fn slot_hover_target(
    drag: &CardDragSelection,
    index: usize,
    interaction: Interaction,
    cursor: &RelativeCursorPosition,
) -> f32 {
    f32::from(if drag.active {
        cursor.cursor_over() || index == drag.current
    } else {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    })
}

pub(crate) fn advance_towards(current: f32, target: f32, response: f32) -> f32 {
    let next = current + (target - current) * response;
    if (target - next).abs() < 0.001 {
        target
    } else {
        next
    }
}

pub(crate) fn update_drag_selection<C>(
    mouse: &ButtonInput<MouseButton>,
    drag: &mut CardDragSelection,
    selected: &mut HashSet<C>,
    pressed: Option<(usize, C)>,
    hovered: Option<usize>,
    cards: impl Iterator<Item = (usize, C)>,
) -> bool
where
    C: Copy + Eq + Hash,
{
    if mouse.just_pressed(MouseButton::Left)
        && let Some((index, card)) = pressed
    {
        drag.active = true;
        drag.anchor = index;
        drag.current = index;
        drag.select = !selected.contains(&card);
    }
    if drag.active
        && mouse.pressed(MouseButton::Left)
        && let Some(index) = hovered
    {
        drag.current = index;
    }
    if !drag.active || !mouse.just_released(MouseButton::Left) {
        return false;
    }

    let mut changed = false;
    for (index, card) in cards {
        if !drag.contains(index) {
            continue;
        }
        changed |= if drag.select {
            selected.insert(card)
        } else {
            selected.remove(&card)
        };
    }
    drag.active = false;
    changed
}

pub(crate) fn drag_preview_color(drag: &CardDragSelection, index: usize) -> Color {
    if drag.contains(index) {
        Color::srgba(0.12, 0.14, 0.14, 0.52)
    } else {
        Color::NONE
    }
}

pub(crate) fn hand_card_pose(
    index: usize,
    hand_len: usize,
    hover_amount: f32,
    selected_amount: f32,
    deal_elapsed: f32,
    dealing: bool,
) -> HandCardPose {
    let lift = hover_amount * 5.0 + selected_amount * HAND_CARD_SELECTED_LIFT;
    let deal_progress = if dealing {
        (deal_elapsed / 0.20).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let deal_progress = 1.0 - (1.0 - deal_progress).powi(3);
    let deal_offset = 1.0 - deal_progress;
    let reveal = shengji_hand_card_reveal(hand_len);
    let hand_width = if hand_len == 0 {
        0.0
    } else {
        (hand_len.saturating_sub(1) as f32 * reveal) + CardSize::Hand.dimensions().0
    };
    let final_center_x =
        index as f32 * reveal + CardSize::Hand.dimensions().0 * 0.5 - hand_width * 0.5;

    HandCardPose {
        translation: Val2::px(-final_center_x * deal_offset, -270.0 * deal_offset - lift),
        rotation: Rot2::radians(final_center_x * 0.0015 * deal_offset),
    }
}

pub(crate) fn shengji_hand_card_reveal(hand_len: usize) -> f32 {
    let card_width = CardSize::Hand.dimensions().0;
    let available_width = DESIGN_WIDTH - 96.0;
    if hand_len <= 1 {
        HAND_CARD_REVEAL
    } else {
        HAND_CARD_REVEAL.min((available_width - card_width) / (hand_len - 1) as f32)
    }
}
