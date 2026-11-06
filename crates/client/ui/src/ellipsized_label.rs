use crate::{AnchoredPopup, HelpTooltipStyle, UiComposer, add_label};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::text::ComputedTextBlock;
use bevy::ui::OverrideClip;

#[derive(Component)]
pub struct EllipsizedText {
    viewport: Entity,
    measurement: Entity,
    suffix: Entity,
    tooltip: Entity,
}

/// A single-line label measured with the actual font, with its full text on hover only when truncated.
pub struct EllipsizedLabel<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub color: Color,
    pub tooltip: HelpTooltipStyle<'a>,
}

impl EllipsizedLabel<'_> {
    pub fn mount(self, commands: &mut UiComposer, parent: Entity) -> Entity {
        let viewport = commands.child(
            parent,
            (
                Node {
                    width: percent(100),
                    min_width: px(0),
                    height: px(self.font_size * 1.5),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                Pickable::default(),
                PickingInteraction::default(),
            ),
        );
        let measurement = self.measure(commands, viewport, self.text);
        let suffix = self.measure(commands, viewport, "...");
        let label = add_label(
            commands,
            viewport,
            self.text,
            self.font_size,
            self.color,
            self.tooltip.font,
        );
        commands
            .entity(label)
            .insert((TextLayout::no_wrap(), Pickable::IGNORE));
        let tooltip = commands.child(
            viewport,
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(self.tooltip.width),
                    padding: UiRect::all(px(15)),
                    ..default()
                },
                self.tooltip.background.clone(),
                Visibility::Hidden,
                GlobalZIndex(1000),
                OverrideClip,
                AnchoredPopup(viewport),
                Pickable::IGNORE,
            ),
        );
        let tip = add_label(
            commands,
            tooltip,
            self.text,
            12.0,
            self.tooltip.text,
            self.tooltip.font,
        );
        commands.entity(tip).insert(Pickable::IGNORE);
        commands.entity(label).insert(EllipsizedText {
            viewport,
            measurement,
            suffix,
            tooltip,
        });
        viewport
    }

    fn measure(&self, commands: &mut UiComposer, parent: Entity, text: &str) -> Entity {
        let measurement = add_label(
            commands,
            parent,
            text,
            self.font_size,
            self.color,
            self.tooltip.font,
        );
        commands.entity(measurement).insert((
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            TextLayout::no_wrap(),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
        measurement
    }
}

/// Hidden full-text and suffix layouts keep truncation stable across resizing and scaling.
pub fn update_ellipsized_labels(
    mut labels: Query<(&EllipsizedText, &mut Text)>,
    measurements: Query<(&Text, &ComputedTextBlock), Without<EllipsizedText>>,
    viewports: Query<(&ComputedNode, &PickingInteraction)>,
    mut tooltips: Query<&mut Visibility>,
) {
    for (label, mut text) in &mut labels {
        let (Ok((viewport, interaction)), Ok((full, layout)), Ok((_, suffix))) = (
            viewports.get(label.viewport),
            measurements.get(label.measurement),
            measurements.get(label.suffix),
        ) else {
            continue;
        };
        let width = viewport.size().x;
        let buffer = layout.buffer();
        let measured = width > 0.0 && buffer.width() > 0.0 && suffix.buffer().width() > 0.0;
        let truncated = measured && buffer.width() > width;
        if let Ok(mut visibility) = tooltips.get_mut(label.tooltip) {
            visibility.set_if_neq(if truncated && *interaction != PickingInteraction::None {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
        }
        if !measured {
            continue;
        }
        let value = if !truncated {
            full.0.clone()
        } else {
            let available = (width - suffix.buffer().width()).max(0.0);
            let mut advance = 0.0;
            let mut end = 0;
            'lines: for line in buffer.lines() {
                for run in line.runs() {
                    for cluster in run.clusters() {
                        advance += cluster.advance();
                        if advance > available {
                            break 'lines;
                        }
                        end = cluster.text_range().end;
                    }
                }
            }
            format!("{}...", full.0[..end].trim_end())
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
