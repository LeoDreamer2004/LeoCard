use super::super::{MinecraftGuessAssets, MinecraftGuessUiAction, MinecraftGuessUiState};
use super::buttons::{GuessButton, GuessButtonSpec, GuessSubmit, add_guess_button, set_enabled};
use crate::app::presentation::UiComposer;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_minecraft_guess::{Catalogue, EntryKind};
use leocard_protocol::MinecraftGuessSnapshot;

#[derive(Component, Default)]
pub(in super::super) struct GuessCandidates(Option<CandidateQuery>);

#[derive(PartialEq)]
struct CandidateQuery {
    input: String,
    guesses: usize,
    active: bool,
    kind: EntryKind,
}

impl CandidateQuery {
    fn new(input: &str, game: &MinecraftGuessSnapshot) -> Self {
        Self {
            input: input.to_owned(),
            guesses: game.round.guesses.len(),
            active: game.can_guess() && game.rules.options().candidate_hints,
            kind: game.round.kind,
        }
    }
}

#[derive(Component)]
pub(in super::super) struct GuessCandidate(pub usize);

fn add_candidates(
    commands: &mut UiComposer,
    parent: Entity,
    game: &MinecraftGuessSnapshot,
    input: &str,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) {
    if !game.rules.options().candidate_hints || input.trim().is_empty() || !game.can_guess() {
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
    mut commands: UiComposer,
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
        let expected = CandidateQuery::new(&state.input, game);
        if rendered.0.as_ref() == Some(&expected) {
            continue;
        }
        rendered.0 = Some(expected);
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
    let enabled = game.can_guess() && !state.input.trim().is_empty();
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
