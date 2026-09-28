//! 建房游戏选择与主连接页面。

use super::super::{ConnectionUiAction, InputField, UiAction};
use crate::app::games::SUPPORTED_GAMES;
use crate::app::presentation::{DANGER, TEXT, add_avatar, add_text, spawn_node};
use crate::app::runtime::{AppearancePreferences, AvatarImages, ConnectionDraft, UiAssets};
use bevy::prelude::*;
use bevy::ui::{
    BackgroundGradient, ColorStop, FocusPolicy, Gradient, LinearGradient, RadialGradient,
    RadialGradientShape, UiPosition, VisualBox,
};
use leocard_client::{NetworkState, TcpGameClient};
use leocard_protocol::PlayerGender;

const HOME_PURPLE: Color = Color::srgb(0.78, 0.74, 1.0);
const HOME_SOFT: Color = Color::srgb(0.73, 0.79, 0.75);
#[derive(Component)]
pub(crate) enum HomeHighlightKind {
    Card {
        arrows: [Entity; 2],
    },
    Button {
        overlay: Entity,
        arrows: Option<[Entity; 2]>,
    },
}

#[derive(Component)]
pub(super) struct HomeAnimatedArrows {
    left: bool,
    owner: Entity,
    elapsed: f32,
}

pub(super) fn animate_home_arrows(
    time: Res<Time>,
    interactions: Query<&Interaction>,
    mut images: Query<(&mut HomeAnimatedArrows, &mut ImageNode)>,
) {
    for (mut arrow, mut image) in &mut images {
        if !interactions
            .get(arrow.owner)
            .is_ok_and(|state| *state != Interaction::None)
        {
            arrow.elapsed = 0.0;
            continue;
        }
        arrow.elapsed += time.delta_secs();
        let tick = ((arrow.elapsed * 20.0) as usize) % 33;
        let frame = if tick < 16 {
            tick
        } else if tick < 19 {
            15
        } else {
            33 - tick
        } as f32;
        let left = if arrow.left { 8.0 } else { 136.0 };
        let rect = Rect::from_corners(
            Vec2::new(left, frame * 80.0 + 8.0),
            Vec2::new(left + 112.0, frame * 80.0 + 64.0),
        );
        if image.rect != Some(rect) {
            image.rect = Some(rect);
        }
    }
}

pub(super) fn update_home_highlights(
    mut commands: Commands,
    buttons: Query<(Entity, &Interaction, &HomeHighlightKind), Changed<Interaction>>,
    mut card_images: Query<&mut ImageNode>,
    assets: Res<UiAssets>,
) {
    for (entity, interaction, kind) in &buttons {
        let hovered = *interaction != Interaction::None;
        let visibility = if hovered {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        match kind {
            HomeHighlightKind::Card { arrows } => {
                if let Ok(mut image) = card_images.get_mut(entity) {
                    image.image = if hovered {
                        assets.home.game_card_hover.clone()
                    } else {
                        assets.home.game_card.clone()
                    };
                }
                for arrow in arrows {
                    commands.entity(*arrow).insert(visibility);
                }
            }
            HomeHighlightKind::Button { overlay, arrows } => {
                commands.entity(*overlay).insert(visibility);
                if let Some(arrows) = arrows {
                    for arrow in arrows {
                        commands.entity(*arrow).insert(visibility);
                    }
                }
            }
        }
    }
}

pub(crate) struct ConnectionScreen<'a> {
    connection: &'a ConnectionDraft,
    appearance: &'a AppearancePreferences,
    network: Option<&'a TcpGameClient>,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> ConnectionScreen<'a> {
    pub(crate) fn new(
        connection: &'a ConnectionDraft,
        appearance: &'a AppearancePreferences,
        network: Option<&'a TcpGameClient>,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            connection,
            appearance,
            network,
            assets,
            avatars,
        }
    }

    pub(crate) fn render(self, commands: &mut Commands, root: Entity) {
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(Color::srgb(0.045, 0.05, 0.075)),
        );
        commands.entity(canvas).insert(BackgroundGradient(vec![
            Gradient::Linear(LinearGradient::to_bottom_right(vec![
                ColorStop::percent(Color::srgb(0.11, 0.10, 0.16), 0.0),
                ColorStop::percent(Color::srgb(0.075, 0.078, 0.12), 48.0),
                ColorStop::percent(Color::srgb(0.038, 0.055, 0.078), 100.0),
            ])),
            Gradient::Radial(RadialGradient::new(
                UiPosition::TOP_RIGHT,
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::percent(Color::srgba(0.30, 0.24, 0.45, 0.24), 0.0),
                    ColorStop::percent(Color::srgba(0.30, 0.24, 0.45, 0.0), 66.0),
                ],
            )),
            Gradient::Radial(RadialGradient::new(
                UiPosition::BOTTOM_LEFT,
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::percent(Color::srgba(0.09, 0.18, 0.22, 0.13), 0.0),
                    ColorStop::percent(Color::srgba(0.09, 0.18, 0.22, 0.0), 55.0),
                ],
            )),
        ]));
        let felt = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                ImageNode::new(self.assets.table_felt.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(Color::srgba(0.45, 0.45, 0.70, 0.08)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(canvas).add_child(felt);

        let content = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1370),
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(16),
                ..default()
            },
            None,
        );
        self.render_games(commands, content);
        self.render_sidebar(commands, content);
    }

    fn render_games(&self, commands: &mut Commands, parent: Entity) {
        let panel = home_panel(
            commands,
            parent,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.1,
                padding: UiRect::all(px(26)),
                flex_direction: FlexDirection::Column,
                row_gap: px(0),
                ..default()
            },
            self.assets,
        );
        let heading = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        add_text(commands, heading, "马上开局", 30.0, TEXT, self.assets);
        let port = spawn_node(
            commands,
            heading,
            Node {
                width: px(190),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        add_text(commands, port, "建房端口", 13.0, HOME_SOFT, self.assets);
        let port_input = spawn_node(
            commands,
            port,
            Node {
                width: px(110),
                ..default()
            },
            None,
        );
        ConnectionInput::new(
            "",
            &self.connection.host_port,
            InputField::HostPort,
            self.connection.active == InputField::HostPort,
            self.connection.selected_all && self.connection.active == InputField::HostPort,
            self.assets,
        )
        .render(commands, port_input);
        spawn_node(
            commands,
            panel,
            Node {
                width: px(94),
                height: px(2),
                ..default()
            },
            Some(HOME_PURPLE.with_alpha(0.75)),
        );

        let shelf = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                margin: UiRect::top(px(16)),
                ..default()
            },
            None,
        );
        for indices in [&[4, 2, 1][..], &[3, 0][..]] {
            let row = spawn_node(
                commands,
                shelf,
                Node {
                    width: percent(100),
                    min_height: px(192),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(10),
                    ..default()
                },
                None,
            );
            for &index in indices {
                self.render_game_card(commands, row, index);
            }
        }
    }

    fn render_game_card(&self, commands: &mut Commands, parent: Entity, index: usize) {
        let game = &SUPPORTED_GAMES[index];
        let card = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::CreateRoom(game.kind)),
                Node {
                    min_width: px(0),
                    min_height: px(0),
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    padding: UiRect::axes(px(17), px(14)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    row_gap: px(4),
                    ..default()
                },
                home_game_card_image(self.assets.home.game_card.clone()),
            ))
            .id();
        commands.entity(parent).add_child(card);
        let title_row = spawn_node(
            commands,
            card,
            Node {
                width: percent(100),
                min_height: px(30),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(4),
                ..default()
            },
            None,
        );
        let left_arrow = add_home_animated_arrow(
            commands,
            title_row,
            card,
            self.assets,
            true,
            Vec2::new(56.0, 35.0),
        );
        add_text(commands, title_row, game.title, 21.0, TEXT, self.assets);
        let right_arrow = add_home_animated_arrow(
            commands,
            title_row,
            card,
            self.assets,
            false,
            Vec2::new(56.0, 35.0),
        );
        let art_holder = spawn_node(
            commands,
            card,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let art_frame = spawn_node(
            commands,
            art_holder,
            Node {
                width: px(72),
                height: px(94),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            (index == 4).then_some(Color::srgb(0.91, 0.90, 0.82)),
        );
        let art = commands
            .spawn((
                Node {
                    width: px(if index == 4 { 58 } else { 68 }),
                    height: px(if index == 4 { 78 } else { 91 }),
                    ..default()
                },
                ImageNode::new(self.assets.home.game_art[index].clone()),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(art_frame).add_child(art);
        add_text(
            commands,
            card,
            game.description,
            15.0,
            HOME_SOFT,
            self.assets,
        );
        commands.entity(card).insert(HomeHighlightKind::Card {
            arrows: [left_arrow, right_arrow],
        });
    }

    fn render_sidebar(&self, commands: &mut Commands, parent: Entity) {
        let rail = spawn_node(
            commands,
            parent,
            Node {
                width: percent(32),
                min_width: px(300),
                max_width: px(420),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
                ..default()
            },
            None,
        );
        self.render_identity(commands, rail);
        self.render_join(commands, rail);
    }

    fn render_identity(&self, commands: &mut Commands, parent: Entity) {
        let panel = home_panel(
            commands,
            parent,
            Node {
                width: percent(100),
                padding: UiRect::all(px(21)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            self.assets,
        );
        add_text(commands, panel, "玩家身份", 19.0, TEXT, self.assets);
        let row = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::FlexEnd,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        let avatar_button = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::ChooseAvatar),
                Node {
                    width: px(48),
                    height: px(48),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(percent(50)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.19, 0.28, 0.24)),
            ))
            .id();
        commands.entity(row).add_child(avatar_button);
        add_avatar(
            commands,
            avatar_button,
            &self.connection.player_name,
            self.avatars.local.as_ref(),
            48.0,
            self.assets,
        );
        let name = spawn_node(
            commands,
            row,
            Node {
                min_width: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        ConnectionInput::new(
            "",
            &self.connection.player_name,
            InputField::PlayerName,
            self.connection.active == InputField::PlayerName,
            self.connection.selected_all && self.connection.active == InputField::PlayerName,
            self.assets,
        )
        .render(commands, name);
        let (symbol, color) = match self.connection.gender {
            PlayerGender::Male => ("♂", Color::srgb(0.42, 0.70, 1.0)),
            PlayerGender::Female => ("♀", Color::srgb(1.0, 0.58, 0.76)),
        };
        let gender = commands
            .spawn((
                Button,
                UiAction::Connection(ConnectionUiAction::ToggleGender),
                Node {
                    width: px(48),
                    height: px(48),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .id();
        commands.entity(row).add_child(gender);
        let symbol = add_text(commands, gender, symbol, 31.0, color, self.assets);
        commands.entity(symbol).insert(FocusPolicy::Pass);
        if self.appearance.avatar_png.is_some() {
            home_button(
                commands,
                panel,
                "清除头像",
                ConnectionUiAction::ClearAvatar,
                self.assets,
                false,
                px(92),
            );
        }
    }

    fn render_join(&self, commands: &mut Commands, parent: Entity) {
        let panel = home_panel(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(0),
                padding: UiRect::all(px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            self.assets,
        );
        add_text(commands, panel, "加入房间", 24.0, TEXT, self.assets);
        add_text(
            commands,
            panel,
            "已有朋友开桌？输入房主地址直接入座。",
            13.0,
            HOME_SOFT,
            self.assets,
        );
        ConnectionInput::new(
            "房间地址",
            &self.connection.join_address,
            InputField::JoinAddress,
            self.connection.active == InputField::JoinAddress,
            self.connection.selected_all && self.connection.active == InputField::JoinAddress,
            self.assets,
        )
        .render(commands, panel);
        home_button(
            commands,
            panel,
            "加入房间",
            ConnectionUiAction::JoinRoom,
            self.assets,
            true,
            percent(100),
        );
        add_text(
            commands,
            panel,
            "示例  192.168.1.20:52300",
            12.0,
            HOME_SOFT,
            self.assets,
        );
        ConnectionStatus::new(self.network, self.assets).render(commands, panel);
    }
}

fn home_panel_image(assets: &UiAssets) -> ImageNode {
    let mut image =
        ImageNode::new(assets.home.panel.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(80.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.42,
        }));
    image.visual_box = VisualBox::BorderBox;
    image
}

fn home_game_card_image(texture: Handle<Image>) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }));
    image.visual_box = VisualBox::BorderBox;
    image
}

fn home_panel(commands: &mut Commands, parent: Entity, node: Node, assets: &UiAssets) -> Entity {
    let entity = commands.spawn(node).id();
    commands.entity(parent).add_child(entity);
    let background = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            home_panel_image(assets),
            ZIndex(-1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(entity).add_child(background);
    entity
}

fn home_button_image(assets: &UiAssets) -> ImageNode {
    let mut image = ImageNode::new(assets.home.button.clone()).with_mode(NodeImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        },
    ));
    image.visual_box = VisualBox::BorderBox;
    image
}

fn add_home_purple_overlay(
    commands: &mut Commands,
    parent: Entity,
    assets: &UiAssets,
    visible: bool,
    compact: bool,
) -> Entity {
    let texture = if compact {
        &assets.home.purple_button_compact
    } else {
        &assets.home.purple_button
    };
    let mut image =
        ImageNode::new(texture.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect {
                min_inset: Vec2::new(32.0, 32.0),
                max_inset: Vec2::new(32.0, 32.0),
            },
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        }));
    image.visual_box = VisualBox::BorderBox;
    let overlay = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            image,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(overlay);
    overlay
}

fn add_home_animated_arrow(
    commands: &mut Commands,
    parent: Entity,
    owner: Entity,
    assets: &UiAssets,
    left: bool,
    size: Vec2,
) -> Entity {
    let mut image = ImageNode::new(assets.home.button_arrows.clone());
    let x = if left { 8.0 } else { 136.0 };
    image.rect = Some(Rect::from_corners(
        Vec2::new(x, 8.0),
        Vec2::new(x + 112.0, 64.0),
    ));
    let entity = commands
        .spawn((
            Node {
                width: px(size.x),
                height: px(size.y),
                ..default()
            },
            image,
            HomeAnimatedArrows {
                left,
                owner,
                elapsed: 0.0,
            },
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

fn home_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: ConnectionUiAction,
    assets: &UiAssets,
    primary: bool,
    width: Val,
) {
    let entity = commands
        .spawn((
            Button,
            UiAction::Connection(action),
            Node {
                width,
                height: px(43),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(5),
                ..default()
            },
            home_button_image(assets),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    let overlay = add_home_purple_overlay(commands, entity, assets, false, true);
    let left_arrow = primary.then(|| {
        add_home_animated_arrow(
            commands,
            entity,
            entity,
            assets,
            true,
            Vec2::new(64.0, 40.0),
        )
    });
    add_text(commands, entity, label, 14.0, TEXT, assets);
    let right_arrow = primary.then(|| {
        add_home_animated_arrow(
            commands,
            entity,
            entity,
            assets,
            false,
            Vec2::new(64.0, 40.0),
        )
    });
    commands.entity(entity).insert(HomeHighlightKind::Button {
        overlay,
        arrows: left_arrow
            .zip(right_arrow)
            .map(|(left, right)| [left, right]),
    });
}

struct ConnectionStatus<'a> {
    network: Option<&'a TcpGameClient>,
    assets: &'a UiAssets,
}

impl<'a> ConnectionStatus<'a> {
    fn new(network: Option<&'a TcpGameClient>, assets: &'a UiAssets) -> Self {
        Self { network, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
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

struct ConnectionInput<'a> {
    label: &'a str,
    value: &'a str,
    field: InputField,
    active: bool,
    selected_all: bool,
    assets: &'a UiAssets,
}

impl<'a> ConnectionInput<'a> {
    fn new(
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

    fn render(self, commands: &mut Commands, parent: Entity) {
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
