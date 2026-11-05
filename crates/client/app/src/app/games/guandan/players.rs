use super::cards::PLAY_AREA_WIDTH;
use super::summary::placement_label;
use super::table::GuandanTable;
use crate::app::presentation::{
    PlayerMenuProfile, PlayerPortraitSpec, SeatSide, TEXT, TRICK_AVATAR_SIZE, TRICK_PORTRAIT_WIDTH,
    TrickSeatLayout, TurnBorderAnimationKey, add_hand_count, add_player_portrait,
    add_player_role_badge, add_player_role_badge_row, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition,
};
use bevy::prelude::*;
use leocard_protocol::{GameKind, GuandanPlayerView};
use leocard_ui::{UiComposer, spawn_node};

impl GuandanTable<'_> {
    pub(super) fn players(
        &mut self,
        commands: &mut UiComposer,
        table: Entity,
        hand_area: Entity,
        intro: bool,
    ) -> [Option<Entity>; 4] {
        let mut portraits = [None; 4];
        let own_seat = self
            .game
            .players
            .iter()
            .find(|player| player.player.id == self.game.you)
            .map_or(0, |player| player.seat.0);
        for player in &self.game.players {
            let relative = (player.seat.0 + 4 - own_seat) % 4;
            if player.player.id == self.game.you {
                portraits[player.player.id.index()] = Some(self.portrait(
                    commands,
                    hand_area,
                    player,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(12),
                        bottom: px(42),
                        width: px(TRICK_PORTRAIT_WIDTH),
                        height: px(76.0 * 1.17),
                        ..default()
                    },
                    intro,
                ));
                if !intro {
                    let area = spawn_node(
                        commands,
                        table,
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(35),
                            right: percent(35),
                            bottom: px(8),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        None,
                    );
                    self.play_area(commands, area, player.player.id, SeatSide::Top);
                }
                continue;
            }
            let layout = TrickSeatLayout::new(relative, 4, PLAY_AREA_WIDTH);
            let side = layout.side;
            let slot = layout.spawn(commands, table);
            if matches!(side, SeatSide::Right) && !intro {
                self.play_area(commands, slot, player.player.id, side);
            }
            portraits[player.player.id.index()] = Some(self.portrait(
                commands,
                slot,
                player,
                Node {
                    width: px(TRICK_PORTRAIT_WIDTH),
                    height: px(76.0 * 1.17),
                    margin: UiRect::bottom(px(if matches!(side, SeatSide::Top) {
                        27.0
                    } else {
                        0.0
                    })),
                    ..default()
                },
                intro,
            ));
            if !matches!(side, SeatSide::Right) && !intro {
                self.play_area(commands, slot, player.player.id, side);
            }
        }
        portraits
    }

    fn portrait(
        &mut self,
        commands: &mut UiComposer,
        parent: Entity,
        player: &GuandanPlayerView,
        node: Node,
        intro: bool,
    ) -> Entity {
        let portrait = add_player_portrait(
            commands,
            parent,
            node,
            PlayerPortraitSpec {
                player: player.player.id,
                profile: PlayerMenuProfile {
                    name: &player.player.name,
                    avatar: player
                        .player
                        .avatar
                        .and_then(|id| self.avatars.remote.get(&id)),
                    reference_points: player.player.reference_points,
                    completed_games: player.player.completed_games,
                    game_profiles: &player.player.game_profiles,
                },
                avatar_size: TRICK_AVATAR_SIZE,
                auto_play: player.auto_play,
                menu_open: !intro && self.social.interaction_menu_open == Some(player.player.id),
                name_color: TEXT,
            },
            self.assets,
        );
        attach_start_game_seat_transition(commands, portrait.portrait, player.player.id, intro);
        if intro {
            return portrait.avatar_ring;
        }
        if self.game.current == Some(player.player.id) && self.game.result.is_none() {
            add_turn_border_trace_with_radius(
                commands,
                portrait.avatar_ring,
                self.turn_materials,
                TurnBorderAnimationKey::new(
                    GameKind::Guandan,
                    self.game.match_id,
                    player.player.id,
                ),
                TRICK_AVATAR_SIZE * 0.2,
                TRICK_AVATAR_SIZE,
            );
        }
        if let Some(place) = self
            .game
            .finishing_order
            .iter()
            .position(|id| *id == player.player.id)
        {
            let row = add_player_role_badge_row(commands, portrait.avatar_ring);
            add_player_role_badge(commands, row, placement_label(place), self.assets);
        }
        let count = spawn_node(
            commands,
            portrait.portrait,
            Node {
                position_type: PositionType::Absolute,
                top: px(76.0 * 1.17 + 2.0),
                width: percent(100),
                height: px(27),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(4),
                ..default()
            },
            None,
        );
        add_hand_count(commands, count, player.hand_len, self.assets);
        portrait.avatar_ring
    }
}
