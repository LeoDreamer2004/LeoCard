//! 通用准备大厅分派与席位组件。

use super::*;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::{GameKind, GameRules, LobbyPlayer, LobbySnapshot, SeatId, TABLE_SEAT_COUNT};
use leocard_shengji::ShengjiRuleSet;
use leocard_uno::UnoRuleSet;

pub(super) struct LobbyScreen<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    ui: &'a UiState,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> LobbyScreen<'a> {
    pub(super) fn new(
        client: &'a ClientResource,
        lobby: &'a LobbySnapshot,
        ui: &'a UiState,
        assets: &'a UiAssets,
        avatars: &'a AvatarImages,
    ) -> Self {
        Self {
            client,
            lobby,
            ui,
            assets,
            avatars,
        }
    }

    pub(super) fn render(self, commands: &mut Commands, root: Entity) {
        let Self {
            client,
            lobby,
            ui,
            assets,
            avatars,
        } = self;
        match lobby.game {
            GameKind::QiGui523 => {
                render_qigui523_lobby(commands, root, client, lobby, assets, avatars)
            }
            GameKind::TexasHoldem => {
                render_texas_holdem_lobby(commands, root, client, lobby, assets, avatars)
            }
            GameKind::Shengji => {
                render_shengji_lobby(commands, root, client, lobby, assets, avatars)
            }
            GameKind::Uno => render_uno_lobby(commands, root, client, lobby, ui, assets, avatars),
            GameKind::Mahjong => {
                render_mahjong_lobby(commands, root, client, lobby, assets, avatars)
            }
        }
    }
}

pub struct LobbySeatSelector<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    assets: &'a UiAssets,
    avatars: &'a AvatarImages,
}

impl<'a> LobbySeatSelector<'a> {
    pub fn new(
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

    pub fn render(self, commands: &mut Commands, parent: Entity) {
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
        let metrics = LobbyMetrics::new(lobby);
        LobbyTable::new(client, lobby, assets).render(commands, ring);

        for seat_index in 0..metrics.seat_count() {
            LobbySeat::new(client, lobby, assets, avatars, seat_index).render(commands, ring);
        }
    }
}

struct LobbyTable<'a> {
    client: &'a ClientResource,
    lobby: &'a LobbySnapshot,
    assets: &'a UiAssets,
}

impl<'a> LobbyTable<'a> {
    fn new(client: &'a ClientResource, lobby: &'a LobbySnapshot, assets: &'a UiAssets) -> Self {
        Self {
            client,
            lobby,
            assets,
        }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let table = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(150),
                    top: px(108),
                    width: px(320),
                    height: px(174),
                    padding: UiRect::axes(px(24), px(18)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(12),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(percent(50)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ImageNode::new(self.assets.table_felt.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(Color::srgba(0.72, 0.83, 0.76, 0.92)),
                BackgroundColor(TABLE_BG),
                BorderColor::all(Color::srgba(0.62, 0.82, 0.70, 0.28)),
                BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(7), px(0), px(9)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(parent).add_child(table);

        let metrics = LobbyMetrics::new(self.lobby);
        add_text(
            commands,
            table,
            format!(
                "等待准备  {}/{}",
                metrics.ready_player_count(),
                metrics.connected_player_count()
            ),
            17.0,
            TEXT,
            self.assets,
        );
        #[cfg(feature = "developer")]
        if self.client.0.model().you() == self.lobby.host {
            add_text(
                commands,
                table,
                "右键空座添加机器人，右键机器人移除",
                10.5,
                MUTED,
                self.assets,
            );
        }
        #[cfg(not(feature = "developer"))]
        let _ = self.client;
        LobbyRuleSummary::new(&self.lobby.rules, self.assets).render(commands, table);
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

struct LobbyRuleSummary<'a> {
    rules: &'a GameRules,
    assets: &'a UiAssets,
}

impl<'a> LobbyRuleSummary<'a> {
    fn new(rules: &'a GameRules, assets: &'a UiAssets) -> Self {
        Self { rules, assets }
    }

    fn render(self, commands: &mut Commands, parent: Entity) {
        let chips = spawn_node(
            commands,
            parent,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        for label in self.labels() {
            self.add_chip(commands, chips, label);
        }
    }

    fn labels(&self) -> [String; 3] {
        match self.rules {
            GameRules::QiGui523(rules) => [
                format!("{}副", rules.deck_count),
                format!("{}张", rules.hand_size),
                time_control_label(rules.time_control).to_owned(),
            ],
            GameRules::TexasHoldem(rules) => [
                format!("{}筹码", rules.starting_chips),
                match (rules.omaha, rules.short_deck) {
                    (true, true) => "短牌奥马哈".to_owned(),
                    (true, false) => "奥马哈".to_owned(),
                    (false, true) => "短牌德州".to_owned(),
                    (false, false) => "标准德州".to_owned(),
                },
                if rules.ignore_kickers {
                    "只比较最大牌型".to_owned()
                } else {
                    "标准比牌".to_owned()
                },
            ],
            GameRules::Shengji(rules) => [
                format!("{}副牌", rules.deck_count),
                if rules.bottom_copy {
                    "允许抄底".to_owned()
                } else {
                    "不抄底".to_owned()
                },
                if rules.five_trump_crossing {
                    "五主过江".to_owned()
                } else {
                    "不过江".to_owned()
                },
            ],
            GameRules::Uno(rules) if rules.is_no_mercy() => [
                "No Mercy".to_owned(),
                "+2 至 +10 递增堆叠".to_owned(),
                if rules.no_mercy.mercy_elimination {
                    "25 张淘汰".to_owned()
                } else {
                    "不启用慈悲淘汰".to_owned()
                },
            ],
            GameRules::Uno(rules) if rules.is_flip() => [
                "UNO FLIP".to_owned(),
                if rules.flip.random_pairing {
                    "随机双面配对".to_owned()
                } else {
                    "固定双面配对".to_owned()
                },
                if rules.flip.action_stacking {
                    "功能牌可堆叠".to_owned()
                } else {
                    "功能牌不堆叠".to_owned()
                },
            ],
            GameRules::Uno(rules) => [
                if rules.action_stacking {
                    "功能牌可堆叠".to_owned()
                } else {
                    "功能牌不堆叠".to_owned()
                },
                if rules.jump_in {
                    "允许抢出".to_owned()
                } else {
                    "不抢出".to_owned()
                },
                if rules.uno_callout {
                    "UNO 检举".to_owned()
                } else {
                    "不检举".to_owned()
                },
            ],
            GameRules::Mahjong(rules) => [
                match rules.match_length {
                    MahjongMatchLength::SingleHand => "单局结算".to_owned(),
                    MahjongMatchLength::EastRound => "东风场".to_owned(),
                    MahjongMatchLength::HalfGame => "半庄场".to_owned(),
                    MahjongMatchLength::FullGame => "全庄场".to_owned(),
                },
                if rules.minimum_eight_points {
                    "8 番起和".to_owned()
                } else {
                    "不限起和番数".to_owned()
                },
                if rules.multiple_winners {
                    "允许一炮多响".to_owned()
                } else {
                    "截和".to_owned()
                },
            ],
        }
    }

    fn add_chip(&self, commands: &mut Commands, parent: Entity, label: String) {
        let chip = spawn_node(
            commands,
            parent,
            Node {
                min_height: px(25),
                padding: UiRect::axes(px(8), px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            Some(Color::srgba(0.015, 0.065, 0.048, 0.72)),
        );
        commands
            .entity(chip)
            .insert(BorderColor::all(Color::srgba(0.70, 0.88, 0.78, 0.18)));
        add_text(commands, chip, label, 11.5, TEXT, self.assets);
    }
}

pub struct LobbyMetrics<'a> {
    lobby: &'a LobbySnapshot,
}

impl<'a> LobbyMetrics<'a> {
    pub fn new(lobby: &'a LobbySnapshot) -> Self {
        Self { lobby }
    }

    pub fn connected_player_count(&self) -> usize {
        self.lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .count()
    }

    pub fn ready_player_count(&self) -> usize {
        self.lobby
            .players
            .iter()
            .filter(|player| player.connected && player.ready)
            .count()
    }

    pub fn seat_count(&self) -> u8 {
        match &self.lobby.rules {
            GameRules::Shengji(_) => ShengjiRuleSet::PLAYER_COUNT as u8,
            GameRules::Uno(_) => UnoRuleSet::MAX_PLAYERS,
            GameRules::Mahjong(_) => MahjongRuleSet::PLAYER_COUNT as u8,
            GameRules::QiGui523(_) | GameRules::TexasHoldem(_) => TABLE_SEAT_COUNT,
        }
    }
}
