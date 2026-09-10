//! 自定义桌布的文件选择与加载。

use super::super::UiState;
use crate::app::runtime::{
    AppearancePreferences, TableAppearance, TableFeltPicker, TableFeltPickerReceiver,
    save_appearance_preferences,
};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::{Mutex, mpsc};
use std::thread;

pub(crate) fn start_table_felt_picker() -> Result<TableFeltPickerReceiver, String> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("leocard-table-felt-picker".to_owned())
        .spawn(move || {
            let _ = sender.send(open_table_felt_dialog());
        })
        .map_err(|error| format!("无法启动桌布选择器：{error}"))?;
    Ok(Mutex::new(receiver))
}

fn open_table_felt_dialog() -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title("选择自定义桌布背景")
        .add_filter("桌布图片", &["png", "jpg", "jpeg"])
        .pick_file())
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
) {
    let Some(receiver) = picker.pending.as_ref() else {
        return;
    };
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
            preferences.table_felt_path = Some(path);
            appearance.error = save_appearance_preferences(&preferences).err();
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
    if appearance.loaded_path == preferences.table_felt_path {
        return;
    }
    appearance
        .loaded_path
        .clone_from(&preferences.table_felt_path);
    appearance.custom_felt = None;
    appearance.error = None;
    if let Some(path) = &preferences.table_felt_path {
        let result = fs::read(path)
            .map_err(|error| format!("无法读取桌布图片：{error}"))
            .and_then(|bytes| decode_table_felt_image(&bytes));
        match result {
            Ok(image) => {
                appearance.custom_felt = Some(images.add(Image::from_dynamic(
                    image,
                    true,
                    RenderAssetUsages::default(),
                )));
            }
            Err(error) => appearance.error = Some(error),
        }
    }
    ui.dirty = true;
}
