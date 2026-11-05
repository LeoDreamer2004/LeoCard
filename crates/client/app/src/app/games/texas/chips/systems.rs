use super::{ChipZone, TexasChipSprite, TexasChipTableState, TexasChipUpdates};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::{GameRules, GameSnapshot};

pub(crate) fn sync_texas_chip_state(
    client: Option<Res<ClientResource>>,
    mut updates: ResMut<TexasChipUpdates>,
    mut state: ResMut<TexasChipTableState>,
) {
    let Some(client) = client.as_deref() else {
        state.reset();
        *updates = TexasChipUpdates::default();
        return;
    };
    let snapshot = client
        .0
        .model()
        .game_snapshot()
        .and_then(GameSnapshot::texas);
    let Some(snapshot) = snapshot else {
        if state.match_id.is_some() {
            state.reset();
        }
        return;
    };
    let side_pots = client
        .0
        .model()
        .game_rules()
        .and_then(GameRules::texas)
        .is_none_or(|rules| rules.side_pots);
    let ready = updates.take();
    if state.match_id != Some(snapshot.match_id)
        || state.hand_number != snapshot.hand_number
        || state.you != Some(snapshot.you)
    {
        // Do not replay queued intro/previous-hand history on first displaying
        // this hand. Events newer than this snapshot remain pending in updates.
        state.initialize(snapshot, side_pots);
        return;
    }
    for (snapshot, events) in ready {
        state.observe(&snapshot, events, side_pots);
    }
}

pub(crate) fn animate_texas_chip_sprites(
    time: Res<Time>,
    mut state: ResMut<TexasChipTableState>,
    mut sprites: Query<(
        Entity,
        &TexasChipSprite,
        &mut Node,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();
    if let Some(division) = state.division.as_mut() {
        division.elapsed += delta;
        if division.elapsed >= 0.72 {
            state.division = None;
        }
    }
    for label in state.actions.values_mut() {
        label.elapsed += delta;
    }
    for chip in &mut state.chips {
        let Some(mut motion) = chip.motion else {
            continue;
        };
        motion.elapsed += delta;
        let progress = ((motion.elapsed - motion.delay) / motion.duration).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        chip.position = motion.start.lerp(motion.target, eased);
        chip.rotation =
            motion.start_rotation + (motion.target_rotation - motion.start_rotation) * eased;
        if progress >= 1.0 {
            chip.motion = None;
        } else {
            chip.motion = Some(motion);
        }
    }
    state
        .chips
        .retain(|chip| chip.zone != ChipZone::Retired || chip.motion.is_some());

    for (entity, marker, mut node, mut transform, mut visibility) in &mut sprites {
        let Some(chip) = state.chips.iter().find(|chip| chip.id == marker.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let visible = matches!(
            chip.zone,
            ChipZone::Bet(_) | ChipZone::Pot(_) | ChipZone::Retired
        ) || chip.motion.is_some();
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.left = px(chip.position.x);
        node.top = px(chip.position.y);
        transform.rotation = Rot2::radians(chip.rotation);
    }
}
