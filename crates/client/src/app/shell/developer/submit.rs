use super::{
    DeveloperHandInput, DeveloperHandInputField, parse_developer_hand, parse_developer_mahjong_hand,
};
use crate::app::presentation::{TextInputAction, TextInputEvent};
use crate::app::runtime::{ClientResource, PageErrorState};
use crate::app::shell::game_command;
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText};
use leocard_mahjong::MahjongHandReplacementError;
use leocard_protocol::{MahjongCommand, QiGui523Command};

pub(crate) fn submit_developer_hand(
    developer_hand: &mut DeveloperHandInput,
    page_error: &mut PageErrorState,
    client: Option<&mut ClientResource>,
) {
    let input = developer_hand.value.trim().to_owned();
    if input.is_empty() {
        page_error.error = None;
        return;
    }
    let Some(client) = client else {
        return;
    };
    let command = if let Some(game) = client.0.model().mahjong_game() {
        parse_developer_mahjong_hand(&input).and_then(|tiles| {
            if tiles.len() != game.your_hand.len() {
                return Err(MahjongHandReplacementError::WrongTileCount {
                    expected: game.your_hand.len() as u16,
                    actual: tiles.len() as u16,
                }
                .to_string());
            }
            Ok(game_command(MahjongCommand::SetDeveloperHand { tiles }))
        })
    } else {
        parse_developer_hand(&input)
            .map(|cards| game_command(QiGui523Command::SetDeveloperHand { cards }))
    };
    match command {
        Ok(command) => {
            if client.0.send(command) {
                page_error.error = None;
                developer_hand.value.clear();
            } else {
                page_error.error = Some("当前未连接，无法编辑开发者手牌".to_owned());
            }
        }
        Err(error) => page_error.error = Some(error),
    }
}

pub(super) fn sync_developer_input(
    mut events: MessageReader<TextInputEvent>,
    mut inputs: Query<&mut EditableText, With<DeveloperHandInputField>>,
    mut draft: ResMut<DeveloperHandInput>,
    mut error: ResMut<PageErrorState>,
    mut client: Option<ResMut<ClientResource>>,
    mut focus: ResMut<InputFocus>,
) {
    for input in &inputs {
        if input.value() != draft.value.as_str() {
            draft.value = input.value().to_string();
        }
    }
    for event in events.read() {
        let Ok(mut input) = inputs.get_mut(event.entity) else {
            continue;
        };
        if event.action == TextInputAction::Submit {
            submit_developer_hand(&mut draft, &mut error, client.as_deref_mut());
            if draft.value.is_empty() {
                input.clear();
            }
        }
        focus.clear();
    }
}
