use super::super::assets::EntryIcons;
use super::super::{MinecraftGuessAssets, MinecraftGuessUiAction};
use crate::app::presentation::{MUTED, SilentButton, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::input_focus::InputFocus;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;
use leocard_ui::{UiComposer, spawn_node};

#[derive(Component)]
pub(in super::super) struct GuessButton {
    pub enabled: bool,
    pub label: Entity,
}

#[derive(Component)]
pub(in super::super) struct GuessSubmit;

pub(in super::super) struct GuessButtonSpec<'a> {
    pub label: &'a str,
    pub action: Option<MinecraftGuessUiAction>,
    pub width: Val,
    pub height: f32,
    pub icon: Option<ImageNode>,
}

pub(in super::super) fn add_guess_button(
    commands: &mut UiComposer,
    parent: Entity,
    spec: GuessButtonSpec<'_>,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) -> Entity {
    let enabled = spec.action.is_some();
    let button = spawn_node(
        commands,
        parent,
        Node {
            width: spec.width,
            height: px(spec.height),
            flex_shrink: 0.0,
            padding: UiRect::horizontal(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    commands.entity(button).insert(Button);
    if matches!(
        spec.action,
        Some(MinecraftGuessUiAction::Guess(_) | MinecraftGuessUiAction::Skip)
    ) {
        commands.entity(button).insert(SilentButton);
    }
    if let Some(action) = spec.action {
        commands
            .entity(button)
            .insert(UiAction::MinecraftGuess(action));
    }
    if let Some(icon) = spec.icon {
        EntryIcons::spawn(commands, button, icon, 32.0);
    }
    let label = add_text(commands, button, spec.label, 18.0, TEXT, assets);
    commands
        .entity(label)
        .insert((Pickable::IGNORE, TextLayout::no_wrap()));
    commands.entity(button).insert((
        GuessButton { enabled, label },
        game_assets.button(enabled, false),
    ));
    set_enabled(commands, button, enabled);
    commands
        .entity(label)
        .insert(TextColor(if enabled { TEXT } else { MUTED }));
    button
}

pub(super) fn set_enabled(commands: &mut UiComposer, button: Entity, enabled: bool) {
    let mut entity = commands.entity(button);
    if enabled {
        entity.remove::<InteractionDisabled>();
    } else {
        entity.insert(InteractionDisabled);
    }
}

pub(in super::super) fn sync_guess_button_skins(
    assets: Res<MinecraftGuessAssets>,
    focus: Res<InputFocus>,
    mut buttons: Query<(
        Entity,
        Ref<GuessButton>,
        Ref<PickingInteraction>,
        &mut ImageNode,
    )>,
    mut labels: Query<&mut TextColor>,
) {
    for (entity, button, interaction, mut image) in &mut buttons {
        if !focus.is_changed() && !button.is_changed() && !interaction.is_changed() {
            continue;
        }
        let highlighted = button.enabled
            && (*interaction != PickingInteraction::None || focus.get() == Some(entity));
        *image = assets.button(button.enabled, highlighted);
        if let Ok(mut color) = labels.get_mut(button.label) {
            color.0 = if !button.enabled {
                MUTED
            } else if highlighted {
                Color::srgb(0.08, 0.10, 0.16)
            } else {
                TEXT
            };
        }
    }
}
