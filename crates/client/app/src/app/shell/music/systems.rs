use super::motion::MusicMotion;
use super::state::{AccountResult, CatalogResult, MusicState};
use super::task::Task;
use crate::app::runtime::image_handle_from_png;
use crate::app::shell::UiState;
use bevy::prelude::*;
use image::{DynamicImage, ImageFormat, RgbaImage};
use leocard_music::{LoginPoll, MusicClient};
use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) fn poll_music(
    mut state: ResMut<MusicState>,
    mut ui: ResMut<UiState>,
    mut images: ResMut<Assets<Image>>,
    motion: Res<MusicMotion>,
) {
    let visible = motion.target_open;
    if visible && !state.initialized {
        state.initialized = true;
        if let Some(saved) = state.saved.as_ref() {
            match MusicClient::restore(saved) {
                Ok(client) => {
                    let worker = client.clone();
                    state.client = Some(client);
                    state.status = "正在恢复网易云账号".to_owned();
                    match Task::start(move || worker.account().map(AccountResult::LoggedIn)) {
                        Ok(task) => state.account_task = Some(task),
                        Err(error) => state.error = Some(error),
                    }
                }
                Err(error) => state.error = Some(error),
            }
        }
        ui.dirty = true;
    }
    if let Some(result) = state.account_task.as_ref().and_then(Task::poll) {
        state.account_task = None;
        if let Err(error) =
            result.and_then(|outcome| apply_account(outcome, &mut state, &mut images))
        {
            state.error = Some(error);
            state.qr_key = None;
            state.status = "关联未完成，请重新扫码".to_owned();
        }
        ui.dirty = true;
    }
    if let Some(result) = state.catalog_task.as_ref().and_then(Task::poll) {
        state.catalog_task = None;
        match result {
            Ok(CatalogResult::Playlists(playlists)) => {
                state.playlists = playlists;
                state.playlist_page = 0;
                if state
                    .selected
                    .is_some_and(|id| !state.playlists.iter().any(|playlist| playlist.id == id))
                {
                    state.selected = None;
                    state.songs = Arc::default();
                }
            }
            Ok(CatalogResult::Songs(id, songs)) if state.selected == Some(id) => {
                state.songs = Arc::new(songs)
            }
            Ok(CatalogResult::Songs(_, _)) => {}
            Err(error) => state.error = Some(error),
        }
        ui.dirty = true;
    }
    if visible
        && state.account_task.is_none()
        && state.account.is_none()
        && Instant::now() >= state.poll_at
        && let (Some(client), Some(key)) = (state.client.clone(), state.qr_key.clone())
    {
        state.poll_at = Instant::now() + Duration::from_secs(3);
        match Task::start(move || client.poll_login(&key).map(AccountResult::Poll)) {
            Ok(task) => state.account_task = Some(task),
            Err(error) => {
                state.error = Some(error);
                state.qr_key = None;
                ui.dirty = true;
            }
        }
    }
}

fn apply_account(
    outcome: AccountResult,
    state: &mut MusicState,
    images: &mut Assets<Image>,
) -> Result<(), String> {
    match outcome {
        AccountResult::Qr(client, qr) => {
            let (pixels, width) = qr.pixels()?;
            let image = RgbaImage::from_raw(width, width, pixels).ok_or("登录二维码生成失败")?;
            let mut png = Cursor::new(Vec::new());
            DynamicImage::ImageRgba8(image)
                .write_to(&mut png, ImageFormat::Png)
                .map_err(|_| "登录二维码生成失败")?;
            state.qr_image = image_handle_from_png(png.get_ref(), images);
            state.qr_key = Some(qr.key);
            state.client = Some(client);
            state.poll_at = Instant::now() + Duration::from_secs(3);
            state.status = "请用网易云音乐 App 扫码登录".to_owned();
        }
        AccountResult::Poll(LoginPoll::Waiting) => {}
        AccountResult::Poll(LoginPoll::Scanned) => {
            state.status = "已扫码，请在网易云音乐中确认登录".to_owned()
        }
        AccountResult::Poll(LoginPoll::Expired) => {
            state.qr_key = None;
            state.qr_image = None;
            state.status = "二维码已过期，请刷新二维码".to_owned();
        }
        AccountResult::Poll(LoginPoll::Authorized) => {
            state.qr_key = None;
            state.qr_image = None;
            let client = state.client.clone().ok_or("登录状态丢失，请重新扫码")?;
            state.status = "正在读取网易云账号".to_owned();
            state.account_task = Some(Task::start(move || {
                client.account().map(AccountResult::LoggedIn)
            })?);
        }
        AccountResult::LoggedIn(account) => {
            state.status = format!("已关联 {}", account.nickname);
            state.account = Some(account);
            state.error = None;
            state.save();
            state.load_playlists()?;
        }
    }
    Ok(())
}
