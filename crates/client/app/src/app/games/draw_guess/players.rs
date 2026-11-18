use super::feedback::{DrawingScoreDelta, DrawingScoreValue};
use super::view::DrawingContext;
use crate::app::presentation::{
    ACCENT, MUTED, PlayerMenuProfile, PlayerPortraitSpec, READY, SeatRows, TEXT,
    TurnBorderAnimationKey, add_player_portrait, add_ready_badge, add_text,
    add_turn_border_trace_with_radius, attach_start_game_seat_transition,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_draw_guess::DrawGuessPhase;
use leocard_protocol::{DrawGuessSnapshot, GameKind};
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DrawGuessSnapshot,
    context: &mut DrawingContext<'_>,
) {
    let layout = SeatRows::new(game.players.len() as u8, 8);
    let row = spawn_node(commands, parent, layout.node(82.0), None);
    for player in &game.players {
        let id = player.player.id;
        let state = game
            .state
            .players
            .iter()
            .find(|state| state.id == id)
            .expect("round roster");
        let active = game.state.artist == Some(id)
            && matches!(
                game.state.phase,
                DrawGuessPhase::Choosing | DrawGuessPhase::Drawing
            );
        let avatar = if id == game.you {
            context.avatars.local.as_ref()
        } else {
            player
                .player
                .avatar
                .and_then(|id| context.avatars.remote.get(&id))
        };
        let portrait = add_player_portrait(
            commands,
            row,
            Node {
                width: layout.cell_width(),
                min_width: px(0),
                height: px(80),
                flex_shrink: 0.0,
                ..default()
            },
            PlayerPortraitSpec {
                player: id,
                profile: PlayerMenuProfile {
                    name: &player.player.name,
                    avatar,
                    reference_points: player.player.reference_points,
                    completed_games: player.player.completed_games,
                    game_profiles: &player.player.game_profiles,
                },
                avatar_size: 44.0,
                auto_play: false,
                menu_open: context.social.interaction_menu_open == Some(id),
                name_color: if !player.connected {
                    MUTED
                } else if active {
                    ACCENT
                } else if state.guessed {
                    READY
                } else {
                    TEXT
                },
            },
            context.assets,
        );
        attach_start_game_seat_transition(
            commands,
            portrait.portrait,
            id,
            context.transition.is_active_for(game.match_id),
        );
        let score = add_text(
            commands,
            portrait.avatar_ring,
            state.score.to_string(),
            28.0,
            if state.guessed { READY } else { ACCENT },
            context.assets,
        );
        commands.entity(score).insert((
            DrawingScoreValue(id),
            Node {
                position_type: PositionType::Absolute,
                left: px(50),
                top: px(8),
                ..default()
            },
            Pickable::IGNORE,
        ));
        let gain = add_text(
            commands,
            portrait.avatar_ring,
            "",
            15.0,
            READY.with_alpha(0.0),
            context.assets,
        );
        commands.entity(gain).insert((
            DrawingScoreDelta(id),
            UiTransform::IDENTITY,
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(50),
                top: px(-12),
                ..default()
            },
            Pickable::IGNORE,
        ));
        if active && !context.transition.is_active_for(game.match_id) {
            add_turn_border_trace_with_radius(
                commands,
                portrait.avatar_ring,
                context.turn_border_materials,
                TurnBorderAnimationKey::new(GameKind::DrawGuess, game.match_id, id),
                44.0 * 0.2,
                44.0,
            );
        }
        if game.match_complete && player.ready {
            add_ready_badge(commands, portrait.avatar_ring, 44.0, context.assets);
        }
    }
}
