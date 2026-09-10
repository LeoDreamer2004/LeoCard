//! 头像拖放、文件选择与图片同步。

use super::super::UiState;
use crate::app::runtime::{
    AppearancePreferences, AvatarImages, AvatarPicker, AvatarPickerReceiver, ClientResource,
    PageErrorState, image_handle_from_png, normalize_avatar, save_appearance_preferences,
};
use bevy::prelude::*;
use bevy::window::FileDragAndDrop;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::{Mutex, mpsc};
use std::thread;

pub(crate) fn handle_avatar_drop(
    mut dropped_files: MessageReader<FileDragAndDrop>,
    client: Option<Res<ClientResource>>,
    mut appearance: ResMut<AppearancePreferences>,
    mut page_error: ResMut<PageErrorState>,
    mut ui: ResMut<UiState>,
) {
    for event in dropped_files.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = event else {
            continue;
        };
        if client.is_some() {
            continue;
        }
        match normalize_avatar(path_buf) {
            Ok(png) => {
                appearance.avatar_png = Some(png);
                page_error.error = save_appearance_preferences(&appearance).err();
            }
            Err(error) => page_error.error = Some(error),
        }
        ui.dirty = true;
    }
}

pub(crate) fn start_avatar_picker() -> Result<AvatarPickerReceiver, String> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("leocard-avatar-picker".to_owned())
        .spawn(move || {
            let _ = sender.send(open_avatar_dialog());
        })
        .map_err(|error| format!("无法启动头像选择器：{error}"))?;
    Ok(Mutex::new(receiver))
}

pub(crate) fn poll_avatar_picker(
    mut picker: ResMut<AvatarPicker>,
    mut appearance: ResMut<AppearancePreferences>,
    mut page_error: ResMut<PageErrorState>,
    mut ui: ResMut<UiState>,
) {
    let Some(receiver) = picker.pending.as_ref() else {
        return;
    };
    let result = receiver
        .lock()
        .expect("avatar picker mutex poisoned")
        .try_recv();
    let result = match result {
        Ok(result) => result,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => Err("头像选择器意外关闭".to_owned()),
    };
    picker.pending = None;
    match result {
        Ok(Some(path)) => match normalize_avatar(&path) {
            Ok(png) => {
                appearance.avatar_png = Some(png);
                page_error.error = save_appearance_preferences(&appearance).err();
            }
            Err(error) => page_error.error = Some(error),
        },
        Ok(None) => {}
        Err(error) => page_error.error = Some(error),
    }
    ui.dirty = true;
}

fn open_avatar_dialog() -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title("选择玩家头像")
        .add_filter("头像图片", &["png", "jpg", "jpeg"])
        .pick_file())
}

pub(crate) fn sync_avatar_images(
    client: Option<Res<ClientResource>>,
    preferences: Res<AppearancePreferences>,
    mut avatar_images: ResMut<AvatarImages>,
    mut images: ResMut<Assets<Image>>,
    mut ui: ResMut<UiState>,
) {
    if avatar_images.local_png != preferences.avatar_png {
        avatar_images.local = preferences
            .avatar_png
            .as_deref()
            .and_then(|png| image_handle_from_png(png, &mut images));
        avatar_images.local_png = preferences.avatar_png.clone();
        ui.dirty = true;
    }
    let Some(client) = client.as_deref() else {
        return;
    };
    for (id, png) in client.0.model().avatars() {
        if avatar_images.remote.contains_key(id) {
            continue;
        }
        if let Some(handle) = image_handle_from_png(png, &mut images) {
            avatar_images.remote.insert(*id, handle);
            ui.dirty = true;
        }
    }
}
