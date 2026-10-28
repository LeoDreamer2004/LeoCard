use super::DoudizhuTable;
use crate::app::presentation::{
    ACCENT, MUTED, PlayerMenuProfile, PlayerPortraitSpec, SeatSide, TEXT, TRICK_AVATAR_SIZE,
    TRICK_PORTRAIT_WIDTH, TrickSeatLayout, TurnBorderAnimationKey, add_hand_count_score,
    add_player_portrait, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition,
};

use bevy::prelude::*;
use leocard_doudizhu::Phase;
use leocard_protocol::{GameKind, PlayerPublicState};
use leocard_ui::{UiComposer, spawn_node};

impl DoudizhuTable<'_> {
    pub(super) fn players(
        &mut self,
        commands: &mut UiComposer,
        table: Entity,
        hand_area: Entity,
        intro: bool,
    ) {
        let game = self.game;
        let own = game.players.iter().find(|player| player.id == game.you);
        let own_seat = own.map_or(0, |player| player.seat.0);
        let count = game.rules.mode.players();
        for player in &game.players {
            let relative = (player.seat.0 + count - own_seat) % count;
            if Some(player.id) == own.map(|player| player.id) {
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
                            min_height: px(104),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        None,
                    );
                    self.play_area(commands, area, player.id, SeatSide::Top, false);
                }
                continue;
            }
            let layout = TrickSeatLayout::new(relative, count, 190.0);
            let side = layout.side;
            let slot = layout.spawn(commands, table);
            if matches!(side, SeatSide::Right) {
                self.play_area(commands, slot, player.id, side, intro);
            }
            self.portrait(
                commands,
                slot,
                player,
                Node {
                    width: px(TRICK_PORTRAIT_WIDTH),
                    height: px(76.0 * 1.17),
                    // Leave room for the card count below the top opponent's portrait.
                    margin: UiRect::bottom(px(if matches!(side, SeatSide::Top) {
                        27.0
                    } else {
                        0.0
                    })),
                    ..default()
                },
                intro,
            );
            if !matches!(side, SeatSide::Right) {
                self.play_area(commands, slot, player.id, side, intro);
            }
        }
    }

    fn portrait(
        &mut self,
        commands: &mut UiComposer,
        parent: Entity,
        player: &PlayerPublicState,
        node: Node,
        intro: bool,
    ) {
        let portrait = add_player_portrait(
            commands,
            parent,
            node,
            PlayerPortraitSpec {
                player: player.id,
                profile: PlayerMenuProfile {
                    name: &player.name,
                    avatar: player.avatar.and_then(|id| self.avatars.remote.get(&id)),
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                    game_profiles: &player.game_profiles,
                },
                avatar_size: TRICK_AVATAR_SIZE,
                auto_play: player.auto_play,
                menu_open: !intro && self.social.interaction_menu_open == Some(player.id),
                name_color: if player.connected { TEXT } else { MUTED },
            },
            self.assets,
        );
        attach_start_game_seat_transition(commands, portrait.portrait, player.id, intro);
        if intro {
            return;
        }
        if self.game.current == player.id && self.game.result.is_none() {
            add_turn_border_trace_with_radius(
                commands,
                portrait.avatar_ring,
                self.turn_materials,
                TurnBorderAnimationKey::new(GameKind::Doudizhu, self.game.match_id, player.id),
                TRICK_AVATAR_SIZE * 0.2,
                TRICK_AVATAR_SIZE,
            );
        }
        if self.game.phase != Phase::Bidding {
            let landlord = self.game.landlord == Some(player.id);
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
                if landlord { "地主" } else { "农民" },
                13.0,
                if landlord { ACCENT } else { TEXT },
                self.assets,
            );
        }
        add_hand_count_score(
            commands,
            portrait.portrait,
            player.hand_len,
            i64::from(self.game.scores[usize::from(player.id.0)]),
            self.assets,
        );
    }
}
