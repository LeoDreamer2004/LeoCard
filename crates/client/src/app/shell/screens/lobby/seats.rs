use super::*;
use crate::app::presentation::{
    ACCENT, LobbyEmptySeatLabel, LobbyEmptySeatRing, LobbySeatHover, LobbySeatTransitionSource,
    LobbySeatVisual, MUTED, READY, TEXT, add_avatar, add_host_crown, add_text, spawn_node,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction, reference_level};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
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
                align_self: AlignSelf::Center,
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
            FocusPolicy::Pass,
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
        let avatar_ring = spawn_node(
            commands,
            parent,
            Node {
                width: px(60),
                height: px(60),
                min_width: px(60),
                position_type: PositionType::Relative,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(if self.is_you { 3 } else { 2 })),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::srgba(0.02, 0.08, 0.06, 0.80)),
        );
        commands.entity(avatar_ring).insert((
            BorderColor::all(if self.is_you {
                ACCENT
            } else if self.player.ready {
                READY
            } else {
                MUTED.with_alpha(0.42)
            }),
            BoxShadow::new(
                if self.is_you {
                    ACCENT.with_alpha(0.24)
                } else {
                    Color::BLACK.with_alpha(0.24)
                },
                px(0),
                px(2),
                px(0),
                px(5),
            ),
        ));
        avatar_ring
    }

    fn add_ready_badge(&self, commands: &mut Commands, parent: Entity) {
        let check = spawn_node(
            commands,
            parent,
            Node {
                position_type: PositionType::Absolute,
                right: px(-2),
                bottom: px(-1),
                width: px(18),
                height: px(18),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(READY),
        );
        add_text(commands, check, "✓", 11.5, Color::WHITE, self.assets);
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
                width: px(56),
                height: px(56),
                position_type: PositionType::Relative,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::srgba(0.03, 0.12, 0.085, 0.64)),
        );
        commands.entity(empty_ring).insert((
            LobbyEmptySeatRing(self.index),
            BorderColor::all(MUTED.with_alpha(0.34)),
        ));
        self.add_plus_sign(commands, empty_ring);
        let label = add_text(commands, parent, "空位", 12.0, MUTED, self.assets);
        commands
            .entity(label)
            .insert(LobbyEmptySeatLabel(self.index));
    }

    fn add_plus_sign(&self, commands: &mut Commands, parent: Entity) {
        for node in [
            Node {
                position_type: PositionType::Absolute,
                left: px(15),
                top: px(25),
                width: px(22),
                height: px(2),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(25),
                top: px(15),
                width: px(2),
                height: px(22),
                ..default()
            },
        ] {
            let stroke = spawn_node(commands, parent, node, Some(MUTED.with_alpha(0.72)));
            commands.entity(stroke).insert(FocusPolicy::Pass);
        }
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
