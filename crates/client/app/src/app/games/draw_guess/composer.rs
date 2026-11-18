use super::actions::DrawGuessUiAction;
use super::assets::DrawGuessAssets;
use super::state::{
    DrawGuessUiState, DrawingClock, GuessHistory, GuessInput, can_guess, can_message,
};
use super::widgets::{ButtonKind, ButtonSpec, button};
use crate::app::presentation::{ACCENT, DANGER, GAME_SIDE_CONTROL_SCALE, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_draw_guess::{DrawGuessPhase, GuessMessage};
use leocard_protocol::DrawGuessSnapshot;
use leocard_ui::{TextInput, UiComposer, spawn_node};

const CONTROL_HEIGHT: f32 = 36.0;

pub(super) fn render(
    commands: &mut UiComposer,
    paper: Entity,
    game: &DrawGuessSnapshot,
    ui: &DrawGuessUiState,
    art: &DrawGuessAssets,
    assets: &UiAssets,
) -> f32 {
    if game.state.phase == DrawGuessPhase::Finished {
        return 0.0;
    }
    let scale = GAME_SIDE_CONTROL_SCALE;
    let allowed = can_message(game);
    let row = spawn_node(
        commands,
        paper,
        Node {
            position_type: PositionType::Absolute,
            right: px(10),
            bottom: px(10),
            width: if allowed {
                px(320.0 * scale)
            } else {
                Val::Auto
            },
            max_width: percent(95),
            height: px(CONTROL_HEIGHT * scale),
            align_items: AlignItems::Center,
            column_gap: px(5.0 * scale),
            ..default()
        },
        None,
    );
    commands.entity(row).insert((GuessHistory, ZIndex(4)));
    if allowed {
        let mut input = TextInput::new("draw-guess-answer", &ui.input);
        input.placeholder = if can_guess(game) {
            "猜词"
        } else {
            "私密聊天"
        };
        input.max_characters = GuessMessage::MAX_CHARACTERS;
        input.font_size = 16.0 * scale;
        input.skin = Some(art.controls.input());
        let editor = input.spawn(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                height: px(CONTROL_HEIGHT * scale),
                padding: UiRect::horizontal(px(8.0 * scale)),
                ..default()
            },
            assets.text_input_style(),
        );
        commands.entity(editor).insert(GuessInput);
        let mut spec = ButtonSpec::new(
            "发送",
            Some(UiAction::DrawGuess(DrawGuessUiAction::Submit)),
            px(54),
        );
        spec.kind = ButtonKind::Submit;
        spec.height = CONTROL_HEIGHT;
        button(commands, row, spec, assets, art);
    }
    let seconds = game.state.remaining_ms.div_ceil(1000);
    let timer = add_text(
        commands,
        row,
        seconds.to_string(),
        22.0 * scale,
        if seconds <= 10 { DANGER } else { ACCENT },
        assets,
    );
    commands.entity(timer).insert((
        DrawingClock,
        Pickable::IGNORE,
        TextLayout::no_wrap().with_justify(Justify::Right),
        Node {
            width: px(42.0 * scale),
            flex_shrink: 0.0,
            ..default()
        },
    ));
    (CONTROL_HEIGHT + 6.0) * scale
}
