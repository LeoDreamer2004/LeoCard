use super::cards::render_card_row;
use super::table::GuandanTable;
use crate::app::presentation::{
    ACCENT, MUTED, PlayerMenuProfile, PlayerPortraitSpec, SeatSide, TEXT, TRICK_AVATAR_SIZE,
    TRICK_PORTRAIT_WIDTH, TrickSeatLayout, TurnBorderAnimationKey, add_hand_count,
    add_player_portrait, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition, spawn_trick_play_area,
};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, GuandanPlayerView, GuandanPublicAction};
use leocard_ui::{UiComposer, spawn_node};

impl GuandanTable<'_> {
    pub(super) fn players(
        &mut self,
        commands: &mut UiComposer,
        table: Entity,
        hand_area: Entity,
        intro: bool,
    ) {
        let own_seat = self
            .game
            .players
            .iter()
            .find(|player| player.player.id == self.game.you)
            .map_or(0, |player| player.seat.0);
        for player in &self.game.players {
            let relative = (player.seat.0 + 4 - own_seat) % 4;
            if player.player.id == self.game.you {
                self.portrait(
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
                );
                if !intro {
                    let area = spawn_node(
                        commands,
                        table,
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(35),
                            right: percent(35),
                            bottom: px(8),
                            min_height: px(100),
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
            let layout = TrickSeatLayout::new(relative, 4, 190.0);
            let side = layout.side;
            let slot = layout.spawn(commands, table);
            if matches!(side, SeatSide::Right) && !intro {
                self.play_area(commands, slot, player.player.id, side);
            }
            self.portrait(
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
            );
            if !matches!(side, SeatSide::Right) && !intro {
                self.play_area(commands, slot, player.player.id, side);
            }
        }
    }

    fn portrait(
        &mut self,
        commands: &mut UiComposer,
        parent: Entity,
        player: &GuandanPlayerView,
        node: Node,
        intro: bool,
    ) {
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
            return;
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
        let own_team = self
            .game
            .players
            .iter()
            .find(|own| own.player.id == self.game.you)
            .map(|own| own.seat.0 % 2);
        let team = player.seat.0 % 2;
        let badge_label = match own_team {
            Some(_) if player.player.id == self.game.you => "我方",
            Some(own) if team == own => "搭档",
            Some(_) => "对手",
            None if team == 0 => "一队",
            None => "二队",
        };
        let badge = spawn_node(
            commands,
            portrait.portrait,
            Node {
                position_type: PositionType::Absolute,
                right: px(-12),
                top: px(-6),
                padding: UiRect::axes(px(5), px(2)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Some(Color::BLACK.with_alpha(0.55)),
        );
        add_text(
            commands,
            badge,
            badge_label,
            13.0,
            if own_team == Some(team) { ACCENT } else { TEXT },
            self.assets,
        );
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
    }

    fn play_area(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        player: PlayerId,
        side: SeatSide,
    ) {
        let area = spawn_trick_play_area(commands, parent, side);
        let action = self
            .game
            .last_actions
            .get(player.index())
            .and_then(Option::as_ref);
        match action {
            Some(GuandanPublicAction::Play(cards)) => {
                render_card_row(commands, area, cards, self.assets);
            }
            Some(GuandanPublicAction::Pass) => {
                add_text(commands, area, "不出", 28.0, MUTED, self.assets);
            }
            None => {}
        }
    }
}
