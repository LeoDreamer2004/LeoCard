use super::super::{TexasRaiseAdjustButton, TexasTableLayout, TexasUiAction, TexasUiState};
use super::buttons::add_texas_action_button;
use super::slider::add_raise_slider;
use super::{TexasRaiseInput, raise_sizing};
use crate::app::presentation::ButtonKind;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui_widgets::SelectAllOnFocus;
use leocard_protocol::{TexasPhaseView, TexasPlayerView, TexasSnapshot};
use leocard_texas_holdem::{TexasAction, TexasRaiseSizing};
use leocard_ui::{TextInput, UiComposer, spawn_node};

pub(in super::super) fn add_texas_actions(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    own: &TexasPlayerView,
    ui: &mut TexasUiState,
    assets: &UiAssets,
) {
    if !matches!(game.phase, TexasPhaseView::Betting { .. })
        || game.blind_to_post.is_some()
        || game.current_player != Some(game.you)
        || ui.raise_pending.is_some()
    {
        return;
    }
    let scale = TexasTableLayout::ACTION_SCALE;
    let actions = commands.keyed_child(
        table,
        "betting-controls",
        (
            ZIndex(80),
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(TexasTableLayout::ACTION_BOTTOM),
                width: px(496.0 * scale),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0 * scale),
                ..default()
            },
            UiTransform::from_translation(Val2::px(-248.0 * scale, 0.0)),
        ),
    );
    let sizing = raise_sizing(game);
    if let Some(sizing) = sizing {
        ui.raise_to = sizing.normalize(ui.raise_to).expect("affordable raise");
        if ui.raise_editor_open {
            add_raise_editor(commands, actions, game, sizing, ui.raise_to, assets);
        }
    }
    let row = spawn_node(
        commands,
        actions,
        Node {
            width: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8.0 * scale),
            ..default()
        },
        None,
    );
    let size = Vec2::new(118.0, 50.0);
    if ui.raise_editor_open && sizing.is_some() {
        for (label, action, kind) in [
            ("确认", TexasUiAction::Raise, ButtonKind::Primary),
            ("取消", TexasUiAction::CancelRaise, ButtonKind::Pass),
        ] {
            add_texas_action_button(commands, row, label, action, kind, size, assets);
        }
        return;
    }
    add_texas_action_button(
        commands,
        row,
        "弃牌",
        TexasUiAction::Act(TexasAction::Fold),
        ButtonKind::Pass,
        size,
        assets,
    );
    let call = if game.amount_to_call == 0 {
        "过牌".to_owned()
    } else {
        format!("跟注 {}", game.amount_to_call.min(own.stack))
    };
    add_texas_action_button(
        commands,
        row,
        &call,
        TexasUiAction::Act(if game.amount_to_call == 0 {
            TexasAction::Check
        } else {
            TexasAction::Call
        }),
        ButtonKind::Secondary,
        size,
        assets,
    );
    if sizing.is_some() {
        add_texas_action_button(
            commands,
            row,
            "加注",
            TexasUiAction::BeginRaise,
            ButtonKind::Primary,
            size,
            assets,
        );
    }
    if game.all_in_allowed {
        add_texas_action_button(
            commands,
            row,
            "全下",
            TexasUiAction::Act(TexasAction::AllIn),
            ButtonKind::Warning,
            size,
            assets,
        );
    }
}

fn add_raise_editor(
    commands: &mut UiComposer,
    parent: Entity,
    game: &TexasSnapshot,
    sizing: TexasRaiseSizing,
    target: u32,
    assets: &UiAssets,
) {
    let scale = TexasTableLayout::ACTION_SCALE;
    let minimum = sizing.minimum();
    let maximum = sizing.maximum().expect("affordable raise");
    let step = game
        .minimum_raise_to
        .saturating_sub(game.current_bet)
        .max(1);
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(50.0 * scale),
            align_items: AlignItems::Center,
            column_gap: px(6.0 * scale),
            ..default()
        },
        None,
    );
    for direction in [-1, 1] {
        if direction == 1 {
            add_raise_slider(commands, row, sizing, target, assets);
        }
        let next = if direction < 0 {
            target.saturating_sub(step).max(minimum)
        } else {
            target.saturating_add(step).min(maximum)
        };
        let button = add_texas_action_button(
            commands,
            row,
            if direction < 0 { "−" } else { "+" },
            TexasUiAction::SetRaiseTo(next),
            ButtonKind::Primary,
            Vec2::new(38.0, 38.0),
            assets,
        );
        commands.entity(button).insert(TexasRaiseAdjustButton {
            direction,
            step,
            minimum,
            maximum,
        });
    }
    let field = spawn_node(
        commands,
        row,
        Node {
            width: px(64.0 * scale),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    let value = target.to_string();
    let mut input = TextInput::new("texas.raise-to", &value);
    input.max_characters = 10;
    input.filter = |character| character.is_ascii_digit();
    input.font_size = 17.0 * scale;
    let editor = input.spawn(
        commands,
        field,
        Node {
            width: percent(100),
            height: px(33.0 * scale),
            padding: UiRect::axes(px(12.0 * scale), px(3.0 * scale)),
            ..default()
        },
        assets.text_input_style(),
    );
    commands.entity(editor).insert((
        TexasRaiseInput(sizing),
        SelectAllOnFocus,
        TextLayout::no_wrap().with_justify(Justify::Center),
    ));
}
