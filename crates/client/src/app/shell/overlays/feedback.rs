//! 错误提示的状态同步、重触发与入退场动画。

use super::*;

pub const PLAY_ERROR_TOAST_DURATION: f32 = 2.4;
pub const PLAY_ERROR_TOAST_ENTRY_DURATION: f32 = 0.28;
const PLAY_ERROR_TOAST_FADE_DURATION: f32 = 0.42;
pub const PLAY_ERROR_TOAST_SHAKE_DURATION: f32 = 0.42;

pub fn sync_play_error_toast(
    client: Option<Res<ClientResource>>,
    form: Res<ConnectionForm>,
    appearance: Res<TableAppearance>,
    mut toast: ResMut<PlayErrorToast>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let mut next_message = None;
    let mut play_error_sound = false;
    if let Some(model) = client.as_deref().map(|client| client.0.model()) {
        if model.rejection_serial() != toast.seen_rejection_serial {
            toast.seen_rejection_serial = model.rejection_serial();
            next_message = model.last_rejection().and_then(rejection_label);
            play_error_sound = next_message.is_some();
        }
        if model.notice_serial() != toast.seen_notice_serial {
            toast.seen_notice_serial = model.notice_serial();
            next_message = model.last_notice().map(str::to_owned);
            play_error_sound = false;
        }
    }
    if toast.observed_form_error != form.error {
        toast.observed_form_error.clone_from(&form.error);
        if let Some(error) = &form.error {
            next_message = Some(error.clone());
            play_error_sound = true;
        }
    }
    if toast.observed_appearance_error != appearance.error {
        toast
            .observed_appearance_error
            .clone_from(&appearance.error);
        if let Some(error) = &appearance.error {
            next_message = Some(format!("桌布加载失败：{error}"));
            play_error_sound = true;
        }
    }
    let Some(message) = next_message else {
        return;
    };
    let retriggered = toast.active;
    toast.message = Some(message);
    toast.elapsed = 0.0;
    toast.shake_elapsed = retriggered.then_some(0.0);
    toast.entering = !retriggered;
    toast.active = true;
    if play_error_sound {
        commands.spawn((
            AudioPlayer::new(assets.audio.error_popup_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
    ui.dirty = true;
}

#[derive(Clone, Copy, Debug)]
pub struct PlayErrorToastVisual {
    pub x: f32,
    pub y: f32,
    pub opacity: f32,
}

pub fn play_error_toast_visual(toast: &PlayErrorToast) -> PlayErrorToastVisual {
    let entry = if toast.entering {
        (toast.elapsed / PLAY_ERROR_TOAST_ENTRY_DURATION).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let entry = 1.0 - (1.0 - entry).powi(3);
    let fade = ((PLAY_ERROR_TOAST_DURATION - toast.elapsed) / PLAY_ERROR_TOAST_FADE_DURATION)
        .clamp(0.0, 1.0);
    let life = (toast.elapsed / PLAY_ERROR_TOAST_DURATION).clamp(0.0, 1.0);
    let x = toast.shake_elapsed.map_or(0.0, |elapsed| {
        let strength = 1.0 - (elapsed / PLAY_ERROR_TOAST_SHAKE_DURATION).clamp(0.0, 1.0);
        (elapsed * 58.0).sin() * 11.0 * strength
    });
    PlayErrorToastVisual {
        x,
        y: -31.0 + (1.0 - entry) * 25.0 - life * 4.0,
        opacity: entry * fade,
    }
}

pub fn animate_play_error_popup(
    time: Res<Time>,
    mut toast: ResMut<PlayErrorToast>,
    mut popups: Query<
        (
            &mut UiTransform,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BoxShadow,
            &mut Visibility,
        ),
        With<PlayErrorPopup>,
    >,
    mut texts: Query<&mut TextColor, With<PlayErrorPopupText>>,
) {
    if !toast.active {
        for (_, _, _, _, mut visibility) in &mut popups {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    toast.elapsed += time.delta_secs();
    if toast.entering && toast.elapsed >= PLAY_ERROR_TOAST_ENTRY_DURATION {
        toast.entering = false;
    }
    if let Some(elapsed) = toast.shake_elapsed.as_mut() {
        *elapsed += time.delta_secs();
        if *elapsed >= PLAY_ERROR_TOAST_SHAKE_DURATION {
            toast.shake_elapsed = None;
        }
    }
    if toast.elapsed >= PLAY_ERROR_TOAST_DURATION {
        toast.active = false;
    }

    let visual = play_error_toast_visual(&toast);
    for (mut transform, mut background, mut border, mut shadow, mut visibility) in &mut popups {
        transform.translation = Val2::px(visual.x, visual.y);
        background.0 = HEADER_BG.with_alpha(0.97 * visual.opacity);
        border.set_all(DANGER.with_alpha(0.9 * visual.opacity));
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(0.45 * visual.opacity);
        }
        *visibility = if toast.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text_color in &mut texts {
        text_color.0 = DANGER.with_alpha(visual.opacity);
    }
}
