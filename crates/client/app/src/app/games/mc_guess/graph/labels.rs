use super::super::McGuessUiState;
use super::super::view::GuessTable;
use super::lod::GraphLod;
use crate::app::presentation::add_text;
use crate::app::runtime::ClientResource;
use bevy::ecs::system::SystemParam;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_mc_guess::McGuessGraphView;
use leocard_protocol::GameSnapshot;
use leocard_ui::{UiComposer, spawn_node};

#[derive(Component, Clone, Copy)]
pub(super) enum GraphLabel {
    Number(u16),
    Name(u16),
    Edge(usize),
}

#[derive(Component)]
struct GraphLabelText {
    source: GraphLabel,
    color: Color,
}

#[derive(SystemParam)]
pub(in super::super) struct GraphLabels<'w, 's> {
    text: Query<'w, 's, (&'static GraphLabelText, &'static mut TextColor)>,
    roots: Query<
        'w,
        's,
        (
            &'static GraphLabel,
            &'static mut Visibility,
            Option<&'static mut UiTransform>,
        ),
    >,
}

impl GraphLabel {
    pub fn highlighted(self, graph: &McGuessGraphView, selected: u16) -> bool {
        match self {
            Self::Number(number) | Self::Name(number) => {
                number == selected || graph.nodes[usize::from(number - 1)].revealed()
            }
            Self::Edge(index) => {
                let edge = &graph.edges[index];
                edge.active || edge.source == selected || edge.target == selected
            }
        }
    }

    fn opacity(self, graph: &McGuessGraphView, selected: u16, lod: GraphLod) -> f32 {
        let emphasis = if self.highlighted(graph, selected) {
            1.0
        } else {
            0.7
        };
        emphasis
            * match self {
                Self::Number(_) => 1.0,
                Self::Name(_) => lod.name_opacity,
                Self::Edge(_) => lod.edge_opacity,
            }
    }
}

pub(in super::super) fn animate_graph_labels(
    state: Res<McGuessUiState>,
    client: Option<Res<ClientResource>>,
    mut labels: GraphLabels,
) {
    let Some(graph) = client
        .as_deref()
        .and_then(|c| c.0.model().game_snapshot().and_then(GameSnapshot::mc_guess))
        .and_then(|game| game.graph.as_ref())
        .filter(|graph| !graph.nodes.is_empty())
    else {
        return;
    };
    let selected = state.graph.selected.unwrap_or(graph.selected);
    let lod = GraphLod::at_zoom(state.graph.camera.zoom);
    for (label, mut color) in &mut labels.text {
        let value = label
            .color
            .with_alpha(label.color.alpha() * label.source.opacity(graph, selected, lod));
        if color.0 != value {
            color.0 = value;
        }
    }
    for (source, mut visibility, transform) in &mut labels.roots {
        visibility.set_if_neq(label_visibility(source.opacity(graph, selected, lod)));
        if let GraphLabel::Name(_) = source
            && let Some(mut transform) = transform
        {
            // The parent shrinks and brings the name closer, while glyphs stay readable.
            transform.set_if_neq(lod.name_transform());
        }
    }
}

impl GuessTable<'_> {
    pub(super) fn graph_label(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        source: GraphLabel,
        text: &str,
        size: f32,
        color: Color,
    ) -> Entity {
        let graph = self.game.graph.as_ref().expect("graph mode");
        let opacity = source.opacity(
            graph,
            self.state.viewed_node(self.game).expect("graph node"),
            GraphLod::at_zoom(self.state.graph.camera.zoom),
        );
        let root = spawn_node(
            commands,
            parent,
            Node {
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        commands
            .entity(root)
            .insert((source, Pickable::IGNORE, label_visibility(opacity)));
        // Symmetric glyph passes form a thin outline; no shadow or label backdrop.
        let outline = Color::srgba_u8(36, 38, 49, 200);
        for offset in [
            Vec2::new(-0.75, 0.0),
            Vec2::new(0.75, 0.0),
            Vec2::new(0.0, -0.75),
            Vec2::new(0.0, 0.75),
            Vec2::new(-0.53, -0.53),
            Vec2::new(0.53, -0.53),
            Vec2::new(-0.53, 0.53),
            Vec2::new(0.53, 0.53),
        ] {
            let stroke = add_text(
                commands,
                root,
                text,
                size,
                outline.with_alpha(outline.alpha() * opacity),
                self.assets,
            );
            commands.entity(stroke).insert((
                GraphLabelText {
                    source,
                    color: outline,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(offset.x),
                    top: px(offset.y),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                TextLayout::no_wrap(),
                Pickable::IGNORE,
            ));
        }
        let label = add_text(
            commands,
            root,
            text,
            size,
            color.with_alpha(color.alpha() * opacity),
            self.assets,
        );
        commands.entity(label).insert((
            GraphLabelText { source, color },
            TextLayout::no_wrap(),
            Pickable::IGNORE,
        ));
        root
    }
}

fn label_visibility(opacity: f32) -> Visibility {
    if opacity > 0.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}
