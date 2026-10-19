use super::super::MinecraftGuessUiAction;
use crate::app::presentation::{TEXT, UiComposer, add_text, spawn_node};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{LobbyUiAction, PressedUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_client::NetworkState;
use leocard_minecraft_guess::GuessMode;
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub(in super::super) struct GraphGeneration {
    waiting: bool,
    previous: Option<MatchId>,
    rejection: u64,
    elapsed: f32,
}

#[derive(Component)]
pub(in super::super) struct GenerationOverlay;

#[derive(Component)]
pub(in super::super) struct GenerationSpinner;

pub(in super::super) fn begin_generation_notice(
    mut actions: MessageReader<PressedUiAction>,
    client: Option<Res<ClientResource>>,
    mut generation: ResMut<GraphGeneration>,
) {
    let Some(client) = client.as_deref() else {
        actions.clear();
        return;
    };
    for action in actions.read() {
        let initial = matches!(&action.0, UiAction::Lobby(LobbyUiAction::StartGame))
            && client.0.model().lobby().is_some_and(|lobby| {
                lobby
                    .rules
                    .minecraft_guess()
                    .is_some_and(|rules| rules.mode == GuessMode::Graph)
            });
        let rematch = matches!(
            &action.0,
            UiAction::MinecraftGuess(MinecraftGuessUiAction::Again)
                | UiAction::Lobby(LobbyUiAction::PlayAgain)
        ) && client.0.model().minecraft_guess_game().is_some_and(|game| {
            game.graph.is_some()
                && game.round.finished()
                && game
                    .players
                    .iter()
                    .all(|player| player.id == game.you || player.ready)
        });
        if (initial || rematch) && matches!(client.0.state(), NetworkState::Connected(_)) {
            *generation = GraphGeneration {
                waiting: true,
                previous: client.0.model().minecraft_guess_game().map(|g| g.match_id),
                rejection: client.0.model().rejection_serial(),
                elapsed: 0.0,
            };
        }
    }
}

pub(in super::super) fn show_generation_notice(
    mut commands: UiComposer,
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut generation: ResMut<GraphGeneration>,
    mut overlays: Query<(Entity, &mut BackgroundColor), With<GenerationOverlay>>,
    mut spinners: Query<&mut UiTransform, With<GenerationSpinner>>,
) {
    if generation.waiting
        && !client.as_deref().is_some_and(|client| {
            matches!(client.0.state(), NetworkState::Connected(_))
                && (generation.previous.is_some()
                    || client.0.model().lobby().is_some_and(|lobby| {
                        lobby
                            .rules
                            .minecraft_guess()
                            .is_some_and(|rules| rules.mode == GuessMode::Graph)
                    }))
                && client.0.model().rejection_serial() == generation.rejection
                && client.0.model().minecraft_guess_game().map(|g| g.match_id)
                    == generation.previous
        })
    {
        generation.waiting = false;
    }
    if !generation.waiting {
        for (entity, _) in &overlays {
            commands.entity(entity).despawn();
        }
        return;
    }
    generation.elapsed += time.delta_secs();
    if overlays.is_empty() {
        let root = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(
                    0.025,
                    0.055,
                    0.04,
                    (generation.elapsed / 0.16).min(1.0) * 0.85,
                )),
                GlobalZIndex(1600),
                Pickable::default(),
                GenerationOverlay,
            ))
            .id();
        let panel = spawn_node(
            &mut commands,
            root,
            Node {
                padding: UiRect::axes(px(34), px(25)),
                align_items: AlignItems::Center,
                column_gap: px(20),
                border_radius: BorderRadius::all(px(14)),
                border: UiRect::all(px(1)),
                ..default()
            },
            Some(Color::srgb(0.075, 0.13, 0.10)),
        );
        commands.entity(panel).insert((
            BorderColor::all(Color::srgb(0.32, 0.45, 0.34)),
            Pickable::IGNORE,
        ));
        let spinner = spawn_node(
            &mut commands,
            panel,
            Node {
                width: px(26),
                height: px(26),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            None,
        );
        commands.entity(spinner).insert((
            GenerationSpinner,
            Pickable::IGNORE,
            BorderColor {
                top: Color::srgb(0.74, 0.81, 0.59),
                right: Color::srgba(0.4, 0.55, 0.43, 0.3),
                bottom: Color::srgba(0.4, 0.55, 0.43, 0.3),
                left: Color::srgba(0.4, 0.55, 0.43, 0.3),
            },
        ));
        add_text(&mut commands, panel, "图谱生成中…", 21.0, TEXT, &assets);
    }
    for (_, mut background) in &mut overlays {
        background.0 = Color::srgba(
            0.025,
            0.055,
            0.04,
            (generation.elapsed / 0.16).min(1.0) * 0.85,
        );
    }
    for mut spinner in &mut spinners {
        spinner.rotation = Rot2::radians(generation.elapsed * 3.0);
    }
}
