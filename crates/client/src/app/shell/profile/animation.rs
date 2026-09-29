use super::super::UiState;
use super::{ProfileGameTabButton, SelectedProfileGameTab};
use bevy::prelude::*;

pub(crate) const TAB_IDLE: Color = Color::srgb(0.23, 0.24, 0.27);
pub(crate) const TAB_HOVER: Color = Color::srgb(0.30, 0.31, 0.34);

#[derive(Resource, Default)]
pub(crate) struct ProfileMotion {
    pub progress: f32,
    pub target_open: bool,
}

impl ProfileMotion {
    pub(crate) fn toggle(&mut self, ui: &mut UiState) {
        self.target_open = !self.target_open;
        if self.target_open {
            ui.navigation.profile_open = true;
        }
    }

    pub(crate) fn open(&mut self, ui: &mut UiState) {
        self.target_open = true;
        ui.navigation.profile_open = true;
    }
}

#[expect(
    clippy::type_complexity,
    reason = "only unselected profile tabs change their background on hover"
)]
pub(crate) fn update_profile_tab_hover(
    motion: Res<ProfileMotion>,
    mut tabs: Query<
        (&Interaction, &mut BackgroundColor),
        (With<ProfileGameTabButton>, Without<SelectedProfileGameTab>),
    >,
) {
    let opacity = motion.progress * motion.progress * (3.0 - 2.0 * motion.progress);
    for (interaction, mut background) in &mut tabs {
        let color = if *interaction == Interaction::None {
            TAB_IDLE
        } else {
            TAB_HOVER
        };
        background.0 = color.with_alpha(opacity);
    }
}
