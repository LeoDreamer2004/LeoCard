use super::super::super::{ConnectionUiAction, InputField, UiAction};
use crate::app::presentation::{DANGER, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_client::{NetworkState, TcpGameClient};

use super::page::{HOME_PURPLE, HOME_SOFT};

pub(super) struct ConnectionStatus<'a> {
    network: Option<&'a TcpGameClient>,
    assets: &'a UiAssets,
}

impl<'a> ConnectionStatus<'a> {
    pub(super) fn new(network: Option<&'a TcpGameClient>, assets: &'a UiAssets) -> Self {
        Self { network, assets }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
        if let Some((status, color)) = self.network.and_then(|network| match network.state() {
            NetworkState::Connecting(message) | NetworkState::Reconnecting(message) => {
                Some((message.as_str(), HOME_PURPLE))
            }
            NetworkState::Failed(message) => Some((message.as_str(), DANGER)),
            NetworkState::Connected(_) => None,
        }) {
            add_text(commands, parent, status, 14.0, color, self.assets);
        }
    }
}

pub(super) struct ConnectionInput<'a> {
    label: &'a str,
    value: &'a str,
    field: InputField,
    active: bool,
    selected_all: bool,
    assets: &'a UiAssets,
}

impl<'a> ConnectionInput<'a> {
    pub(super) fn new(
        label: &'a str,
        value: &'a str,
        field: InputField,
        active: bool,
        selected_all: bool,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            label,
            value,
            field,
            active,
            selected_all,
            assets,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
        if !self.label.is_empty() {
            add_text(commands, parent, self.label, 12.0, HOME_SOFT, self.assets);
        }
        let mut image = ImageNode::new(if self.active {
            self.assets.home.focused_input.clone()
        } else {
            self.assets.home.input.clone()
        })
        .with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
        image.visual_box = VisualBox::BorderBox;
        let input = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::FocusInput(self.field)),
                Node {
                    width: percent(100),
                    min_height: px(45),
                    padding: UiRect::axes(px(13), px(9)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                image,
            ))
            .id();
        commands.entity(parent).add_child(input);
        let label = add_text(
            commands,
            input,
            format!(
                "{}{}",
                self.value,
                if self.active && !self.selected_all {
                    "│"
                } else {
                    ""
                }
            ),
            15.0,
            if self.value.is_empty() {
                HOME_SOFT
            } else {
                TEXT
            },
            self.assets,
        );
        if self.selected_all {
            commands
                .entity(label)
                .insert(TextBackgroundColor(Color::srgb(0.20, 0.42, 0.72)));
        }
    }
}
