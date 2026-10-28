use super::state::{PlayEffectKind, PlayEffectRoot, PlayEffectState};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;

pub(crate) fn advance_play_effect(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    mut effect: ResMut<PlayEffectState>,
    mut roots: Query<&mut Visibility, With<PlayEffectRoot>>,
) {
    let match_id = client
        .as_deref()
        .and_then(|client| client.0.model().active_game_meta())
        .map(|meta| meta.match_id);
    if effect
        .active
        .as_ref()
        .is_some_and(|active| Some(active.match_id) != match_id)
    {
        effect.active = None;
    }
    if effect.active.is_none() {
        for mut visibility in &mut roots {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        }
        return;
    }
    let Some(active) = effect.active.as_mut() else {
        unreachable!("active play effect was checked above");
    };
    active.elapsed += time.delta_secs();
    let elapsed = active.elapsed;
    let duration: f32 = match active.kind {
        PlayEffectKind::HeavenBomb => 2.25,
        PlayEffectKind::Bomb => 1.35,
        PlayEffectKind::Airplane => 1.38,
        _ => 1.18,
    };
    let duration = if active.kind.is_sequence() {
        duration.max(active.card_count.saturating_sub(1) as f32 * 0.055 + 0.29)
    } else {
        duration
    };
    if elapsed >= duration {
        effect.active = None;
        for mut visibility in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    for mut visibility in &mut roots {
        if *visibility != Visibility::Visible {
            *visibility = Visibility::Visible;
        }
    }
}
