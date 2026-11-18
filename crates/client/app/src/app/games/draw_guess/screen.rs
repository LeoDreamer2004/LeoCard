use super::assets::DrawGuessAssets;
use super::audio::DrawingAudio;
use super::canvas::CanvasRenderer;
use super::client::DrawGuessClient;
use super::libraries::{self, WordLibraryManager, WordLibraryRoom};
use super::state::{DrawGuessUiState, DrawingClock, can_guess};
use super::{
    actions, assets, audio, canvas, control_skin, feedback, history, input, lobby, palette, reveal,
    view, widgets,
};
use crate::app::games::{
    GameActionPrompt, GameLobbyContext, GameScreen, GameScreenChange, GameTableContext,
    GameUiLifecycle,
};
use crate::app::presentation::{ACCENT, DANGER};
use crate::app::runtime::{ClientResource, ClientUpdateSet, GameProjectionPlugin};
use crate::app::shell::{ModalAnimationSet, SocialUiState, UiActionSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use leocard_draw_guess::DrawGuessPhase;
use leocard_protocol::{DrawGuessSnapshot, GameSnapshot, LobbySnapshot, MatchId};
use leocard_ui::UiComposer;

pub(crate) struct DrawGuessPlugin;

impl Plugin for DrawGuessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameProjectionPlugin::<DrawGuessClient>::default())
            .init_resource::<DrawGuessUiState>()
            .init_resource::<WordLibraryManager>()
            .init_resource::<WordLibraryRoom>()
            .init_resource::<CanvasRenderer>()
            .init_resource::<DrawingAudio>()
            .init_resource::<feedback::DrawingFeedback>()
            .add_systems(Startup, assets::load)
            .add_systems(Update, control_skin::prepare.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                (
                    libraries::sync_editor,
                    input::guess_input,
                    input::draw_input,
                    palette::update_picker,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input)
                    .before(UiActionSet),
            )
            .add_systems(
                Update,
                (actions::dispatch, libraries::dispatch).in_set(UiActionSet),
            )
            .add_systems(Update, libraries::synchronize.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                libraries::animate.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                feedback::sync_feedback.in_set(ClientUpdateSet::Sync),
            )
            .add_systems(PostUpdate, history::scroll_latest.after(UiSystems::Layout))
            .add_systems(
                Update,
                (
                    canvas::sync_canvas,
                    canvas::sync_cursor,
                    audio::play,
                    audio::play_guess_feedback,
                    view::fit_canvas,
                    widgets::sync_buttons,
                    palette::sync_palette,
                    update_clock,
                    feedback::animate_feedback,
                    feedback::animate_history,
                    feedback::animate_choices,
                    reveal::animate.after(feedback::animate_feedback),
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}

#[derive(SystemParam)]
pub(crate) struct DrawGuessScreen<'w> {
    ui: ResMut<'w, DrawGuessUiState>,
    art: Res<'w, DrawGuessAssets>,
    libraries: Res<'w, WordLibraryManager>,
    library_room: Res<'w, WordLibraryRoom>,
    pub(super) feedback: Res<'w, feedback::DrawingFeedback>,
}

impl GameScreen for DrawGuessScreen<'_> {
    type Snapshot = DrawGuessSnapshot;

    fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        snapshot: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    ) {
        lobby::render(
            commands,
            root,
            snapshot,
            context,
            &self.libraries,
            &self.library_room,
        );
    }

    fn render_table(
        &mut self,
        commands: &mut UiComposer,
        root: Entity,
        game: &Self::Snapshot,
        context: GameTableContext<'_>,
    ) {
        view::render(commands, root, game, &self.ui, &self.art, context.into());
    }
}

impl GameActionPrompt for DrawGuessScreen<'_> {
    fn requires_action(&self, game: &Self::Snapshot) -> bool {
        (matches!(
            game.state.phase,
            DrawGuessPhase::Choosing | DrawGuessPhase::Drawing
        ) && game.state.artist == Some(game.you))
            || can_guess(game)
    }
}

impl GameUiLifecycle for DrawGuessScreen<'_> {
    fn reconcile(&mut self, client: Option<&ClientResource>, _social: &mut SocialUiState) {
        if let Some(game) = client
            .and_then(|client| client.0.model().game_snapshot())
            .and_then(GameSnapshot::draw_guess)
        {
            self.ui.reconcile(game);
        } else if self.ui.key.is_some() {
            *self.ui = DrawGuessUiState::default();
        }
    }
}

impl GameScreenChange for DrawGuessSnapshot {
    fn is_transient_change(&self, after: &Self) -> bool {
        let mut before = self.clone();
        before.state.remaining_ms = after.state.remaining_ms;
        before.state.canvas = after.state.canvas.clone();
        before == *after
    }
}

#[derive(Default)]
struct DrawingClockState {
    key: Option<(MatchId, u32)>,
    serial: u64,
    remaining_ms: f32,
}

fn update_clock(
    time: Res<Time>,
    client: Res<DrawGuessClient>,
    mut clock: Local<DrawingClockState>,
    mut labels: Query<(&mut Text, &mut TextColor), With<DrawingClock>>,
) {
    if clock.key != client.key || clock.serial != client.clock_serial {
        clock.key = client.key;
        clock.serial = client.clock_serial;
        clock.remaining_ms = client.clock_remaining_ms as f32;
    } else {
        clock.remaining_ms = (clock.remaining_ms - time.delta_secs() * 1000.0).max(0.0);
    }
    let value = (clock.remaining_ms / 1000.0).ceil() as u32;
    for (mut label, mut color) in &mut labels {
        let next = value.to_string();
        if label.0 != next {
            label.0 = next;
        }
        color.0 = if value <= 10 { DANGER } else { ACCENT };
    }
}
