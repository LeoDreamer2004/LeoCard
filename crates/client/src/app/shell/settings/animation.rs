use super::super::{ProfileMotion, UiState, UpdateManager};
use crate::app::games::{MahjongUiState, UnoUiState};
use crate::app::presentation::{TEXT, ease_out_cubic};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;

const DURATION: f32 = 0.22;
const BACKDROP_ALPHA: f32 = 0.76;

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

#[derive(Clone, Copy, Component)]
pub(crate) enum CozyModalKind {
    Settings,
    Profile,
    MahjongFanGuide,
    UnoExpansionSettings,
    UpdateDialog,
}

#[derive(Component)]
pub(crate) struct CozyModalBackdrop(pub CozyModalKind);

#[derive(Component)]
pub(crate) struct CozyModalPanel(pub CozyModalKind);

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

#[derive(Component)]
pub(crate) struct CozyModalFadeBase {
    image: Option<Color>,
    text: Option<Color>,
    background: Option<Color>,
    border: Option<BorderColor>,
}

impl SettingsMotion {
    pub(crate) fn toggle(&mut self, ui: &mut UiState) {
        self.target_open = !self.target_open;
        if self.target_open {
            ui.navigation.settings_open = true;
        }
    }
}

pub(crate) fn cozy_backdrop_color(progress: f32) -> Color {
    Color::srgba(
        0.005,
        0.015,
        0.012,
        BACKDROP_ALPHA * ease_out_cubic(progress),
    )
}

pub(crate) fn cozy_panel_transform(progress: f32) -> UiTransform {
    let visible = ease_out_cubic(progress);
    let mut transform = UiTransform::from_translation(Val2::px(0.0, 18.0 * (1.0 - visible)));
    transform.scale = Vec2::splat(0.96 + 0.04 * visible);
    transform
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy provides the modal state and separate visual components for the fade"
)]
pub(crate) fn animate_cozy_modals(
    time: Res<Time>,
    mut settings_motion: ResMut<SettingsMotion>,
    mut profile_motion: ResMut<ProfileMotion>,
    mut mahjong_ui: ResMut<MahjongUiState>,
    mut uno_ui: ResMut<UnoUiState>,
    mut updater: ResMut<UpdateManager>,
    mut ui: ResMut<UiState>,
    mut commands: Commands,
    mut backdrops: Query<(&CozyModalBackdrop, &mut BackgroundColor)>,
    mut panels: Query<(Entity, &CozyModalPanel, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: Query<
        (
            Option<&mut ImageNode>,
            Option<&mut TextColor>,
            Option<&mut BackgroundColor>,
            Option<&mut BorderColor>,
            Option<&CozyModalFadeBase>,
        ),
        Without<CozyModalBackdrop>,
    >,
) {
    if ui.navigation.settings_open {
        let direction = if settings_motion.target_open {
            1.0
        } else {
            -1.0
        };
        settings_motion.progress =
            (settings_motion.progress + direction * time.delta_secs() / DURATION).clamp(0.0, 1.0);
    } else {
        settings_motion.progress = 0.0;
        settings_motion.target_open = false;
    }
    if ui.navigation.profile_open {
        let direction = if profile_motion.target_open {
            1.0
        } else {
            -1.0
        };
        profile_motion.progress =
            (profile_motion.progress + direction * time.delta_secs() / DURATION).clamp(0.0, 1.0);
    } else {
        profile_motion.progress = 0.0;
        profile_motion.target_open = false;
    }
    let direction = if mahjong_ui.fan_guide_open { 1.0 } else { -1.0 };
    mahjong_ui.fan_guide_progress =
        (mahjong_ui.fan_guide_progress + direction * time.delta_secs() / DURATION).clamp(0.0, 1.0);
    let direction = if uno_ui.expansion_settings_open {
        1.0
    } else {
        -1.0
    };
    let previous_uno_progress = uno_ui.expansion_settings_progress;
    uno_ui.expansion_settings_progress =
        (previous_uno_progress + direction * time.delta_secs() / DURATION).clamp(0.0, 1.0);
    let direction = if updater.dialog_open { 1.0 } else { -1.0 };
    let previous_update_progress = updater.dialog_progress;
    updater.dialog_progress =
        (previous_update_progress + direction * time.delta_secs() / DURATION).clamp(0.0, 1.0);
    for (kind, mut backdrop) in &mut backdrops {
        let progress = match kind.0 {
            CozyModalKind::Settings => settings_motion.progress,
            CozyModalKind::Profile => profile_motion.progress,
            CozyModalKind::MahjongFanGuide => mahjong_ui.fan_guide_progress,
            CozyModalKind::UnoExpansionSettings => uno_ui.expansion_settings_progress,
            CozyModalKind::UpdateDialog => updater.dialog_progress,
        };
        backdrop.0 = cozy_backdrop_color(progress);
    }
    let mut roots = Vec::new();
    for (entity, kind, mut panel) in &mut panels {
        let progress = match kind.0 {
            CozyModalKind::Settings => settings_motion.progress,
            CozyModalKind::Profile => profile_motion.progress,
            CozyModalKind::MahjongFanGuide => mahjong_ui.fan_guide_progress,
            CozyModalKind::UnoExpansionSettings => uno_ui.expansion_settings_progress,
            CozyModalKind::UpdateDialog => updater.dialog_progress,
        };
        roots.push((entity, progress));
        *panel = cozy_panel_transform(progress);
    }
    for (root, progress) in roots {
        let opacity = progress * progress * (3.0 - 2.0 * progress);
        let mut pending = vec![root];
        while let Some(entity) = pending.pop() {
            if let Ok(child_entities) = children.get(entity) {
                pending.extend(child_entities.iter());
            }
            let Ok((mut image, mut text, mut background, mut border, base)) =
                visuals.get_mut(entity)
            else {
                continue;
            };
            let base = if let Some(base) = base {
                (base.image, base.text, base.background, base.border)
            } else {
                let base = CozyModalFadeBase {
                    image: image.as_ref().map(|image| image.color),
                    text: text.as_ref().map(|text| text.0),
                    background: background.as_ref().map(|background| background.0),
                    border: border.as_ref().map(|border| **border),
                };
                let colors = (base.image, base.text, base.background, base.border);
                commands.entity(entity).insert(base);
                colors
            };
            if let (Some(image), Some(color)) = (image.as_mut(), base.0) {
                image.color = color.with_alpha(color.alpha() * opacity);
            }
            if let (Some(text), Some(color)) = (text.as_mut(), base.1) {
                text.0 = color.with_alpha(color.alpha() * opacity);
            }
            if let (Some(background), Some(color)) = (background.as_mut(), base.2) {
                background.0 = color.with_alpha(color.alpha() * opacity);
            }
            if let (Some(border), Some(colors)) = (border.as_mut(), base.3) {
                border.top = colors.top.with_alpha(colors.top.alpha() * opacity);
                border.right = colors.right.with_alpha(colors.right.alpha() * opacity);
                border.bottom = colors.bottom.with_alpha(colors.bottom.alpha() * opacity);
                border.left = colors.left.with_alpha(colors.left.alpha() * opacity);
            }
        }
    }
    if ui.navigation.settings_open
        && !settings_motion.target_open
        && settings_motion.progress == 0.0
    {
        ui.navigation.settings_open = false;
        ui.dirty = true;
    }
    if ui.navigation.profile_open && !profile_motion.target_open && profile_motion.progress == 0.0 {
        ui.navigation.profile_open = false;
        ui.navigation.player_profile = None;
        ui.dirty = true;
    }
    if !uno_ui.expansion_settings_open
        && previous_uno_progress > 0.0
        && uno_ui.expansion_settings_progress == 0.0
    {
        ui.dirty = true;
    }
    if !updater.dialog_open && previous_update_progress > 0.0 && updater.dialog_progress == 0.0 {
        ui.dirty = true;
    }
}
