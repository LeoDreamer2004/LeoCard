use super::super::{MinecraftGuessAssets, MinecraftGuessUiAction, MinecraftGuessUiState};
use super::buttons::{GuessButton, GuessButtonSpec, GuessSubmit, add_guess_button, set_enabled};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_minecraft_guess::Catalogue;
use leocard_protocol::MinecraftGuessSnapshot;

#[derive(Component)]
pub(in super::super) struct GuessCandidates(pub String);

#[derive(Component)]
pub(in super::super) struct GuessCandidate(pub usize);

pub(super) fn add_candidates(
    commands: &mut Commands,
    parent: Entity,
    game: &MinecraftGuessSnapshot,
    input: &str,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) {
    if !game.rules.candidate_hints
        || input.trim().is_empty()
        || game.round.current_player != Some(game.you.0)
    {
        return;
    }
    for (index, entry) in Catalogue::bundled()
        .search(game.round.kind, input)
        .filter(|entry| {
            !game
                .round
                .guesses
                .iter()
                .any(|guess| guess.result.has_entry(&entry.id))
        })
        .take(5)
        .enumerate()
    {
        let button = add_guess_button(
            commands,
            parent,
            GuessButtonSpec {
                label: &entry.name,
                action: Some(MinecraftGuessUiAction::Guess(Some(entry.id.clone()))),
                width: Val::Auto,
                height: 38.0,
                icon: Some(game_assets.icon(entry)),
            },
            assets,
            game_assets,
        );
        commands.entity(button).insert(GuessCandidate(index));
    }
}

pub(in super::super) fn sync_guess_controls(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    state: Res<MinecraftGuessUiState>,
    assets: Res<UiAssets>,
    game_assets: Res<MinecraftGuessAssets>,
    mut candidates: Query<(Entity, &mut GuessCandidates, Option<&Children>)>,
    mut submits: Query<(Entity, &mut GuessButton), With<GuessSubmit>>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game())
    else {
        return;
    };
    for (parent, mut rendered, children) in &mut candidates {
        if rendered.0 == state.input {
            continue;
        }
        rendered.0.clone_from(&state.input);
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        add_candidates(
            &mut commands,
            parent,
            game,
            &state.input,
            &assets,
            &game_assets,
        );
    }
    let enabled = game.round.current_player == Some(game.you.0) && !state.input.trim().is_empty();
    for (entity, mut submit) in &mut submits {
        if submit.enabled == enabled {
            continue;
        }
        submit.enabled = enabled;
        commands
            .entity(entity)
            .insert(UiAction::MinecraftGuess(MinecraftGuessUiAction::Guess(
                None,
            )));
        set_enabled(&mut commands, entity, enabled);
    }
}
