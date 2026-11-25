use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::shell::ShellOverlayLayer;
use crate::app::shell::{UiAction, add_cozy_panel};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{OverrideClip, VisualBox};
use bevy::ui_widgets::Button;
use leocard_ui::{ButtonHighlight, CustomButtonMotion, UiComposer, spawn_node};

pub(crate) struct CozyDropdownOption {
    pub label: &'static str,
    pub selected: bool,
    pub action: Option<UiAction>,
}

pub(crate) struct CozyDropdown {
    pub label: &'static str,
    pub toggle_action: UiAction,
    pub close_action: UiAction,
    pub options: Vec<CozyDropdownOption>,
}

struct DropdownPlacement {
    root: Entity,
    parent: Entity,
    layer: i32,
    closed_layer: i32,
}

impl CozyDropdown {
    pub(crate) fn render(
        self,
        commands: &mut UiComposer,
        root: Entity,
        parent: Entity,
        can_configure: bool,
        open: bool,
        assets: &UiAssets,
    ) {
        self.mount(
            commands,
            DropdownPlacement {
                root,
                parent,
                layer: 900,
                closed_layer: 0,
            },
            can_configure,
            open,
            assets,
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn render_in_dialog(
        self,
        commands: &mut UiComposer,
        root: Entity,
        parent: Entity,
        open: bool,
        assets: &UiAssets,
    ) {
        self.mount(
            commands,
            DropdownPlacement {
                root,
                parent,
                layer: ShellOverlayLayer::Dialog.z_index().0 + 1,
                closed_layer: ShellOverlayLayer::Dialog.z_index().0 + 1,
            },
            true,
            open,
            assets,
        );
    }

    fn mount(
        self,
        commands: &mut UiComposer,
        placement: DropdownPlacement,
        can_configure: bool,
        open: bool,
        assets: &UiAssets,
    ) {
        let DropdownPlacement {
            root,
            parent,
            layer,
            closed_layer,
        } = placement;
        let dropdown_accent = Color::srgb(0.85, 0.82, 1.0);

        if open && can_configure {
            commands.child(
                root,
                (
                    Button,
                    CustomButtonMotion,
                    self.close_action.clone(),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    GlobalZIndex(layer),
                    Pickable::default(),
                ),
            );
        }

        let selector = spawn_node(
            commands,
            parent,
            Node {
                position_type: PositionType::Relative,
                width: px(190),
                height: px(40),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        // Expanded menus sit above page controls and below tooltips and account popups.
        commands
            .entity(selector)
            .insert(GlobalZIndex(if open && can_configure {
                layer + 10
            } else {
                closed_layer
            }));

        let mut trigger_image = ImageNode::new(assets.home.input.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.55,
            }),
        );
        trigger_image.visual_box = VisualBox::BorderBox;
        if !can_configure {
            trigger_image.color = Color::WHITE.with_alpha(0.72);
        }
        let child = commands.child(
            selector,
            (
                Node {
                    width: percent(100),
                    height: px(40),
                    padding: UiRect::horizontal(px(13)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                trigger_image,
            ),
        );
        let mut trigger = commands.entity(child);
        if can_configure {
            trigger.insert((Button, CustomButtonMotion, self.toggle_action.clone()));
        } else {
            trigger.insert(Pickable::default());
        }
        let trigger = trigger.id();

        if can_configure {
            let mut hover_image = ImageNode::new(assets.home.focused_input.clone()).with_mode(
                NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(32.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 0.55,
                }),
            );
            hover_image.visual_box = VisualBox::BorderBox;
            let hover = commands.child(
                trigger,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    hover_image,
                    Visibility::Hidden,
                    Pickable::IGNORE,
                ),
            );
            commands.entity(trigger).insert(ButtonHighlight {
                overlay: hover,
                arrows: None,
            });
        }
        let label = add_text(commands, trigger, self.label, 14.0, TEXT, assets);
        commands.entity(label).insert(Pickable::IGNORE);
        let arrow = add_text(
            commands,
            trigger,
            if can_configure {
                if open { "▲" } else { "▼" }
            } else {
                "—"
            },
            12.0,
            if can_configure {
                dropdown_accent
            } else {
                MUTED
            },
            assets,
        );
        commands.entity(arrow).insert(Pickable::IGNORE);

        if !open || !can_configure {
            return;
        }
        let menu = add_cozy_panel(
            commands,
            selector,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(44),
                padding: UiRect::all(px(7)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                ..default()
            },
            assets,
        );
        commands
            .entity(menu)
            .insert((GlobalZIndex(layer + 20), Pickable::default(), OverrideClip));
        for option in self.options {
            let selected = option.selected;
            let enabled = option.action.is_some();
            let option_label = option.label;
            let mut option_image = ImageNode::new(if selected {
                assets.home.purple_button_compact.clone()
            } else {
                assets.home.button.clone()
            })
            .with_mode(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.55,
            }));
            option_image.visual_box = VisualBox::BorderBox;
            if !enabled {
                option_image.color = Color::WHITE.with_alpha(0.45);
            }
            let action = option.action;
            let option = commands.child(
                menu,
                (
                    Node {
                        width: percent(90),
                        min_width: px(0),
                        height: px(38),
                        padding: UiRect::horizontal(px(11)),
                        align_self: AlignSelf::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    option_image,
                ),
            );
            if let Some(action) = action {
                commands
                    .entity(option)
                    .insert((Button, CustomButtonMotion, action));
            }
            if enabled && !selected {
                let mut hover_image = ImageNode::new(assets.home.purple_button_compact.clone())
                    .with_mode(NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::all(32.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 0.55,
                    }));
                hover_image.visual_box = VisualBox::BorderBox;
                let hover = commands.child(
                    option,
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            right: px(0),
                            top: px(0),
                            bottom: px(0),
                            ..default()
                        },
                        hover_image,
                        Visibility::Hidden,
                        Pickable::IGNORE,
                    ),
                );
                commands.entity(option).insert(ButtonHighlight {
                    overlay: hover,
                    arrows: None,
                });
            }
            let label = add_text(
                commands,
                option,
                option_label,
                13.5,
                if enabled { TEXT } else { MUTED },
                assets,
            );
            commands.entity(label).insert(Pickable::IGNORE);
        }
    }
}
