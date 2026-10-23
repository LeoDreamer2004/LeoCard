use super::{
    BlackjackTable, areas::add_blackjack_hand_areas, chips::add_blackjack_balance,
    hands::BlackjackHandSpec, layout::BlackjackSeatLayout,
};
use crate::app::presentation::{
    ACCENT, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT, TurnBorderAnimationKey, UiComposer,
    add_player_portrait, add_poker_chip_value, add_ready_badge, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition, spawn_node,
};
use bevy::prelude::*;
use leocard_blackjack::Phase;
use leocard_protocol::{GameKind, SeatId, TABLE_SEAT_COUNT};

impl BlackjackTable<'_> {
    pub(super) fn players(&mut self, commands: &mut UiComposer, table: Entity, pile: Entity) {
        let own_seat = self
            .game
            .players
            .iter()
            .find(|view| view.player.id == self.game.you)
            .and_then(|view| view.player.seat)
            .unwrap_or(SeatId(0));
        let intro = self.transition.is_active_for(self.game.match_id);
        let areas = (!intro).then(|| add_blackjack_hand_areas(commands, table));
        for view in &self.game.players {
            let player = &view.player;
            let own = player.id == self.game.you;
            let relative = (player.seat.expect("seated player").0 + TABLE_SEAT_COUNT - own_seat.0)
                % TABLE_SEAT_COUNT;
            let layout = BlackjackSeatLayout::new(relative);
            let active = self.game.current_player == Some(player.id);
            let seat = commands.keyed_child(
                table,
                format!("blackjack-player:{}", player.id.0),
                (layout.portrait,),
            );
            let avatar = if own {
                self.avatars.local.as_ref()
            } else {
                player.avatar.and_then(|id| self.avatars.remote.get(&id))
            };
            let portrait = add_player_portrait(
                commands,
                seat,
                Node {
                    width: percent(100),
                    ..default()
                },
                PlayerPortraitSpec {
                    player: player.id,
                    profile: PlayerMenuProfile {
                        name: &player.name,
                        avatar,
                        reference_points: player.reference_points,
                        completed_games: player.completed_games,
                        game_profiles: &player.game_profiles,
                    },
                    avatar_size: 62.0,
                    auto_play: view.auto_play,
                    menu_open: self.social.interaction_menu_open == Some(player.id),
                    name_color: if active {
                        ACCENT
                    } else if player.connected {
                        TEXT
                    } else {
                        MUTED
                    },
                },
                self.assets,
            );
            attach_start_game_seat_transition(
                commands,
                if own { portrait.portrait } else { seat },
                player.id,
                intro,
            );
            let stack = if own {
                add_blackjack_balance(
                    commands,
                    seat,
                    self.playback.displayed_stack(view),
                    self.assets,
                )
            } else {
                let row = spawn_node(
                    commands,
                    portrait.portrait,
                    Node {
                        width: percent(100),
                        height: px(32),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        column_gap: px(5),
                        ..default()
                    },
                    None,
                );
                add_poker_chip_value(
                    commands,
                    row,
                    self.playback.displayed_stack(view),
                    self.assets,
                );
                row
            };
            if player.ready && self.game.phase == Phase::Complete {
                add_ready_badge(commands, portrait.avatar_ring, 62.0, self.assets);
            }
            if active {
                add_turn_border_trace_with_radius(
                    commands,
                    portrait.avatar_ring,
                    self.turn_materials,
                    TurnBorderAnimationKey::new(GameKind::Blackjack, self.game.match_id, player.id),
                    12.4,
                    62.0,
                );
            }
            let Some(areas) = &areas else {
                continue;
            };
            if own {
                self.own_wagers(commands, table, view, stack, pile);
            }
            let hands = areas[usize::from(relative)];
            if !own && view.hands.is_empty() {
                self.waiting_hand(commands, hands, view.stack < self.game.rules.minimum_bet);
            }
            for (index, hand) in view.hands.iter().enumerate() {
                self.hand(
                    commands,
                    hands,
                    BlackjackHandSpec {
                        hand,
                        player: player.id,
                        index,
                        own,
                        width: ((layout.width - 12.0) / view.hands.len() as f32 - 6.0)
                            .min(if own { 270.0 } else { 220.0 }),
                        active: active && self.game.current_hand == Some(index as u8),
                        wager_height: layout.wager_height,
                        pile,
                        stack,
                    },
                );
            }
        }
    }
}
