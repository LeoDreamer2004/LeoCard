use super::playback::{LinkGainVisual, LinkPlayback, LinkScoreVisual};
use crate::app::games::GameTableContext;
use crate::app::presentation::{
    ACCENT, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT, TurnBorderAnimationKey,
    add_player_portrait, add_ready_badge, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition,
};
use bevy::prelude::*;
use leocard_protocol::{GameKind, MahjongLinkSnapshot};
use leocard_ui::UiComposer;

pub(super) fn render(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongLinkSnapshot,
    playback: &LinkPlayback,
    context: &mut GameTableContext<'_>,
) {
    let dense = game.players.len() > 4;
    let avatar_size = match (dense, cfg!(target_os = "android")) {
        (true, true) => 52.0,
        (true, false) => 48.0,
        (false, true) => 74.0,
        (false, false) => 58.0,
    };
    let score_size = if dense {
        18.0
    } else if cfg!(target_os = "android") {
        24.0
    } else {
        21.0
    };
    let own_index = game
        .players
        .iter()
        .position(|player| player.player.id == game.you)
        .unwrap_or(0);
    for (index, player) in game.players.iter().enumerate() {
        let relative = (index + game.players.len() - own_index) % game.players.len();
        let (left, top) = seat_position(relative, game.players.len());
        let own = player.player.id == game.you;
        let seat = commands.keyed_child(
            table,
            format!("player:{}", player.player.id.0),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: if left { px(12) } else { Val::Auto },
                    right: if left { Val::Auto } else { px(12) },
                    top: percent(top),
                    width: px(210),
                    height: if dense { px(106) } else { Val::Auto },
                    margin: UiRect::top(px(if dense { -53.0 } else { 0.0 })),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(4),
                    ..default()
                },
                ZIndex(100),
            ),
        );
        let avatar = if own {
            context.avatars.local.as_ref()
        } else {
            player
                .player
                .avatar
                .and_then(|id| context.avatars.remote.get(&id))
        };
        let portrait = add_player_portrait(
            commands,
            seat,
            Node {
                width: percent(100),
                ..default()
            },
            PlayerPortraitSpec {
                player: player.player.id,
                profile: PlayerMenuProfile {
                    name: &player.player.name,
                    avatar,
                    reference_points: player.player.reference_points,
                    completed_games: player.player.completed_games,
                    game_profiles: &player.player.game_profiles,
                },
                avatar_size,
                auto_play: player.auto_play,
                menu_open: context.social.interaction_menu_open == Some(player.player.id),
                name_color: if own { ACCENT } else { TEXT },
            },
            context.assets,
        );
        attach_start_game_seat_transition(
            commands,
            seat,
            player.player.id,
            context.transition.is_active_for(game.match_id),
        );
        let score = add_text(
            commands,
            seat,
            format!("{} 分", player.score),
            score_size,
            if player.connected { TEXT } else { MUTED },
            context.assets,
        );
        let gained = playback.gained(player.player.id);
        if gained > 0 {
            commands.entity(score).insert(LinkScoreVisual {
                before: u32::from(player.score.saturating_sub(gained)),
                after: u32::from(player.score),
                started: playback.started,
            });
            let gain = add_text(
                commands,
                seat,
                format!("+{gained}"),
                22.0,
                ACCENT,
                context.assets,
            );
            commands.entity(gain).insert((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    bottom: px(0),
                    ..default()
                },
                UiTransform::IDENTITY,
                LinkGainVisual(playback.started),
            ));
        }
        if player.ready && game.match_complete {
            add_ready_badge(commands, portrait.avatar_ring, avatar_size, context.assets);
        }
        if game.current_player == Some(player.player.id) {
            add_turn_border_trace_with_radius(
                commands,
                portrait.avatar_ring,
                context.turn_border_materials,
                TurnBorderAnimationKey::new(GameKind::MahjongLink, game.match_id, player.player.id),
                avatar_size * 0.2,
                avatar_size,
            );
        }
    }
}

fn seat_position(relative: usize, count: usize) -> (bool, f32) {
    if count <= 2 {
        return (relative == 0, 38.0);
    }
    if count <= 4 {
        return (
            matches!(relative, 0 | 3),
            if matches!(relative, 2 | 3) {
                20.0
            } else {
                58.0
            },
        );
    }
    let right_count = count / 2;
    let left = relative == 0 || relative > right_count;
    let (row, rows) = if relative == 0 {
        (count - right_count - 1, count - right_count)
    } else if left {
        (relative - right_count - 1, count - right_count)
    } else {
        (right_count - relative, right_count)
    };
    (left, 12.5 + 75.0 * row as f32 / (rows - 1) as f32)
}
