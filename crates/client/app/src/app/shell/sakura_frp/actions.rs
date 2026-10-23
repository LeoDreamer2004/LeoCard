use super::state::{SakuraState, TaskMessage, TaskOutcome, TunnelStartup};
use crate::app::runtime::PageErrorState;
use crate::app::shell::{
    ConfirmationDialog, CozyButtonVariant, DomainUiAction, PageMotion, PressedUiAction, UiAction,
    UiState,
};
use bevy::prelude::*;
use bevy_clipboard::Clipboard;
use leocard_client::open_url;
use leocard_sakura_frp::{CreateTunnel, FrpcProgress, TunnelType};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub(crate) enum SakuraUiAction {
    Show,
    Close,
    Login,
    ToggleRemember,
    Register,
    RecoverPassword,
    Logout,
    Refresh,
    ToggleKindMenu,
    CloseKindMenu,
    SelectKind(TunnelType),
    SelectNode(u64),
    TogglePublicGame,
    Create,
    Start(u64),
    CloseStartup,
    Stop(u64),
    CopyAddress(u64),
    AskDelete(u64),
    ConfirmDelete(u64),
}

impl DomainUiAction for SakuraUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Sakura(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent account, navigation, clipboard and dialog resources"
)]
pub(super) fn dispatch_sakura_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut state: ResMut<SakuraState>,
    mut error: ResMut<PageErrorState>,
    mut images: ResMut<Assets<Image>>,
    mut ui: ResMut<UiState>,
    mut page_motion: ResMut<PageMotion>,
    mut confirmation: ResMut<ConfirmationDialog>,
    mut clipboard: ResMut<Clipboard>,
) {
    for pressed in actions.read() {
        let Some(action) = SakuraUiAction::extract(&pressed.0) else {
            continue;
        };
        ui.dirty = true;
        match action {
            SakuraUiAction::Show => {
                if !state.open {
                    page_motion.begin();
                }
                state.open = true;
                state.kind_menu_open = false;
                error.error = super::account::open_account(&mut state, &mut images).err();
                ui.shop.open = false;
                ui.achievements.open = false;
                ui.settings.open = false;
                ui.profile.open = false;
                if state.client.is_some() {
                    refresh(&mut state);
                }
            }
            SakuraUiAction::Close => {
                if state.open {
                    page_motion.begin();
                }
                state.open = false;
                state.kind_menu_open = false;
            }
            SakuraUiAction::Login => {
                error.error = None;
                super::account::login(&mut state);
            }
            SakuraUiAction::ToggleRemember => {
                error.error = super::account::remember(&mut state).err();
            }
            SakuraUiAction::Register | SakuraUiAction::RecoverPassword => {
                error.error = open_url(if matches!(action, SakuraUiAction::Register) {
                    "https://openid.13a.com/register"
                } else {
                    "https://openid.13a.com/reset_password"
                })
                .err();
            }
            SakuraUiAction::Logout => {
                if !state.busy {
                    state.running.clear();
                    state.client = None;
                    state.user = None;
                    state.token = state
                        .saved_login
                        .as_ref()
                        .map_or_else(String::new, |saved| saved.token.clone());
                    state.nodes.clear();
                    state.tunnels.clear();
                    state.kind_menu_open = false;
                    state.copied_address = None;
                }
            }
            SakuraUiAction::Refresh => refresh(&mut state),
            SakuraUiAction::ToggleKindMenu => state.kind_menu_open = !state.kind_menu_open,
            SakuraUiAction::CloseKindMenu => state.kind_menu_open = false,
            SakuraUiAction::SelectKind(kind) => {
                state.kind = *kind;
                state.kind_menu_open = false;
                state.select_compatible_node();
            }
            SakuraUiAction::SelectNode(id) => {
                if state
                    .nodes
                    .iter()
                    .any(|node| node.id == *id && state.node_compatible(node))
                {
                    state.node = Some(*id);
                }
            }
            SakuraUiAction::TogglePublicGame => {
                state.public_game = !state.public_game;
                state.select_compatible_node();
            }
            SakuraUiAction::Create => {
                let Some(node) = state
                    .selected_node()
                    .filter(|node| state.node_compatible(node))
                    .cloned()
                else {
                    error.error = Some("请先选择可用节点".to_owned());
                    continue;
                };
                let Ok(port) = state.local_port.trim().parse::<u16>() else {
                    error.error = Some("本地端口需为 1 到 65535".to_owned());
                    continue;
                };
                let request = CreateTunnel {
                    name: state.name.trim().to_owned(),
                    kind: state.kind,
                    node: node.id,
                    local_ip: state.local_ip.trim().to_owned(),
                    local_port: port,
                    remote: (!state.remote.trim().is_empty())
                        .then(|| state.remote.trim().to_owned()),
                    public_game: state.public_game,
                };
                if let Err(message) = request.validate(&node) {
                    error.error = Some(message);
                    continue;
                }
                let Some(client) = state.client.clone() else {
                    continue;
                };
                start_task(&mut state, move || {
                    client.create_tunnel(&request, &node)?;
                    let dashboard = client.dashboard().ok();
                    Ok(TaskOutcome::Created(dashboard))
                });
            }
            SakuraUiAction::Start(id) => {
                if state.busy || state.startup.is_some() || state.running.contains_key(id) {
                    continue;
                }
                let Some(client) = state.client.clone() else {
                    continue;
                };
                let Some(tunnel) = state.tunnels.iter().find(|tunnel| tunnel.id == *id) else {
                    continue;
                };
                state.startup = Some(TunnelStartup {
                    id: *id,
                    name: tunnel.name.clone(),
                    stage: FrpcProgress::ResolvingDownload,
                    open: true,
                    animation: 0.0,
                });
                state.kind_menu_open = false;
                error.error = None;
                let messages = state.messages.clone();
                let id = *id;
                start_task(&mut state, move || {
                    client
                        .start_tunnel(id, |progress| {
                            if let Ok(mut queue) = messages.lock() {
                                queue.push(TaskMessage::StartupProgress(progress));
                            }
                        })
                        .map(TaskOutcome::Started)
                });
            }
            SakuraUiAction::CloseStartup => {
                if let Some(startup) = &mut state.startup
                    && startup.stage == FrpcProgress::Started
                {
                    startup.open = false;
                }
            }
            SakuraUiAction::Stop(id) => {
                state.running.remove(id);
            }
            SakuraUiAction::CopyAddress(id) => {
                let Some(tunnel) = state.tunnels.iter().find(|tunnel| tunnel.id == *id) else {
                    continue;
                };
                match clipboard.set_text(tunnel.address(&state.nodes)) {
                    Ok(()) => {
                        state.copied_address = Some((*id, Instant::now() + Duration::from_secs(2)));
                    }
                    Err(message) => {
                        error.error = Some(format!("复制公网地址失败：{message}"));
                    }
                }
            }
            SakuraUiAction::AskDelete(id) => {
                if state.busy {
                    continue;
                }
                let Some(tunnel) = state.tunnels.iter().find(|tunnel| tunnel.id == *id) else {
                    continue;
                };
                confirmation.show_with_button(
                    "删除隧道".to_owned(),
                    format!("确定删除隧道「{}」吗？此操作无法撤销。", tunnel.name),
                    UiAction::Sakura(SakuraUiAction::ConfirmDelete(*id)),
                    "删除",
                    CozyButtonVariant::Danger,
                );
            }
            SakuraUiAction::ConfirmDelete(id) => {
                if state.busy || !state.tunnels.iter().any(|tunnel| tunnel.id == *id) {
                    continue;
                }
                let Some(client) = state.client.clone() else {
                    continue;
                };
                let id = *id;
                start_task(&mut state, move || {
                    let failed_to_disconnect = client.delete_tunnel(id)?;
                    let dashboard = client.dashboard().ok();
                    Ok(TaskOutcome::Deleted(id, failed_to_disconnect, dashboard))
                });
            }
        }
    }
}

pub(super) fn refresh(state: &mut SakuraState) {
    let Some(client) = state.client.clone() else {
        return;
    };
    start_task(state, move || {
        client.dashboard().map(TaskOutcome::Refreshed)
    });
}

pub(super) fn start_task(
    state: &mut SakuraState,
    task: impl FnOnce() -> Result<TaskOutcome, String> + Send + 'static,
) {
    if state.busy {
        return;
    }
    state.busy = true;
    let messages = state.messages.clone();
    thread::spawn(move || {
        let result = task();
        if let Ok(mut queue) = messages.lock() {
            queue.push(TaskMessage::Finished(Box::new(result)));
        }
    });
}
