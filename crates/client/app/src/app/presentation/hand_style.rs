//! 扑克牌悬停与选中共用的呼吸高亮。

use super::{ACCENT, BORDER};
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct HandCardHighlight {
    glow: f32,
    selected: bool,
}

impl HandCardHighlight {
    pub(crate) fn new(hover: f32, selected_amount: f32, selected: bool, pulse: f32) -> Self {
        Self {
            glow: (hover * pulse + selected_amount * 0.72).clamp(0.0, 1.0),
            selected,
        }
    }

    pub(crate) fn tint(self) -> Color {
        Color::srgb(1.0, 1.0 - self.glow * 0.035, 1.0 - self.glow * 0.16)
    }

    pub(crate) fn border(self) -> Color {
        if self.selected { ACCENT } else { BORDER }
    }

    pub(crate) fn outline(self) -> Outline {
        Outline::new(
            px(0.75 + self.glow * 1.5),
            px(0),
            ACCENT.with_alpha(self.glow * 0.92),
        )
    }

    pub(crate) fn shadow(self) -> BoxShadow {
        BoxShadow::new(
            ACCENT.with_alpha(self.glow * 0.58),
            px(0),
            px(0),
            px(self.glow * 2.0),
            px(2.0 + self.glow * 10.0),
        )
    }

    pub(crate) fn apply(
        self,
        image: &mut ImageNode,
        border: &mut BorderColor,
        outline: &mut Outline,
        shadow: &mut BoxShadow,
    ) {
        image.color = self.tint();
        border.set_all(self.border());
        *outline = self.outline();
        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(self.glow * 0.58);
            style.spread_radius = px(self.glow * 2.0);
            style.blur_radius = px(2.0 + self.glow * 10.0);
        }
    }
}
