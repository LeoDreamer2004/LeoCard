//! 自定义桌布的文件选择与加载。

use crate::app::achievements::LocalAchievementTrigger;
use crate::app::runtime::{
    AppearancePreferences, TableAppearance, TableFeltPicker, TableFeltPickerReceiver,
    TableSurfaceKind, save_appearance_preferences,
};
use crate::app::shell::UiState;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalEvent};
use leocard_client::request_image;
use leocard_platform::storage as fs;
use std::sync::mpsc::TryRecvError;

pub(crate) fn start_table_felt_picker() -> Result<TableFeltPickerReceiver, String> {
    request_image("选择自定义桌布背景")
}

pub(crate) fn decode_table_felt_image(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    image::load_from_memory(bytes)
        .map_err(|error| format!("桌布必须是有效的 PNG、JPG 或 JPEG 图片：{error}"))
}

pub(crate) fn poll_table_felt_picker(
    mut picker: ResMut<TableFeltPicker>,
    mut preferences: ResMut<AppearancePreferences>,
    mut appearance: ResMut<TableAppearance>,
    mut ui: ResMut<UiState>,
    mut achievements: MessageWriter<LocalAchievementTrigger>,
) {
    let Some((kind, receiver)) = picker.pending.as_ref() else {
        return;
    };
    let kind = *kind;
    let result = receiver
        .lock()
        .expect("table felt picker mutex poisoned")
        .try_recv();
    let result = match result {
        Ok(result) => result,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => Err("桌布选择器意外关闭".to_owned()),
    };
    picker.pending = None;
    match result {
        Ok(Some(path)) => {
            let changed = preferences.felt_path(kind).as_ref() != Some(&path);
            let result = fs::read(&path)
                .map_err(|error| format!("无法读取桌布图片：{error}"))
                .and_then(|bytes| decode_table_felt_image(&bytes))
                .and_then(|_| {
                    *preferences.felt_path_mut(kind) = Some(path);
                    save_appearance_preferences(&preferences)
                });
            if changed && result.is_ok() {
                achievements.write(LocalAchievementTrigger(AchievementTrigger::Personal(
                    PersonalEvent::TableBackgroundChanged,
                )));
            }
            if result.is_ok() {
                // Choosing the same file again also reloads edits made to that image.
                appearance.felt_mut(kind).loaded_path = None;
            }
            appearance.error = result.err();
        }
        Ok(None) => {}
        Err(error) => appearance.error = Some(error),
    }
    ui.dirty = true;
}

pub(crate) fn sync_table_appearance(
    preferences: Res<AppearancePreferences>,
    mut appearance: ResMut<TableAppearance>,
    mut images: ResMut<Assets<Image>>,
    mut ui: ResMut<UiState>,
) {
    for kind in [TableSurfaceKind::Cards, TableSurfaceKind::Mahjong] {
        let path = preferences.felt_path(kind);
        let felt = appearance.felt_mut(kind);
        if &felt.loaded_path == path {
            continue;
        }
        felt.loaded_path.clone_from(path);
        felt.custom_felt = None;
        if let Some(path) = path {
            let result = fs::read(path)
                .map_err(|error| format!("无法读取桌布图片：{error}"))
                .and_then(|bytes| decode_table_felt_image(&bytes));
            match result {
                Ok(image) => {
                    let mut image = Image::from_dynamic(image, true, RenderAssetUsages::default());
                    image.sampler = ImageSampler::linear();
                    felt.custom_felt = Some(images.add(image));
                }
                Err(error) => appearance.error = Some(error),
            }
        }
        ui.dirty = true;
    }
}
