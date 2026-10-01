use super::super::{CozyModalKind, ModalAnimations, UiState, advance_modal};
use crate::app::presentation::TEXT;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct SettingsMotion {
    pub progress: f32,
    pub target_open: bool,
}

#[derive(Component)]
pub(crate) struct SettingsTabButton {
    pub label: Entity,
}

#[derive(Component)]
pub(crate) struct SelectedSettingsTab;

#[derive(Component)]
pub(crate) struct SettingsVoiceToggle {
    pub icon: Entity,
    pub selected: bool,
}

pub(crate) fn update_settings_voice_toggle_hover(
    assets: Res<UiAssets>,
    toggles: Query<(&Interaction, &SettingsVoiceToggle)>,
    mut images: Query<&mut ImageNode>,
) {
    for (interaction, toggle) in &toggles {
        let texture = match (toggle.selected, *interaction != Interaction::None) {
            (false, false) => &assets.home.checkbox,
            (false, true) => &assets.home.checkbox_highlighted,
            (true, false) => &assets.home.checkbox_selected,
            (true, true) => &assets.home.checkbox_selected_highlighted,
        };
        if let Ok(mut image) = images.get_mut(toggle.icon)
            && image.image != *texture
        {
            image.image = texture.clone();
        }
    }
}

pub(crate) fn update_settings_tab_hover(
    motion: Res<SettingsMotion>,
    assets: Res<UiAssets>,
    mut tabs: Query<
        (&Interaction, &SettingsTabButton, &mut ImageNode),
        Without<SelectedSettingsTab>,
    >,
    mut labels: Query<&mut TextColor>,
) {
    let opacity = motion.progress * motion.progress * (3.0 - 2.0 * motion.progress);
    for (interaction, tab, mut image) in &mut tabs {
        let hovered = *interaction != Interaction::None;
        let texture = if hovered {
            &assets.home.game_card_hover
        } else {
            &assets.home.game_card
        };
        if image.image != *texture {
            image.image = texture.clone();
        }
        let color = if hovered {
            Color::srgb(0.85, 0.82, 1.0)
        } else {
            TEXT
        };
        if let Ok(mut label) = labels.get_mut(tab.label) {
            label.0 = color.with_alpha(opacity);
        }
    }
}

#[derive(Component)]
pub(crate) struct CozySettingsSlider {
    pub owner: Entity,
    pub track: Entity,
    pub hover: f32,
}

pub(crate) fn animate_settings_sliders(
    time: Res<Time>,
    assets: Res<UiAssets>,
    interactions: Query<&Interaction>,
    mut handles: Query<(&mut CozySettingsSlider, &mut ImageNode)>,
    mut tracks: Query<&mut ImageNode, Without<CozySettingsSlider>>,
) {
    for (mut slider, mut handle) in &mut handles {
        let target = if interactions
            .get(slider.owner)
            .is_ok_and(|interaction| *interaction != Interaction::None)
        {
            1.0
        } else {
            0.0
        };
        let step = time.delta_secs() * 8.0;
        slider.hover = if slider.hover < target {
            (slider.hover + step).min(target)
        } else {
            (slider.hover - step).max(target)
        };
        let highlighted = slider.hover > 0.0;
        let track_asset = if highlighted {
            &assets.home.slider_highlighted
        } else {
            &assets.home.slider
        };
        if let Ok(mut track) = tracks.get_mut(slider.track)
            && track.image != *track_asset
        {
            track.image = track_asset.clone();
        }
        let handle_asset = if highlighted {
            &assets.home.slider_handle_highlighted
        } else {
            &assets.home.slider_handle
        };
        if handle.image != *handle_asset {
            handle.image = handle_asset.clone();
        }
        let rect = highlighted.then(|| {
            let frame = (slider.hover * 3.999).floor();
            Rect::from_corners(
                Vec2::new(0.0, frame * 80.0),
                Vec2::new(32.0, (frame + 1.0) * 80.0),
            )
        });
        if handle.rect != rect {
            handle.rect = rect;
        }
    }
}

impl SettingsMotion {
    pub(crate) fn toggle(&mut self, ui: &mut UiState) {
        self.target_open = !self.target_open;
        if self.target_open {
            ui.settings.open = true;
        }
    }
}

pub(super) fn advance_settings_modal(
    time: Res<Time>,
    mut motion: ResMut<SettingsMotion>,
    mut ui: ResMut<UiState>,
    mut animations: ResMut<ModalAnimations>,
) {
    if ui.settings.open {
        let open = motion.target_open;
        advance_modal(&mut motion.progress, open, time.delta_secs());
        if !motion.target_open && motion.progress == 0.0 {
            ui.settings.open = false;
            ui.dirty = true;
        }
    } else {
        motion.progress = 0.0;
        motion.target_open = false;
    }
    animations.set(CozyModalKind::Settings, motion.progress);
}
