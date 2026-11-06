use super::GameScreen;
use crate::app::games::GameScreens;
use crate::app::presentation::{SummaryDescriptor, SummaryPlayback};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;

/// Game-owned result projection and presentation barriers, independent of visual style.
pub(crate) trait GameSummary: GameScreen {
    fn summary_descriptor(&self, game: &Self::Snapshot) -> Option<SummaryDescriptor>;

    fn summary_pause(&self, game: &Self::Snapshot) -> Option<f32>;
}

pub(crate) fn sync_summary_playback(
    client: Option<Res<ClientResource>>,
    screens: GameScreens,
    mut playback: ResMut<SummaryPlayback>,
) {
    *playback = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .map_or_else(SummaryPlayback::default, |game| {
            screens.summary_playback(game)
        });
}
