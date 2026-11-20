use super::actions::DrawGuessUiAction;
use super::assets::DrawGuessAssets;
use super::feedback::DrawingChoicePanel;
use super::state::{BrushCursor, CanvasPaper, DrawGuessUiState, can_draw};
use super::widgets::{ButtonSpec, button};
use super::{composer, history, players, reveal, summary, toolbar};
use crate::app::games::GameTableContext;
use crate::app::presentation::{
    GameSummaryAnimation, MUTED, READY, StartGameSeatTransition, TEXT, TurnBorderMaterial,
    add_text, add_title, game_side_control_scale,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{SocialUiState, UiAction, add_reconnecting_overlay};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_draw_guess::DrawGuessPhase;
use leocard_protocol::DrawGuessSnapshot;
use leocard_ui::{TouchDragTarget, UiComposer, spawn_node};

#[derive(Component)]
pub(super) struct CanvasViewport;

pub(super) struct DrawingContext<'a> {
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub social: &'a SocialUiState,
    pub transition: &'a StartGameSeatTransition,
    pub summary: &'a GameSummaryAnimation,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub reconnecting: Option<&'a str>,
}

impl<'a> From<GameTableContext<'a>> for DrawingContext<'a> {
    fn from(context: GameTableContext<'a>) -> Self {
        Self {
            assets: context.assets,
            avatars: context.avatars,
            social: context.social,
            transition: context.transition,
            summary: context.game_summary,
            turn_border_materials: context.turn_border_materials,
            reconnecting: context.client.0.reconnecting_message(),
        }
    }
}

pub(super) fn render(
    commands: &mut UiComposer,
    root: Entity,
    game: &DrawGuessSnapshot,
    ui: &DrawGuessUiState,
    art: &DrawGuessAssets,
    mut context: DrawingContext<'_>,
) {
    let page = commands.keyed_child(
        root,
        format!("draw-guess:{:?}:{}", game.match_id, game.state.turn),
        (
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                padding: UiRect::axes(px(14), px(8)),
                row_gap: px(8),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.14, 0.17)),
        ),
    );
    let heading = spawn_node(
        commands,
        page,
        Node {
            width: percent(100),
            height: px(30),
            flex_shrink: 0.0,
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::auto(), GridTrack::fr(1.0)],
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    add_text(
        commands,
        heading,
        format!(
            "{}/{}",
            (game.state.turn / game.players.len() as u32 + 1).min(u32::from(game.rules.rounds)),
            game.rules.rounds
        ),
        18.0,
        MUTED,
        context.assets,
    );
    let word = game
        .state
        .word
        .clone()
        .unwrap_or_else(|| "＿".repeat(usize::from(game.state.word_length)));
    add_title(
        commands,
        heading,
        word,
        23.0,
        if game.state.phase == DrawGuessPhase::Reveal {
            READY
        } else {
            TEXT
        },
        context.assets,
    );
    let workspace = spawn_node(
        commands,
        page,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.0,
            flex_basis: px(0),
            column_gap: px(8),
            ..default()
        },
        None,
    );
    if can_draw(game) {
        toolbar::render(commands, workspace, ui, context.assets, art);
    }
    let viewport = spawn_node(
        commands,
        workspace,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            min_height: px(0),
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        },
        None,
    );
    commands
        .entity(viewport)
        .insert((CanvasViewport, art.skin.frame()));
    let paper = commands.keyed_child(
        viewport,
        "paper",
        (
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                width: px(1),
                height: px(1),
                overflow: Overflow::clip(),
                ..default()
            },
            UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
            ImageNode::new(art.canvas.clone()).with_mode(NodeImageMode::Stretch),
            CanvasPaper,
            TouchDragTarget,
            Pickable::default(),
            PickingInteraction::default(),
            Visibility::Hidden,
        ),
    );
    commands.keyed_child(
        paper,
        "brush-cursor",
        (
            Node {
                position_type: PositionType::Absolute,
                width: px(8),
                height: px(8),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BorderColor::all(Color::BLACK.with_alpha(0.8)),
            Outline {
                width: px(1),
                offset: px(0),
                color: Color::WHITE.with_alpha(0.85),
            },
            BrushCursor,
            UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
            Pickable::IGNORE,
            Visibility::Hidden,
            ZIndex(3),
        ),
    );
    if game.state.phase == DrawGuessPhase::Choosing && game.state.artist != Some(game.you) {
        let waiting = commands.keyed_child(
            paper,
            "waiting-for-word",
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    padding: UiRect::all(px(12.0 * game_side_control_scale())),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.5)),
                Pickable::IGNORE,
                ZIndex(2),
            ),
        );
        let label = add_text(
            commands,
            waiting,
            "等待玩家选词中",
            24.0 * game_side_control_scale(),
            TEXT,
            context.assets,
        );
        commands.entity(label).insert((
            Pickable::IGNORE,
            TextLayout::default().with_justify(Justify::Center),
        ));
    }
    let footer_height = composer::render(commands, paper, game, ui, art, context.assets);
    history::render(commands, paper, game, footer_height, context.assets);
    if game.state.phase == DrawGuessPhase::Choosing && game.state.artist == Some(game.you) {
        let choices = spawn_node(
            commands,
            viewport,
            Node {
                position_type: PositionType::Absolute,
                left: percent(15),
                right: percent(15),
                top: percent(30),
                padding: UiRect::all(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands.entity(choices).insert((
            art.skin.popup_frame(),
            DrawingChoicePanel,
            UiTransform::IDENTITY,
        ));
        for (index, word) in game.state.choices.iter().enumerate() {
            let mut spec = ButtonSpec::new(
                word,
                Some(UiAction::DrawGuess(DrawGuessUiAction::ChooseWord(
                    index as u8,
                ))),
                percent(100),
            );
            spec.height = 46.0;
            button(commands, choices, spec, context.assets, art);
        }
    }
    reveal::render(commands, viewport, game, art, context.assets);
    players::render(commands, page, game, &mut context);
    summary::render(commands, page, game, &context);
    if let Some(message) = context.reconnecting {
        add_reconnecting_overlay(commands, page, message, context.assets);
    }
}

pub(super) fn fit_canvas(
    viewports: Query<&ComputedNode, With<CanvasViewport>>,
    mut papers: Query<(&ChildOf, &mut Node, &mut Visibility), With<CanvasPaper>>,
) {
    for (parent, mut node, mut visibility) in &mut papers {
        let Ok(viewport) = viewports.get(parent.parent()) else {
            continue;
        };
        let available =
            (viewport.size() * viewport.inverse_scale_factor() - Vec2::splat(20.0)).max(Vec2::ZERO);
        let width = available.x.min(available.y * 1.6);
        if width <= 1.0 {
            continue;
        }
        let height = width / 1.6;
        if node.width != px(width) || node.height != px(height) {
            node.width = px(width);
            node.height = px(height);
        }
        *visibility = Visibility::Inherited;
    }
}
