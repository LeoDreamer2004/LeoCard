use super::client::DrawGuessClient;
use super::libraries::WordLibraryUiAction;
use super::state::{DrawGuessUiState, DrawTool, can_message};
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_draw_guess::{DrawAction, DrawGuessRuleSet, Ink};
use leocard_protocol::{DrawGuessCommand, GameSnapshot};

#[derive(Clone)]
pub(crate) enum DrawGuessUiAction {
    UpdateRules(DrawGuessRuleSet),
    ChooseWord(u8),
    Tool(DrawTool),
    Color(Ink),
    Size(u8),
    Draw(DrawAction),
    Submit,
    Library(WordLibraryUiAction),
}

impl DomainUiAction for DrawGuessUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        if let UiAction::DrawGuess(action) = action {
            Some(action)
        } else {
            None
        }
    }

    fn rebuilds_ui(&self) -> bool {
        matches!(self, Self::Library(_))
    }
}

#[derive(SystemParam)]
pub(super) struct DrawGuessActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, DrawGuessUiState>,
    prediction: ResMut<'w, DrawGuessClient>,
}

pub(super) fn dispatch(
    mut actions: MessageReader<PressedUiAction>,
    mut context: DrawGuessActionContext,
) {
    dispatch_domain_actions::<DrawGuessUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<DrawGuessActionContext<'_>> for DrawGuessUiAction {
    fn handle(&self, context: &mut DrawGuessActionContext<'_>) {
        match self {
            Self::Library(_) => {}
            Self::Tool(tool) => context.ui.tool = *tool,
            Self::Color(color) => context.ui.select_color(*color),
            Self::Size(size) => context.ui.size = *size,
            Self::UpdateRules(rules) => {
                if let Some(client) = context.client.as_deref_mut() {
                    client.0.send(game_command(DrawGuessCommand::UpdateRules {
                        rules: *rules,
                    }));
                }
            }
            Self::ChooseWord(choice) => {
                if let Some(client) = context.client.as_deref_mut()
                    && let Some(game) = client
                        .0
                        .model()
                        .game_snapshot()
                        .and_then(GameSnapshot::draw_guess)
                {
                    let command = DrawGuessCommand::ChooseWord {
                        turn: game.state.turn,
                        choice: *choice,
                    };
                    client.0.send(game_command(command));
                }
            }
            Self::Draw(action) => {
                if let Some(client) = context.client.as_deref_mut()
                    && let Some(game) = client
                        .0
                        .model()
                        .game_snapshot()
                        .and_then(GameSnapshot::draw_guess)
                {
                    let turn = game.state.turn;
                    send_draw(client, &mut context.prediction, turn, action.clone());
                }
            }
            Self::Submit => {
                if let Some(client) = context.client.as_deref_mut()
                    && let Some(game) = client
                        .0
                        .model()
                        .game_snapshot()
                        .and_then(GameSnapshot::draw_guess)
                    && can_message(game)
                    && !context.ui.input.trim().is_empty()
                {
                    let command = DrawGuessCommand::Guess {
                        turn: game.state.turn,
                        text: context.ui.input.clone(),
                    };
                    client.0.send(game_command(command));
                    context.ui.submitted = Some(context.ui.input.clone());
                }
            }
        }
    }
}

pub(super) fn send_draw(
    client: &mut ClientResource,
    prediction: &mut DrawGuessClient,
    turn: u32,
    action: DrawAction,
) -> bool {
    if !prediction.predict(action.clone()) {
        return false;
    }
    client
        .0
        .send(game_command(DrawGuessCommand::Draw { turn, action }));
    true
}
