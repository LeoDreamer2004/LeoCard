use super::assets::DrawGuessAssets;
use super::client::DrawGuessClient;
use super::input::normalized;
use super::state::{BrushCursor, CanvasPaper, DrawGuessUiState, DrawTool, GuessHistory, can_draw};
use crate::app::runtime::ClientResource;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_draw_guess::{CANVAS_WIDTH, CanvasDocument, DrawOperation, RasterCanvas};
use leocard_protocol::{ClientCommand, GameSnapshot, MatchId};

#[derive(Resource, Default)]
pub(super) struct CanvasRenderer {
    pub raster: Option<RasterCanvas>,
    key: Option<(MatchId, u32)>,
    previous: Option<CanvasDocument>,
}

pub(super) fn sync_canvas(
    mut state: ResMut<DrawGuessClient>,
    mut renderer: ResMut<CanvasRenderer>,
    assets: Res<DrawGuessAssets>,
    mut images: ResMut<Assets<Image>>,
    mut client: Option<ResMut<ClientResource>>,
) {
    if state.needs_snapshot
        && !state.snapshot_requested
        && let Some(client) = client.as_deref_mut()
    {
        client.0.send(ClientCommand::RequestSnapshot);
        state.snapshot_requested = true;
    }
    if renderer.key != state.key {
        renderer.key = state.key;
        renderer.previous = None;
        renderer.raster = None;
    }
    if state.key.is_none() {
        return;
    }
    let document = state.predicted();
    if renderer.previous.as_ref() == Some(&document) {
        return;
    }
    let incremental = renderer.previous.as_ref().and_then(|previous| {
        if previous.operations() != document.operations() {
            return None;
        }
        let pending = document.pending()?;
        let start = if let Some(before) = previous.pending() {
            if before.id != pending.id || before.points.len() > pending.points.len() {
                return None;
            }
            before.points.len()
        } else {
            0
        };
        Some((pending.clone(), start))
    });
    let just_finished = renderer.previous.as_ref().is_some_and(|previous| {
        let Some(pending) = previous.pending() else {
            return false;
        };
        document.pending().is_none()
            && document.operations().len() == previous.operations().len() + 1
            && document.operations()[..previous.operations().len()] == *previous.operations()
            && document.operations().last() == Some(&DrawOperation::Stroke(pending.clone()))
    });
    let raster = renderer.raster.get_or_insert_with(RasterCanvas::default);
    if let Some((stroke, start)) = incremental {
        raster.append_stroke(&stroke, start);
    } else if !just_finished {
        raster.render(&document);
    }
    if let Some(mut image) = images.get_mut(&assets.canvas) {
        image.data = Some(raster.pixels.clone());
    }
    renderer.previous = Some(document);
}

pub(super) fn sync_cursor(
    ui: Res<DrawGuessUiState>,
    client: Option<Res<ClientResource>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    papers: Query<(&ComputedNode, &UiGlobalTransform, &PickingInteraction), With<CanvasPaper>>,
    history: Query<(&ComputedNode, &UiGlobalTransform), With<GuessHistory>>,
    mut cursors: Query<(&mut Node, &mut Visibility), With<BrushCursor>>,
) {
    let allowed = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::draw_guess)
        .is_some_and(can_draw);
    let pointer = windows
        .single()
        .ok()
        .filter(|window| window.focused)
        .and_then(|window| {
            window
                .cursor_position()
                .map(|point| point * window.scale_factor())
        });
    let paper = papers.single().ok();
    let point = pointer
        .zip(paper)
        .and_then(|(pointer, (node, transform, interaction))| {
            let point = normalized(pointer, node, transform)?;
            let over_history = history.iter().any(|(node, transform)| {
                normalized(pointer, node, transform).is_some_and(|point| {
                    point.cmpge(Vec2::ZERO).all() && point.cmple(Vec2::ONE).all()
                })
            });
            (allowed
                && !over_history
                && *interaction != PickingInteraction::None
                && point.cmpge(Vec2::ZERO).all()
                && point.cmple(Vec2::ONE).all())
            .then_some(point)
        });
    for (mut node, mut visibility) in &mut cursors {
        if let (Some(point), Some((paper, _, _))) = (point, paper) {
            let size = if matches!(ui.tool, DrawTool::Pen | DrawTool::AreaEraser) {
                (f32::from(ui.size) * paper.size().x * paper.inverse_scale_factor()
                    / f32::from(CANVAS_WIDTH))
                .max(4.0)
            } else {
                10.0
            };
            node.left = percent(point.x * 100.0);
            node.top = percent(point.y * 100.0);
            node.width = px(size);
            node.height = px(size);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
