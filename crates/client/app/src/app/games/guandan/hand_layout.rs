use super::arrangement::{HandColumn, HandLayoutMode};
use crate::app::presentation::{
    CardAnimationState, CardSize, HandCardPose, game_side_control_scale, hand_card_pose_at,
};
use bevy::prelude::*;
use leocard_guandan::GuandanCard;
use leocard_platform::is_mobile;

const HAND_WIDTH: f32 = 920.0;
const STACK_STEP: f32 = 34.0;
const TOP_MARGIN: f32 = 30.0;

#[derive(Clone, Copy)]
pub(super) struct HandCardPlacement {
    pub index: usize,
    pub column: usize,
    pub position: Vec2,
    pub size: Vec2,
    pub stacked: bool,
    center: Vec2,
}

impl HandCardPlacement {
    pub fn hit_rect(self) -> Rect {
        // Keep a stable target across the selected-card motion, including its
        // visible destination. The second click must still reach the same card.
        let (before, after) = if self.stacked {
            (Vec2::ZERO, Vec2::new(12.0 * game_side_control_scale(), 0.0))
        } else {
            (Vec2::new(0.0, 30.0), Vec2::ZERO)
        };
        Rect::from_corners(self.position - before, self.position + self.size + after)
    }

    pub fn pose(self, mut animation: CardAnimationState) -> HandCardPose {
        let selection_offset = if self.stacked {
            animation.face_hover_amount = 0.0;
            Vec2::new(12.0 * game_side_control_scale(), 0.0)
        } else {
            Vec2::new(0.0, -24.0)
        };
        hand_card_pose_at(self.center, animation, selection_offset)
    }
}

pub(super) struct GuandanHandLayout {
    pub cards: Vec<(GuandanCard, HandCardPlacement)>,
    pub size: Vec2,
}

impl GuandanHandLayout {
    pub fn new(columns: &[HandColumn], mode: HandLayoutMode) -> Self {
        if columns.is_empty() {
            return Self {
                cards: Vec::new(),
                size: Vec2::new(0.0, CardSize::ShengjiHand.dimensions().1 + TOP_MARGIN),
            };
        }
        match mode {
            HandLayoutMode::Columns => Self::columns(columns),
            HandLayoutMode::Row => Self::row(columns),
        }
    }

    fn columns(columns: &[HandColumn]) -> Self {
        let (width, height) = CardSize::ShengjiHand.dimensions();
        // Keep the original card size and exposed suits. Deep stacks continue in
        // neighboring columns, which still share one double-click selection group.
        let maximum_rows = if is_mobile() { 3 } else { 5 };
        let stacks = columns
            .iter()
            .enumerate()
            .flat_map(|(column, group)| {
                group
                    .cards
                    .chunks(maximum_rows)
                    .map(move |cards| (column, cards))
            })
            .collect::<Vec<_>>();
        let rows = stacks
            .iter()
            .map(|(_, cards)| cards.len())
            .max()
            .unwrap_or(1);
        let step = STACK_STEP * game_side_control_scale();
        let pitch = width;
        let body_width = width + stacks.len().saturating_sub(1) as f32 * pitch;
        let body_height = height + rows.saturating_sub(1) as f32 * step;
        let mut result = Vec::new();
        for (stack, &(column, cards)) in stacks.iter().enumerate() {
            for (row, &card) in cards.iter().enumerate() {
                let position = Vec2::new(
                    stack as f32 * pitch,
                    TOP_MARGIN + (rows - cards.len() + row) as f32 * step,
                );
                result.push((
                    card,
                    HandCardPlacement {
                        index: result.len(),
                        column,
                        position,
                        size: Vec2::new(width, height),
                        stacked: true,
                        center: Vec2::new(
                            position.x + width * 0.5 - body_width * 0.5,
                            position.y - TOP_MARGIN,
                        ),
                    },
                ));
            }
        }
        Self {
            cards: result,
            size: Vec2::new(
                body_width + 12.0 * game_side_control_scale(),
                TOP_MARGIN + body_height + 6.0,
            ),
        }
    }

    fn row(columns: &[HandColumn]) -> Self {
        let (width, height) = CardSize::ShengjiHand.dimensions();
        let count = columns
            .iter()
            .map(|column| column.cards.len())
            .sum::<usize>();
        let reveal = if count <= 1 {
            0.0
        } else {
            (30.0 * game_side_control_scale()).min((HAND_WIDTH - width) / (count - 1) as f32)
        };
        let body_width = width + count.saturating_sub(1) as f32 * reveal;
        let mut result = Vec::new();
        let mut x = 0.0;
        for (column, group) in columns.iter().enumerate() {
            for &card in &group.cards {
                result.push((
                    card,
                    HandCardPlacement {
                        index: result.len(),
                        column,
                        stacked: false,
                        position: Vec2::new(x, TOP_MARGIN),
                        size: Vec2::new(width, height),
                        center: Vec2::new(x + width * 0.5 - body_width * 0.5, 0.0),
                    },
                ));
                x += reveal;
            }
        }
        Self {
            cards: result,
            size: Vec2::new(body_width, TOP_MARGIN + height + 6.0),
        }
    }
}
