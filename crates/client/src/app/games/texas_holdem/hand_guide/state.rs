use super::view::render_hand_guide;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{CozyModalKind, ModalAnimations, advance_modal};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_client::ClientPhaseRef;
use leocard_protocol::GameSnapshot;
use leocard_texas_holdem::TexasHoldemRuleSet;

#[derive(Resource, Default)]
pub(in super::super) struct TexasHandGuideState {
    open: bool,
    progress: f32,
}

impl TexasHandGuideState {
    pub(in super::super) fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub(in super::super) fn close(&mut self) {
        self.open = false;
    }
}

#[derive(Component)]
pub(super) struct TexasHandGuideScroll;

#[derive(Component)]
pub(super) struct TexasHandGuideRoot(TexasHoldemRuleSet);

fn active_rules(client: Option<&ClientResource>) -> Option<TexasHoldemRuleSet> {
    let model = client?.0.model();
    if !matches!(
        model.phase(),
        ClientPhaseRef::Playing(GameSnapshot::TexasHoldem(_))
    ) {
        return None;
    }
    model.texas_holdem_rules().copied()
}

pub(super) fn advance_texas_hand_guide(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    mut guide: ResMut<TexasHandGuideState>,
    mut animations: ResMut<ModalAnimations>,
) {
    if active_rules(client.as_deref()).is_some() {
        let open = guide.open;
        advance_modal(&mut guide.progress, open, time.delta_secs());
    } else {
        *guide = TexasHandGuideState::default();
    }
    animations.set(CozyModalKind::TexasHandGuide, guide.progress);
}

pub(super) fn sync_texas_hand_guide(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    guide: Res<TexasHandGuideState>,
    roots: Query<(Entity, &TexasHandGuideRoot)>,
    assets: Res<UiAssets>,
) {
    let desired = if guide.open || guide.progress > 0.0 {
        active_rules(client.as_deref())
    } else {
        None
    };
    let mut retained = false;
    for (entity, root) in &roots {
        if Some(root.0) == desired {
            retained = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some(rules) = desired else {
        return;
    };
    if retained {
        return;
    }
    let root = commands
        .spawn((
            TexasHandGuideRoot(rules),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            GlobalZIndex(2200),
            FocusPolicy::Pass,
        ))
        .id();
    render_hand_guide(&mut commands, root, rules, guide.progress, &assets);
}
