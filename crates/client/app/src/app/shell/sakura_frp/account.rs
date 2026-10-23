use super::actions::start_task;
use super::state::{SakuraState, TaskMessage, TaskOutcome};
use crate::app::runtime::{image_handle_from_png, normalize_avatar_bytes};
use bevy::prelude::*;
use leocard_sakura_frp::{LoginStore, SakuraClient, SavedAccount, SavedLogin, UserInfo};
use std::thread;

pub(super) fn open_account(
    state: &mut SakuraState,
    images: &mut Assets<Image>,
) -> Result<(), String> {
    if !state.login_loaded {
        state.saved_login = LoginStore::open()?.load()?;
        state.login_loaded = true;
        if let Some(saved) = &state.saved_login {
            state.token = saved.token.clone();
            state.remember_token = true;
            state.avatar = saved
                .account
                .avatar_png
                .as_deref()
                .and_then(|png| image_handle_from_png(png, images));
        }
    }
    if state.client.is_none()
        && state.remember_token
        && !state.preview_running
        && let Some(saved) = state.saved_login.clone()
    {
        // Refresh only identity here: opening the login page must not sign in.
        state.preview_running = true;
        let messages = state.messages.clone();
        thread::spawn(move || {
            let result = (|| {
                let client = SakuraClient::new(saved.token.clone())?;
                let user = client.user_info()?;
                Ok(account(
                    &client,
                    saved.token.clone(),
                    &user,
                    Some(&saved.account),
                ))
            })();
            if let Ok(mut queue) = messages.lock() {
                queue.push(TaskMessage::AccountRefreshed {
                    token: saved.token,
                    result,
                });
            }
        });
    }
    Ok(())
}

pub(super) fn login(state: &mut SakuraState) {
    let token = state.token.trim().to_owned();
    start_task(state, move || {
        let client = SakuraClient::new(token.clone())?;
        let dashboard = client.dashboard()?;
        let saved = account(&client, token, &dashboard.user, None);
        Ok(TaskOutcome::LoggedIn(client, dashboard, saved))
    });
}

pub(super) fn remember(state: &mut SakuraState) -> Result<(), String> {
    if state.remember_token {
        LoginStore::open()?.forget()?;
        state.remember_token = false;
        state.saved_login = None;
        if state.client.is_none() {
            state.avatar = None;
        }
    } else {
        state.remember_token = true;
    }
    Ok(())
}

pub(super) fn cache_account(
    state: &mut SakuraState,
    saved: SavedLogin,
    images: &mut Assets<Image>,
) -> Result<(), String> {
    state.avatar = saved
        .account
        .avatar_png
        .as_deref()
        .and_then(|png| image_handle_from_png(png, images));
    if state.remember_token {
        let result = LoginStore::open()?.save(&saved);
        state.saved_login = Some(saved);
        result?;
    } else {
        LoginStore::open()?.forget()?;
        state.saved_login = None;
    }
    Ok(())
}

fn account(
    client: &SakuraClient,
    token: String,
    user: &UserInfo,
    previous: Option<&SavedAccount>,
) -> SavedLogin {
    let avatar_png = client
        .avatar(user)
        .ok()
        .and_then(|bytes| normalize_avatar_bytes(&bytes).ok())
        .or_else(|| {
            previous
                .filter(|previous| previous.avatar_url == user.avatar)
                .and_then(|previous| previous.avatar_png.clone())
        });
    SavedLogin {
        token,
        account: SavedAccount {
            name: user.name.clone(),
            avatar_url: user.avatar.clone(),
            avatar_png,
        },
    }
}
