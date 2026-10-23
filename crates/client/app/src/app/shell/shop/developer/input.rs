use super::state::CoinInputField;
use crate::app::presentation::{TextInputAction, TextInputEvent};
use crate::app::runtime::PageErrorState;
use bevy::{
    input_focus::InputFocus,
    prelude::*,
    text::{EditableText, TextEdit},
};
use leocard_client::PlayerEconomy;

pub(super) fn sync_coin_input(
    mut events: MessageReader<TextInputEvent>,
    mut inputs: Query<(Entity, &mut EditableText), With<CoinInputField>>,
    mut economy: ResMut<PlayerEconomy>,
    mut focus: ResMut<InputFocus>,
    mut error: ResMut<PageErrorState>,
) {
    for event in events.read() {
        let Ok((_, input)) = inputs.get_mut(event.entity) else {
            continue;
        };
        if event.action == TextInputAction::Submit {
            let result = input
                .value()
                .to_string()
                .parse::<u32>()
                .map_err(|_| "金币数量须为 0～4294967295 的整数".to_owned())
                .and_then(|coins| economy.set_developer_coins(coins));
            if let Err(reason) = result {
                error.error = Some(reason);
                continue;
            }
            error.error = None;
        }
        focus.clear();
    }
    let value = economy.coins().to_string();
    for (entity, mut input) in &mut inputs {
        if focus.get() != Some(entity) && input.value() != value.as_str() {
            input.clear();
            input.queue_edit(TextEdit::Insert(value.clone().into()));
        }
    }
}
