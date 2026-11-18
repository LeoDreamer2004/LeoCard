use super::super::actions::DrawGuessUiAction;
use super::actions::{WordLibraryUiAction, action};
use super::state::{LibraryField, LibraryWordCount, WordLibraryManager, WordLibraryRoom};
use crate::app::runtime::{ClientResource, GameNotice, GameStateMessage};
use crate::app::shell::{
    ConfirmationDialog, CozyModalKind, ModalAnimations, UiAction, UiState, advance_modal,
};
use bevy::prelude::*;
use bevy::text::EditableText;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{DrawGuessLibraryEvent, GameEvent, GameKind, ServerEvent, WordLibraryOffer};

pub(super) fn prompt_upload(dialog: &mut ConfirmationDialog, offer: &WordLibraryOffer) {
    dialog.show_choice(
        "接收词库？".into(),
        format!(
            "{}上传了「{}」，共 {} 个词。接收后保存到你的本地词库。",
            offer.sender_name,
            offer.library.title,
            offer.library.words.len()
        ),
        action(WordLibraryUiAction::ResolveUpload {
            id: offer.id,
            accept: true,
        }),
        action(WordLibraryUiAction::ResolveUpload {
            id: offer.id,
            accept: false,
        }),
        "接收",
    );
}

pub(in super::super) fn synchronize(
    client: Option<Res<ClientResource>>,
    mut events: MessageReader<GameStateMessage>,
    mut room: ResMut<WordLibraryRoom>,
    mut manager: ResMut<WordLibraryManager>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut shell: ResMut<UiState>,
    mut notices: MessageWriter<GameNotice>,
) {
    for event in events.read() {
        if let ServerEvent::GameEvent(GameEvent::DrawGuessLibrary(event)) = &event.0 {
            match event {
                DrawGuessLibraryEvent::State { selected, offers } => {
                    if &room.selected != selected || &room.offers != offers {
                        room.selected = selected.clone();
                        room.offers = offers.clone();
                        shell.dirty = true;
                    }
                }
                DrawGuessLibraryEvent::UploadResolved { title, accepted } => {
                    manager.status = format!(
                        "房主{}了词库「{title}」",
                        if *accepted { "接收" } else { "拒绝" }
                    );
                    notices.write(GameNotice(manager.status.clone()));
                    shell.dirty = true;
                }
            }
        }
    }
    let lobby = client
        .as_deref()
        .and_then(|client| match client.0.model().phase() {
            ClientPhaseRef::Lobby(lobby) if lobby.game == GameKind::DrawGuess => Some(lobby),
            _ => None,
        });
    let Some(lobby) = lobby else {
        if manager.open || manager.progress > 0.0 {
            manager.open = false;
            manager.progress = 0.0;
            shell.dirty = true;
        }
        room.offers.clear();
        manager.prompted = None;
        shell.dirty |= dialog.dismiss_if(|action| {
            matches!(action, UiAction::DrawGuess(DrawGuessUiAction::Library(_)))
        });
        return;
    };
    shell.dirty |= dialog.dismiss_if(|action| matches!(action,
        UiAction::DrawGuess(DrawGuessUiAction::Library(WordLibraryUiAction::ResolveUpload { id, .. }))
            if !room.offers.iter().any(|offer| offer.id == *id)
    ));
    let host = client
        .as_deref()
        .is_some_and(|client| client.0.model().you() == lobby.host);
    if host
        && !dialog.is_busy()
        && let Some(offer) = room.offers.first()
        && manager.prompted != Some(offer.id)
    {
        prompt_upload(&mut dialog, offer);
        manager.prompted = Some(offer.id);
        shell.dirty = true;
    }
}

pub(in super::super) fn sync_editor(
    editors: Query<(&LibraryField, &EditableText)>,
    mut manager: ResMut<WordLibraryManager>,
    mut statuses: Query<&mut Text, With<LibraryWordCount>>,
    mut shell: ResMut<UiState>,
) {
    let was_dirty = manager.dirty();
    for (field, editor) in &editors {
        let value = editor.value().to_string();
        let draft = match field {
            LibraryField::Title => &mut manager.title,
            LibraryField::Content => &mut manager.content,
        };
        if *draft != value {
            *draft = value;
            manager.status.clear();
        }
    }
    if was_dirty != manager.dirty() {
        shell.dirty = true;
    }
    let status = format!("{} 个词", manager.word_count());
    for mut label in &mut statuses {
        if label.0 != status {
            label.0 = status.clone();
        }
    }
}

pub(in super::super) fn animate(
    time: Res<Time>,
    mut manager: ResMut<WordLibraryManager>,
    mut animations: ResMut<ModalAnimations>,
    mut shell: ResMut<UiState>,
) {
    let before = manager.progress;
    let open = manager.open;
    advance_modal(&mut manager.progress, open, time.delta_secs());
    animations.set(CozyModalKind::DrawGuessLibraries, manager.progress);
    if !open && before > 0.0 && manager.progress == 0.0 {
        shell.dirty = true;
    }
}
