//! 手牌悬停、拖选和预览输入。

use super::{CardAnimationState, DESIGN_WIDTH, HAND_CARD_REVEAL};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::CardDragSelection;
use std::collections::HashSet;
use std::hash::Hash;

const HAND_CARD_SELECTED_LIFT: f32 = 24.0;

#[derive(Component)]
pub(crate) struct HandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum CardSize {
    ShengjiHand,
    QiGuiHand,
    QiGuiSeat,
    Score,
    TableScore,
    FinishedHand,
    TrickSeat,
}

impl CardSize {
    pub(crate) fn dimensions(self) -> (f32, f32) {
        match self {
            Self::ShengjiHand => (82.0, 111.0),
            Self::QiGuiHand => (94.0, 128.0),
            Self::QiGuiSeat => (80.0, 109.0),
            Self::Score => (36.0, 49.0),
            Self::TableScore => (28.0, 38.0),
            Self::FinishedHand => (43.2, 58.8),
            Self::TrickSeat => (70.0, 95.0),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HandCardPose {
    pub translation: Val2,
    pub rotation: Rot2,
}

pub(crate) struct HandCardLayout {
    pub index: usize,
    pub hand_len: usize,
    pub card_width: f32,
    pub reveal: f32,
}

pub(crate) fn slot_hover_target(
    drag: &CardDragSelection,
    index: usize,
    interaction: PickingInteraction,
    cursor: &RelativeCursorPosition,
) -> f32 {
    f32::from(if drag.active {
        cursor.cursor_over() || index == drag.current
    } else {
        matches!(
            interaction,
            PickingInteraction::Hovered | PickingInteraction::Pressed
        )
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
    let mut cards = cards.peekable();
    // 非当前游戏的选牌系统没有手牌，不能消耗当前游戏的拖选状态。
    if cards.peek().is_none() {
        return false;
    }

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
    size: CardSize,
) -> HandCardPose {
    let card_width = size.dimensions().0;
    let reveal = hand_card_reveal_for_width(
        hand_len,
        card_width,
        match size {
            CardSize::ShengjiHand => 30.0,
            CardSize::QiGuiHand => 32.0,
            _ => HAND_CARD_REVEAL,
        },
    );
    hand_card_pose_for_layout(
        HandCardLayout {
            index,
            hand_len,
            card_width,
            reveal,
        },
        CardAnimationState {
            face_hover_amount: hover_amount,
            selected_amount,
            deal_elapsed,
            dealing,
            ..default()
        },
    )
}

pub(crate) fn hand_card_pose_for_layout(
    layout: HandCardLayout,
    animation: CardAnimationState,
) -> HandCardPose {
    let HandCardLayout {
        index,
        hand_len,
        card_width,
        reveal,
    } = layout;
    let lift =
        animation.face_hover_amount * 5.0 + animation.selected_amount * HAND_CARD_SELECTED_LIFT;
    let progress = if animation.dealing {
        (animation.deal_elapsed / 0.20).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let deal_offset = (1.0 - progress).powi(3);
    let hand_width = if hand_len == 0 {
        0.0
    } else {
        (hand_len.saturating_sub(1) as f32 * reveal) + card_width
    };
    let final_center_x = index as f32 * reveal + card_width * 0.5 - hand_width * 0.5;

    HandCardPose {
        translation: Val2::px(-final_center_x * deal_offset, -270.0 * deal_offset - lift),
        rotation: Rot2::radians(final_center_x * 0.0015 * deal_offset),
    }
}

pub(crate) fn shengji_hand_card_reveal(hand_len: usize) -> f32 {
    hand_card_reveal_for_width(hand_len, CardSize::ShengjiHand.dimensions().0, 30.0)
}

fn hand_card_reveal_for_width(hand_len: usize, card_width: f32, preferred_reveal: f32) -> f32 {
    let available_width = DESIGN_WIDTH - 96.0;
    if hand_len <= 1 {
        preferred_reveal
    } else {
        preferred_reveal.min((available_width - card_width) / (hand_len - 1) as f32)
    }
}
