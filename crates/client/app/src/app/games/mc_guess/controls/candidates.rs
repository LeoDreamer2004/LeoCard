use super::super::{McGuessAssets, McGuessUiAction, McGuessUiState};
use super::buttons::{GuessButton, GuessButtonSpec, GuessSubmit, add_guess_button, set_enabled};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiAction;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_mc_guess::{McGuessCatalogue, McGuessEntryKind, McGuessRecord, McGuessRoundView};
use leocard_protocol::{GameSnapshot, MatchId, McGuessSnapshot};

use leocard_ui::UiComposer;

#[cfg(test)]
mod tests;

#[derive(Component, Default)]
pub(in super::super) struct GuessCandidates {
    query: Option<CandidateQuery>,
    count: usize,
}

#[derive(PartialEq)]
struct CandidateQuery {
    match_id: MatchId,
    game_index: u8,
    input: String,
    guesses: usize,
    active: bool,
    can_submit: bool,
    kind: McGuessEntryKind,
    node: Option<u16>,
}

impl CandidateQuery {
    fn new(state: &McGuessUiState, game: &McGuessSnapshot, round: &McGuessRoundView) -> Self {
        Self {
            match_id: game.match_id,
            game_index: game.progress.game_index,
            input: state.input.clone(),
            guesses: round.guesses.len(),
            active: state.can_suggest(game) && game.rules.options().candidate_hints,
            can_submit: state.can_guess(game),
            kind: round.kind,
            node: state.viewed_node(game),
        }
    }
}

#[derive(Component)]
pub(in super::super) struct GuessCandidate(pub usize);

#[derive(SystemParam)]
pub(in super::super) struct GuessControlAssets<'w> {
    ui: Res<'w, UiAssets>,
    game: Res<'w, McGuessAssets>,
}

fn add_candidates(
    commands: &mut UiComposer,
    parent: Entity,
    query: &CandidateQuery,
    guesses: &[McGuessRecord],
    assets: &UiAssets,
    game_assets: &McGuessAssets,
) -> usize {
    if !query.active || query.input.trim().is_empty() {
        return 0;
    }
    let mut count = 0;
    for (index, entry) in McGuessCatalogue::bundled()
        .search(query.kind, &query.input)
        .filter(|entry| {
            !guesses
                .iter()
                .any(|guess| guess.result.has_entry(&entry.id))
        })
        .take(if query.node.is_some() { 3 } else { 5 })
        .enumerate()
    {
        let button = add_guess_button(
            commands,
            parent,
            GuessButtonSpec {
                label: &entry.name,
                action: query
                    .can_submit
                    .then(|| McGuessUiAction::Guess(Some(entry.id.clone()))),
                width: Val::Auto,
                height: 38.0,
                icon: Some(game_assets.icon(entry)),
            },
            assets,
            game_assets,
        );
        commands.entity(button).insert(GuessCandidate(index));
        count += 1;
    }
    count
}

pub(in super::super) fn sync_guess_controls(
    mut commands: UiComposer,
    client: Option<Res<ClientResource>>,
    state: Res<McGuessUiState>,
    assets: GuessControlAssets,
    mut candidates: Query<(Entity, &mut GuessCandidates, Option<&Children>)>,
    mut submits: Query<(Entity, &mut GuessButton), With<GuessSubmit>>,
    buttons: Query<(), With<GuessCandidate>>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::mc_guess)
    }) else {
        return;
    };
    let round = state.viewed_round(game);
    for (parent, mut rendered, children) in &mut candidates {
        let expected = CandidateQuery::new(&state, game, &round);
        let complete = children.map_or(rendered.count == 0, |children| {
            children.len() == rendered.count && children.iter().all(|child| buttons.contains(child))
        });
        if rendered.query.as_ref() == Some(&expected) && complete {
            continue;
        }
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        rendered.count = add_candidates(
            &mut commands,
            parent,
            &expected,
            &round.guesses,
            &assets.ui,
            &assets.game,
        );
        rendered.query = Some(expected);
    }
    let enabled = state.can_guess(game) && !state.input.trim().is_empty();
    for (entity, mut submit) in &mut submits {
        if submit.enabled == enabled {
            continue;
        }
        submit.enabled = enabled;
        commands
            .entity(entity)
            .insert(UiAction::McGuess(McGuessUiAction::Guess(None)));
        set_enabled(&mut commands, entity, enabled);
    }
}
