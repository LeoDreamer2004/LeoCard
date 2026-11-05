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
    let avatar_size = if cfg!(target_os = "android") {
        74.0
    } else {
        58.0
    };
    for (index, player) in game.players.iter().enumerate() {
        let own = player.player.id == game.you;
        let seat = commands.keyed_child(
            table,
            format!("player:{}", player.player.id.0),
            (Node {
                position_type: PositionType::Absolute,
                left: px(28),
                top: px(12.0 + index as f32 * 150.0),
                width: px(210),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(4),
                ..default()
            },),
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
            if cfg!(target_os = "android") {
                24.0
            } else {
                21.0
            },
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
