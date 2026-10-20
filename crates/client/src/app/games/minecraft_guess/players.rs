use super::feedback::GuessWinner;
use super::process_presence::add_process_presence_overlay;
use super::{
    scores::{GuessScoreDelta, GuessScoreValue},
    view::GuessTable,
};
use crate::app::presentation::{
    ACCENT, DANGER, MUTED, PlayerMenuProfile, PlayerPortraitSpec, READY, TEXT,
    TurnBorderAnimationKey, UiComposer, add_player_portrait, add_ready_badge, add_text,
    add_turn_border_trace_with_radius, attach_start_game_seat_transition, score_gain_visual,
    spawn_node,
};
use bevy::prelude::*;
use leocard_protocol::GameKind;

impl GuessTable<'_> {
    pub(super) fn players(&mut self, commands: &mut UiComposer, page: Entity) {
        let compact = self.game.graph.is_some();
        let avatar_size = if compact { 44.0 } else { 52.0 };
        let columns = if compact {
            self.game.players.len().min(4)
        } else {
            self.game.players.len()
        };
        let rows = self.game.players.len().div_ceil(columns);
        let row = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                height: px(if compact { 78.0 * rows as f32 } else { 88.0 }),
                flex_wrap: if compact {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::NoWrap
                },
                flex_shrink: 0.0,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        for player in &self.game.players {
            let active = self.game.round.current_player == Some(player.id.0);
            let avatar = if player.id == self.game.you {
                self.avatars.local.as_ref()
            } else {
                player.avatar.and_then(|id| self.avatars.remote.get(&id))
            };
            let portrait = add_player_portrait(
                commands,
                row,
                Node {
                    width: percent(100.0 / columns as f32),
                    min_width: px(0),
                    flex_shrink: 0.0,
                    height: px(if compact { 74.0 } else { 84.0 }),
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
                    avatar_size,
                    auto_play: false,
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
            if compact {
                commands
                    .entity(portrait.name)
                    .entry::<TextFont>()
                    .and_modify(|mut font| font.font_size = px(14).into());
            }
            attach_start_game_seat_transition(
                commands,
                portrait.portrait,
                player.id,
                self.transition.is_active_for(self.game.match_id),
            );
            let score = self
                .game
                .scores
                .iter()
                .find(|score| score.player == player.id)
                .map_or(0, |score| score.score);
            let value = add_text(
                commands,
                portrait.avatar_ring,
                self.scores.score(player.id, score).to_string(),
                if compact { 26.0 } else { 30.0 },
                ACCENT,
                self.assets,
            );
            commands.entity(value).insert((
                GuessScoreValue(player.id),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(if compact { 54.0 } else { 64.0 }),
                    top: px(if compact { 9.0 } else { 7.0 }),
                    ..default()
                },
                TextShadow::default(),
            ));
            let gain = self.scores.gain(player.id);
            let (pose, alpha) = score_gain_visual(gain.map_or(0.0, |(_, elapsed)| elapsed));
            let delta = add_text(
                commands,
                portrait.avatar_ring,
                gain.map_or_else(String::new, |(gain, _)| {
                    format!("+{}", gain.after - gain.before)
                }),
                if compact { 18.0 } else { 22.0 },
                ACCENT.with_alpha(alpha),
                self.assets,
            );
            commands.entity(delta).insert((
                GuessScoreDelta(player.id),
                pose,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(if compact { 80.0 } else { 94.0 }),
                    top: px(0),
                    ..default()
                },
            ));
            let minecraft_running = self.game.minecraft_process_players.contains(&player.id);
            if self.game.round.winner == Some(player.id.0) && !minecraft_running {
                commands.entity(portrait.avatar_ring).insert(GuessWinner);
            }
            let skipped = self.game.round.skipped_players.contains(&player.id.0);
            if self.game.round.finished() && player.ready {
                add_ready_badge(commands, portrait.avatar_ring, avatar_size, self.assets);
            }
            commands
                .entity(portrait.avatar_ring)
                .entry::<Node>()
                .and_modify(move |mut node| {
                    node.border_radius = BorderRadius::all(px(avatar_size * 0.2))
                });
            if minecraft_running {
                commands
                    .entity(portrait.avatar_ring)
                    .insert(Outline::new(px(3), px(0), DANGER));
                add_process_presence_overlay(
                    commands,
                    portrait.avatar_ring,
                    avatar_size,
                    self.assets,
                );
            } else if !active {
                commands.entity(portrait.avatar_ring).insert(Outline::new(
                    px(3),
                    px(0),
                    if skipped {
                        Color::srgb(0.48, 0.49, 0.53)
                    } else {
                        READY
                    },
                ));
            }
            if active && !self.transition.is_active_for(self.game.match_id) {
                add_turn_border_trace_with_radius(
                    commands,
                    portrait.avatar_ring,
                    self.turn_border_materials,
                    TurnBorderAnimationKey::new(
                        GameKind::MinecraftGuess,
                        self.game.match_id,
                        player.id,
                    ),
                    avatar_size * 0.2,
                    avatar_size,
                );
            }
        }
    }
}
