use super::*;
use crate::app::presentation::{
    ACCENT, LobbyEmptySeatLabel, LobbyEmptySeatRing, LobbySeatHover, LobbySeatTransitionSource,
    LobbySeatVisual, MUTED, READY, SecondaryPressTarget, TEXT, add_avatar, add_host_crown,
    add_text, spawn_node,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction, reference_level};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use bevy::ui_widgets::Button;
use leocard_protocol::{GameKind, LobbyPlayer, LobbySnapshot, SeatId};

pub(super) struct LobbySeatSelector<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> LobbySeatSelector<'a> {
    pub(super) fn new(
        client: &'a ClientResource,
        lobby: &'a LobbySnapshot,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            client,
            lobby,
            assets,
            avatars,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, parent: Entity) {
        let Self {
            client,
            lobby,
            assets,
            avatars,
        } = self;
        let ring = spawn_node(
            commands,
            parent,
            Node {
                width: px(620),
                height: px(390),
                max_width: percent(100),
                flex_shrink: 0.0,
                align_self: AlignSelf::Center,
                margin: UiRect::vertical(Val::Auto),
                position_type: PositionType::Relative,
                ..default()
            },
            None,
        );
        LobbyTable::new(client, lobby, assets).render(commands, ring);

        for seat_index in 0..LobbyMetrics::new(lobby).seat_count() {
            LobbySeat::new(client, lobby, assets, avatars, seat_index).render(commands, ring);
        }
    }
}

struct LobbySeat<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
    index: u8,
}

impl<'a> LobbySeat<'a> {
    fn new(
        client: &'a ClientResource,
        lobby: &'a LobbySnapshot,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
        index: u8,
    ) -> Self {
        Self {
            client,
            lobby,
            assets,
            avatars,
            index,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let seat = SeatId(self.index);
        let occupant = self
            .lobby
            .players
            .iter()
            .find(|player| player.seat == Some(seat));
        let entity = self.spawn_container(commands, parent, seat);
        let visual = self.spawn_visual(commands, entity);

        if let Some(player) = occupant {
            OccupiedLobbySeat::new(
                player,
                self.client.0.model().you() == Some(player.id),
                self.lobby.host == Some(player.id),
                self.assets,
                self.avatars,
            )
            .render(commands, entity, visual);
        } else {
            EmptyLobbySeat::new(self.index, self.assets).render(commands, visual);
        }
    }

    fn spawn_container(&self, commands: &mut Commands, parent: Entity, seat: SeatId) -> Entity {
        let (left, top) = self.position();
        let entity = commands
            .spawn((
                Button,
                SecondaryPressTarget,
                UiAction::Lobby(LobbyUiAction::SelectSeat(seat)),
                LobbySeatHover {
                    seat: self.index,
                    amount: 0.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(142),
                    height: px(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                UiTransform::IDENTITY,
            ))
            .id();
        commands.entity(parent).add_child(entity);
        entity
    }

    fn spawn_visual(&self, commands: &mut Commands, parent: Entity) -> Entity {
        let visual = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(3),
                ..default()
            },
            None,
        );
        commands.entity(visual).insert((
            LobbySeatVisual(self.index),
            UiTransform::IDENTITY,
            Pickable::IGNORE,
        ));
        visual
    }

    fn position(&self) -> (f32, f32) {
        if matches!(self.lobby.game, GameKind::Shengji | GameKind::Mahjong) {
            match self.index {
                0 => (239.0, 290.0),
                1 => (0.0, 145.0),
                2 => (239.0, 0.0),
                3 => (478.0, 145.0),
                _ => unreachable!("双升和麻将固定四个座位"),
            }
        } else {
            lobby_seat_position(self.index)
        }
    }
}

struct OccupiedLobbySeat<'a> {
    player: &'a LobbyPlayer,
    is_you: bool,
    is_host: bool,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> OccupiedLobbySeat<'a> {
    fn new(
        player: &'a LobbyPlayer,
        is_you: bool,
        is_host: bool,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            player,
            is_you,
            is_host,
            assets,
            avatars,
        }
    }

    fn render(self, commands: &mut Commands, container: Entity, visual: Entity) {
        commands
            .entity(container)
            .insert(LobbySeatTransitionSource(self.player.id));
        let avatar_ring = self.spawn_avatar_ring(commands, visual);
        let handle = self
            .player
            .avatar
            .and_then(|id| self.avatars.remote.get(&id));
        let avatar = add_avatar(
            commands,
            avatar_ring,
            &self.player.name,
            handle,
            52.0,
            self.assets,
        );
        if self.is_host {
            add_host_crown(commands, avatar, self.assets);
        }
        if self.player.ready {
            self.add_ready_badge(commands, avatar_ring);
        }
        add_text(
            commands,
            visual,
            &self.player.name,
            14.0,
            if self.is_you { ACCENT } else { TEXT },
            self.assets,
        );
        self.add_status(commands, visual);
    }

    fn spawn_avatar_ring(&self, commands: &mut Commands, parent: Entity) -> Entity {
        let ring = spawn_node(
            commands,
            parent,
            Node {
                width: px(58),
                height: px(58),
                min_width: px(58),
                position_type: PositionType::Relative,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::srgba(0.03, 0.07, 0.06, 0.88)),
        );
        commands
            .entity(ring)
            .insert(BorderColor::all(if self.is_you {
                ACCENT
            } else if self.player.ready {
                READY
            } else {
                MUTED.with_alpha(0.7)
            }));
        ring
    }

    fn add_ready_badge(&self, commands: &mut Commands, parent: Entity) {
        let check = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(-2),
                    bottom: px(-1),
                    width: px(20),
                    height: px(20),
                    ..default()
                },
                ImageNode::new(self.assets.home.checkbox_selected.clone()),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(parent).add_child(check);
    }

    fn add_status(&self, commands: &mut Commands, parent: Entity) {
        let status = spawn_node(
            commands,
            parent,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(5),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            status,
            reference_level(self.player.reference_points),
            10.5,
            MUTED,
            self.assets,
        );
        add_text(
            commands,
            status,
            if self.player.ready {
                "已准备"
            } else {
                "未准备"
            },
            12.5,
            if self.player.ready { READY } else { MUTED },
            self.assets,
        );
    }
}

struct EmptyLobbySeat<'a> {
    index: u8,
    assets: &'a UiAssets,
}

impl<'a> EmptyLobbySeat<'a> {
    fn new(index: u8, assets: &'a UiAssets) -> Self {
        Self { index, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let empty_ring = spawn_node(
            commands,
            parent,
            Node {
                width: px(64),
                height: px(64),
                position_type: PositionType::Relative,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        let mut image = ImageNode::new(self.assets.home.button.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(32.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.42,
            }),
        );
        image.visual_box = VisualBox::BorderBox;
        commands
            .entity(empty_ring)
            .insert((LobbyEmptySeatRing(self.index), image));
        let plus = add_text(commands, empty_ring, "+", 31.0, TEXT, self.assets);
        commands.entity(plus).insert(Pickable::IGNORE);
        let label = add_text(commands, parent, "空位", 12.0, MUTED, self.assets);
        commands
            .entity(label)
            .insert(LobbyEmptySeatLabel(self.index));
    }
}

fn lobby_seat_position(seat: u8) -> (f32, f32) {
    match seat {
        0 => (150.0, 290.0),
        1 => (0.0, 145.0),
        2 => (150.0, 0.0),
        3 => (328.0, 0.0),
        4 => (478.0, 145.0),
        5 => (328.0, 290.0),
        _ => unreachable!("there are exactly six table seats"),
    }
}
