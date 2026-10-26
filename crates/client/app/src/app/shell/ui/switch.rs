use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{CustomButtonMotion, UiComposer, sliced_image};

#[derive(Component)]
pub(super) struct CozySwitch {
    selected: bool,
    enabled: bool,
}

#[derive(Component, Clone)]
pub(super) struct CozySwitchThumb {
    owner: Entity,
    progress: f32,
}

impl Default for CozySwitchThumb {
    fn default() -> Self {
        Self {
            owner: Entity::PLACEHOLDER,
            progress: 0.0,
        }
    }
}

impl CozySwitchThumb {
    pub(super) fn same_owner(&self, other: &Self) -> bool {
        self.owner == other.owner
    }
}

pub(crate) fn add_cozy_switch(
    commands: &mut UiComposer,
    parent: Entity,
    selected: bool,
    action: Option<UiAction>,
    assets: &UiAssets,
) -> Entity {
    let enabled = action.is_some();
    let switch = commands.child(
        parent,
        (
            CozySwitch { selected, enabled },
            CustomButtonMotion,
            Node {
                width: px(56),
                height: px(30),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                ..default()
            },
            sliced_image(assets.home.input.clone(), 32.0, 0.35),
        ),
    );
    if let Some(action) = action {
        commands.entity(switch).insert((Button, action));
    } else {
        commands.entity(switch).insert(Pickable::IGNORE);
    }
    commands.child(
        switch,
        (
            CozySwitchThumb {
                owner: switch,
                progress: f32::from(selected),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(3),
                top: px(3),
                width: px(24),
                height: px(24),
                ..default()
            },
            sliced_image(assets.home.button.clone(), 32.0, 0.32),
            Pickable::IGNORE,
        ),
    );
    switch
}

pub(super) fn animate_cozy_switches(
    time: Res<Time>,
    mut switches: Query<
        (&CozySwitch, Option<&PickingInteraction>, &mut ImageNode),
        Without<CozySwitchThumb>,
    >,
    mut thumbs: Query<(&mut CozySwitchThumb, &mut Node, &mut ImageNode)>,
) {
    let response = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (mut thumb, mut node, mut image) in &mut thumbs {
        let Ok((switch, interaction, mut track)) = switches.get_mut(thumb.owner) else {
            continue;
        };
        thumb.progress += (f32::from(switch.selected) - thumb.progress) * response;
        node.left = px(3.0 + 26.0 * thumb.progress);
        let hovered = interaction.is_some_and(|value| *value != PickingInteraction::None);
        let opacity = if switch.enabled { 1.0 } else { 0.45 };
        track.color = Color::srgba(
            1.0 - thumb.progress * 0.52,
            1.0,
            1.0 - thumb.progress * 0.36,
            opacity,
        );
        let brightness = if hovered { 1.0 } else { 0.88 };
        image.color = Color::srgba(brightness, brightness, brightness, opacity);
    }
}
