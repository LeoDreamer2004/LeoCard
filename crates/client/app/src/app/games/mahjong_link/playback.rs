use super::table::{STEP_X, STEP_Y, TILE_HEIGHT, TILE_WIDTH};
use crate::app::presentation::{
    ACCENT, MahjongTileMaterial, MahjongTileSurface, ease_out_cubic, rolling_score,
    score_gain_visual,
};
use crate::app::runtime::{ClientResource, ClientUpdateSet, ServerNotification};
use crate::app::shell::UiState;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_mahjong_link::LinkCell;
use leocard_protocol::{GameEvent, GameSnapshot, MahjongLinkEvent, MatchId, ServerEvent};
use leocard_ui::{UiComposer, spawn_node};

const PUSH_SECONDS: f32 = 0.28;
const LINK_SECONDS: f32 = 0.65;

#[derive(Resource, Default)]
pub(super) struct LinkPlayback {
    match_id: Option<MatchId>,
    sequence: Option<u128>,
    pub event: Option<MahjongLinkEvent>,
    pub started: f64,
    pub now: f64,
}

impl LinkPlayback {
    pub fn tile_offset(&self, cell: LinkCell) -> Vec2 {
        self.event
            .as_ref()
            .and_then(|event| {
                event
                    .outcome
                    .movements
                    .iter()
                    .find(|movement| movement.to == cell)
            })
            .map_or(Vec2::ZERO, |movement| {
                Vec2::new(
                    (f32::from(movement.from.column) - f32::from(cell.column)) * STEP_X,
                    0.0,
                )
            })
    }

    pub fn gained(&self, player: PlayerId) -> u16 {
        self.event
            .as_ref()
            .filter(|event| event.player == player)
            .map_or(0, |event| event.outcome.points)
    }
}

#[derive(Component)]
pub(super) struct LinkTileVisual {
    pub offset: Vec2,
    pub started: f64,
    pub selected: bool,
}

#[derive(Component)]
pub(super) struct LinkScoreVisual {
    pub before: u32,
    pub after: u32,
    pub started: f64,
}

#[derive(Component)]
pub(super) struct LinkGainVisual(pub f64);

#[derive(Component)]
struct LinkPathVisual(f64);

pub(super) fn register(app: &mut App) {
    app.init_resource::<LinkPlayback>()
        .add_systems(Update, sync.in_set(ClientUpdateSet::Sync))
        .add_systems(
            Update,
            (animate_tiles, animate_scores, animate_paths).in_set(ClientUpdateSet::Animate),
        );
}

fn sync(
    mut notifications: MessageReader<ServerNotification>,
    client: Option<Res<ClientResource>>,
    time: Res<Time>,
    mut playback: ResMut<LinkPlayback>,
    mut ui: ResMut<UiState>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::mahjong_link)
    else {
        notifications.clear();
        *playback = LinkPlayback::default();
        return;
    };
    if playback.match_id != Some(game.match_id) {
        *playback = LinkPlayback {
            match_id: Some(game.match_id),
            ..default()
        };
    }
    playback.now = time.elapsed_secs_f64();
    if playback
        .event
        .as_ref()
        .is_some_and(|event| event.turn != game.turn)
    {
        playback.event = None;
    }
    for notification in notifications.read() {
        let ServerEvent::GameEvent(GameEvent::MahjongLink(event)) = &notification.event else {
            continue;
        };
        let Some(context) = notification.game_context else {
            continue;
        };
        if context.match_id != game.match_id
            || event.turn != game.turn
            || playback
                .sequence
                .is_some_and(|sequence| sequence >= context.sequence)
        {
            continue;
        }
        playback.sequence = Some(context.sequence);
        playback.event = Some(event.clone());
        playback.started = playback.now;
        ui.dirty = true;
    }
    if playback.event.is_some() && playback.now - playback.started > 1.6 {
        playback.event = None;
        ui.dirty = true;
    }
}

fn animate_tiles(
    time: Res<Time>,
    mut tiles: Query<(
        &LinkTileVisual,
        &MahjongTileSurface,
        Option<&PickingInteraction>,
        &mut UiTransform,
    )>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    for (visual, surface, interaction, mut transform) in &mut tiles {
        let elapsed = (time.elapsed_secs_f64() - visual.started) as f32;
        let offset =
            visual.offset * (1.0 - ease_out_cubic((elapsed / PUSH_SECONDS).clamp(0.0, 1.0)));
        let hovered =
            interaction.is_some_and(|interaction| *interaction != PickingInteraction::None);
        transform.translation = Val2::px(
            offset.x,
            offset.y
                - if visual.selected {
                    4.0
                } else if hovered {
                    2.0
                } else {
                    0.0
                },
        );
        if let Some(mut material) = materials.get_mut(&surface.0) {
            material.params.x = if visual.selected || hovered { 1.0 } else { 0.0 };
        }
    }
}

fn animate_scores(
    time: Res<Time>,
    mut scores: Query<(&LinkScoreVisual, &mut Text)>,
    mut gains: Query<(&LinkGainVisual, &mut UiTransform, &mut TextColor)>,
) {
    for (visual, mut text) in &mut scores {
        let elapsed = (time.elapsed_secs_f64() - visual.started) as f32;
        text.0 = format!("{} 分", rolling_score(visual.before, visual.after, elapsed));
    }
    for (visual, mut transform, mut color) in &mut gains {
        let (pose, alpha) = score_gain_visual((time.elapsed_secs_f64() - visual.0) as f32);
        *transform = pose;
        color.0 = ACCENT.with_alpha(alpha);
    }
}

fn animate_paths(time: Res<Time>, mut paths: Query<(&LinkPathVisual, &mut BackgroundColor)>) {
    for (visual, mut color) in &mut paths {
        let progress = ((time.elapsed_secs_f64() - visual.0) as f32 / LINK_SECONDS).clamp(0.0, 1.0);
        color.0 = ACCENT.with_alpha(1.0 - progress);
    }
}

pub(super) fn path(commands: &mut UiComposer, board: Entity, playback: &LinkPlayback) {
    let Some(event) = &playback.event else {
        return;
    };
    if playback.now - playback.started > f64::from(LINK_SECONDS) {
        return;
    }
    for segment in event.outcome.path.windows(2) {
        let first = Vec2::new(
            f32::from(segment[0].column) * STEP_X + TILE_WIDTH * 0.5,
            f32::from(segment[0].row) * STEP_Y + TILE_HEIGHT * 0.5,
        );
        let second = Vec2::new(
            f32::from(segment[1].column) * STEP_X + TILE_WIDTH * 0.5,
            f32::from(segment[1].row) * STEP_Y + TILE_HEIGHT * 0.5,
        );
        let node = spawn_node(
            commands,
            board,
            Node {
                position_type: PositionType::Absolute,
                left: px(first.x.min(second.x) - 2.0),
                top: px(first.y.min(second.y) - 2.0),
                width: px((second.x - first.x).abs().max(4.0)),
                height: px((second.y - first.y).abs().max(4.0)),
                ..default()
            },
            Some(ACCENT),
        );
        commands.entity(node).insert((
            Pickable::IGNORE,
            ZIndex(20),
            LinkPathVisual(playback.started),
        ));
    }
}
