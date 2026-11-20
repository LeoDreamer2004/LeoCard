use super::super::actions::DrawGuessUiAction;
use super::state::{LibrarySelection, WordLibraryManager, WordLibraryRoom};
use super::sync::prompt_upload;
use crate::app::runtime::{ClientResource, GameNotice};
use crate::app::shell::{
    ConfirmationDialog, DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState,
    dispatch_domain_actions, game_command,
};
use crate::platform::open_directory;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_draw_guess::WordLibrary;
use leocard_platform::storage as fs;
use leocard_protocol::DrawGuessCommand;

#[derive(Clone)]
pub(crate) enum WordLibraryUiAction {
    Open,
    Close,
    DiscardClose,
    Select(LibrarySelection),
    DiscardSelect(LibrarySelection),
    Save,
    Delete,
    DeleteConfirmed(String),
    Use,
    Upload,
    ReviewUpload(u64),
    ResolveUpload { id: u64, accept: bool },
    OpenDirectory,
    Refresh,
}

pub(super) fn action(action: WordLibraryUiAction) -> UiAction {
    UiAction::DrawGuess(DrawGuessUiAction::Library(action))
}

impl DomainUiAction for WordLibraryUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        if let UiAction::DrawGuess(DrawGuessUiAction::Library(action)) = action {
            Some(action)
        } else {
            None
        }
    }
}

#[derive(SystemParam)]
pub(in super::super) struct LibraryActionContext<'w> {
    manager: ResMut<'w, WordLibraryManager>,
    room: Res<'w, WordLibraryRoom>,
    client: Option<ResMut<'w, ClientResource>>,
    confirmation: ResMut<'w, ConfirmationDialog>,
    shell: ResMut<'w, UiState>,
    notices: MessageWriter<'w, GameNotice>,
}

pub(in super::super) fn dispatch(
    mut events: MessageReader<PressedUiAction>,
    mut context: LibraryActionContext,
) {
    dispatch_domain_actions::<WordLibraryUiAction, _>(&mut events, &mut context);
}

impl LibraryActionContext<'_> {
    fn send(&mut self, command: DrawGuessCommand) -> Result<(), String> {
        if self
            .client
            .as_deref_mut()
            .is_some_and(|client| client.0.send(game_command(command)))
        {
            Ok(())
        } else {
            Err("发送失败，请检查连接后重试".into())
        }
    }

    fn execute(&mut self, command: &WordLibraryUiAction) -> Result<(), String> {
        let manager = &mut self.manager;
        match command {
            WordLibraryUiAction::Open | WordLibraryUiAction::Refresh => {
                if manager.dirty() && matches!(command, WordLibraryUiAction::Refresh) {
                    return Err("请先保存当前修改".into());
                }
                let was_dirty = manager.dirty();
                let warnings = manager.store.reload()?;
                let selection = match &manager.selection {
                    LibrarySelection::Local(id)
                        if !manager.store.entries.iter().any(|entry| &entry.id == id) =>
                    {
                        LibrarySelection::Builtin
                    }
                    selection => selection.clone(),
                };
                if !was_dirty {
                    manager.select(selection);
                }
                manager.status = warnings.join("\n");
                manager.open = true;
            }
            WordLibraryUiAction::Close => {
                if manager.dirty() {
                    self.confirmation.show(
                        "放弃修改？".into(),
                        "未保存的词库内容将被丢弃。".into(),
                        action(WordLibraryUiAction::DiscardClose),
                    );
                } else {
                    manager.open = false;
                }
            }
            WordLibraryUiAction::DiscardClose => {
                manager.open = false;
                let selection = manager.selection.clone();
                manager.select(selection);
            }
            WordLibraryUiAction::Select(selection) => {
                if manager.dirty() {
                    self.confirmation.show(
                        "放弃修改？".into(),
                        "未保存的词库内容将被丢弃。".into(),
                        action(WordLibraryUiAction::DiscardSelect(selection.clone())),
                    );
                } else {
                    manager.select(selection.clone());
                }
            }
            WordLibraryUiAction::DiscardSelect(selection) => manager.select(selection.clone()),
            WordLibraryUiAction::Save => {
                if manager.selection == LibrarySelection::Builtin {
                    return Err("默认词库不可修改".into());
                }
                let previous = manager
                    .selected_library()
                    .map(|library| library.fingerprint());
                let library = WordLibrary::from_lines(&manager.title, &manager.content)
                    .map_err(|error| error.to_string())?;
                let id = match &manager.selection {
                    LibrarySelection::Local(id) => Some(id.clone()),
                    _ => None,
                };
                let saved = manager.store.save(id.as_deref(), library.clone())?;
                manager.select(LibrarySelection::Local(saved));
                manager.status = "已保存".into();
                if self.client.as_deref().is_some_and(|client| {
                    client.0.model().you() == client.0.model().lobby().and_then(|lobby| lobby.host)
                }) && previous == Some(self.room.selected.fingerprint)
                {
                    self.send(DrawGuessCommand::SelectLibrary {
                        library: Some(library),
                    })?;
                }
            }
            WordLibraryUiAction::Delete => {
                if let LibrarySelection::Local(id) = &manager.selection {
                    self.confirmation.show(
                        "删除词库？".into(),
                        format!("将删除本地词库「{}」。", manager.title),
                        action(WordLibraryUiAction::DeleteConfirmed(id.clone())),
                    );
                }
            }
            WordLibraryUiAction::DeleteConfirmed(id) => {
                let active = manager
                    .store
                    .entries
                    .iter()
                    .find(|entry| &entry.id == id)
                    .is_some_and(|entry| {
                        entry.library.fingerprint() == self.room.selected.fingerprint
                    });
                manager.store.delete(id)?;
                manager.select(LibrarySelection::Builtin);
                if active
                    && self.client.as_deref().is_some_and(|client| {
                        client.0.model().you()
                            == client.0.model().lobby().and_then(|lobby| lobby.host)
                    })
                {
                    self.send(DrawGuessCommand::SelectLibrary { library: None })?;
                }
            }
            WordLibraryUiAction::Use | WordLibraryUiAction::Upload => {
                if manager.dirty() {
                    return Err("请先保存当前修改".into());
                }
                let library = manager.selected_library().ok_or("请先保存词库")?;
                if matches!(command, WordLibraryUiAction::Use) {
                    let library =
                        (manager.selection != LibrarySelection::Builtin).then_some(library);
                    self.send(DrawGuessCommand::SelectLibrary { library })?;
                } else {
                    self.send(DrawGuessCommand::UploadLibrary { library })?;
                    self.manager.status = "已上传，等待房主确认".into();
                }
            }
            WordLibraryUiAction::ReviewUpload(id) => {
                if let Some(offer) = self.room.offers.iter().find(|offer| offer.id == *id) {
                    prompt_upload(&mut self.confirmation, offer);
                    manager.prompted = Some(*id);
                }
            }
            WordLibraryUiAction::ResolveUpload { id, accept } => {
                let offer = self
                    .room
                    .offers
                    .iter()
                    .find(|offer| offer.id == *id)
                    .ok_or("上传已失效")?
                    .clone();
                if *accept {
                    let existing = manager
                        .store
                        .entries
                        .iter()
                        .find(|entry| entry.library == offer.library)
                        .map(|entry| entry.id.clone());
                    let saved = if let Some(id) = existing {
                        id
                    } else {
                        manager.store.save(None, offer.library)?
                    };
                    if !manager.dirty() {
                        manager.select(LibrarySelection::Local(saved));
                    }
                    manager.status = "词库已接收并保存到本地".into();
                }
                self.send(DrawGuessCommand::ResolveLibraryUpload {
                    offer_id: *id,
                    accept: *accept,
                })?;
            }
            WordLibraryUiAction::OpenDirectory => {
                let directory = manager.store.directory()?;
                fs::create_dir_all(directory).map_err(|error| error.to_string())?;
                open_directory(directory)?;
            }
        }
        Ok(())
    }
}

impl UiActionHandler<LibraryActionContext<'_>> for WordLibraryUiAction {
    fn handle(&self, context: &mut LibraryActionContext<'_>) {
        let before = context.manager.status.clone();
        if let Err(error) = context.execute(self) {
            context.notices.write(GameNotice(error.clone()));
            context.manager.status = error;
            context.manager.open = true;
        } else if !context.manager.status.is_empty() && context.manager.status != before {
            context
                .notices
                .write(GameNotice(context.manager.status.clone()));
        }
        context.shell.dirty = true;
    }
}
