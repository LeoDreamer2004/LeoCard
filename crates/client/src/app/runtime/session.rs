//! 网络快照轮询与客户端状态同步。

use leocard_client::{NetworkState, TcpGameClient};
use leocard_protocol::{ShengjiFiveTrumpCrossingStage, ShengjiPhaseView};

use super::*;

#[derive(Resource)]
pub struct ClientResource(pub TcpGameClient);

pub fn poll_network(
    mut client: Option<ResMut<ClientResource>>,
    mut form: ResMut<ConnectionForm>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
    mut seat_transition: ResMut<StartGameSeatTransition>,
    lobby_seats: Query<(
        &LobbySeatTransitionSource,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut commands: Commands,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let previous_state = client.0.state().clone();
    let previous_game = client.0.model().qigui523_game().cloned();
    let previous_shengji = client.0.model().shengji_game().cloned();
    let previous_lobby = client.0.model().lobby().cloned();
    let mut lobby_seat_snapshots = previous_lobby
        .as_ref()
        .map(|lobby| {
            lobby_seats
                .iter()
                .filter_map(|(source, node, transform)| {
                    let player = lobby.players.iter().find(|player| player.id == source.0)?;
                    let size = node.size() * node.inverse_scale_factor();
                    (size.min_element() > 1.0).then(|| LobbySeatTransitionSnapshot {
                        player: player.id,
                        center_global: transform.to_scale_angle_translation().2,
                        size,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let was_host = client
        .0
        .model()
        .qigui523_game()
        .is_some_and(|game| game.you == game.host)
        || client
            .0
            .model()
            .texas_holdem_game()
            .is_some_and(|game| game.you == game.host)
        || client
            .0
            .model()
            .shengji_game()
            .is_some_and(|game| game.you == game.host)
        || client
            .0
            .model()
            .uno_game()
            .is_some_and(|game| game.you == game.host)
        || client.0.model().lobby().is_some_and(|lobby| {
            client
                .0
                .model()
                .you()
                .is_some_and(|you| lobby.host == Some(you))
        });
    if !client.0.poll() {
        return;
    }
    let started_game = client
        .0
        .model()
        .qigui523_game()
        .map(|game| {
            (
                game.match_id,
                game.players
                    .iter()
                    .map(|player| player.id)
                    .collect::<Vec<_>>(),
            )
        })
        .or_else(|| {
            client.0.model().texas_holdem_game().map(|game| {
                (
                    game.match_id,
                    game.players
                        .iter()
                        .map(|player| player.id)
                        .collect::<Vec<_>>(),
                )
            })
        })
        .or_else(|| {
            client.0.model().shengji_game().map(|game| {
                (
                    game.match_id,
                    game.players
                        .iter()
                        .map(|player| player.id)
                        .collect::<Vec<_>>(),
                )
            })
        })
        .or_else(|| {
            client.0.model().uno_game().map(|game| {
                (
                    game.match_id,
                    game.players
                        .iter()
                        .map(|player| player.id)
                        .collect::<Vec<_>>(),
                )
            })
        });
    if previous_lobby.is_some() {
        if let Some((match_id, players)) = started_game.as_ref() {
            lobby_seat_snapshots.retain(|seat| players.contains(&seat.player));
            seat_transition.begin(*match_id, lobby_seat_snapshots);
        } else {
            seat_transition.clear();
        }
    } else if client.0.model().lobby().is_some() || started_game.is_none() {
        seat_transition.clear();
    }
    let mut profile_changed = client
        .0
        .model()
        .last_finished_match()
        .is_some_and(|(match_id, changes)| profile.apply_finished_match(match_id, changes));
    let authoritative_qigui523_profile = client.0.model().qigui523_game().and_then(|game| {
        game.players
            .iter()
            .find(|player| player.id == game.you)
            .and_then(|player| player.game_profiles.qigui523.as_ref())
    });
    if let Some(stats) = authoritative_qigui523_profile {
        profile_changed |= profile.sync_qigui523_profile(stats);
    }
    let authoritative_texas_holdem_profile =
        client.0.model().texas_holdem_game().and_then(|game| {
            game.players
                .iter()
                .find(|player| player.id == game.you)
                .and_then(|player| player.game_profiles.texas_holdem.as_ref())
        });
    if let Some(stats) = authoritative_texas_holdem_profile {
        profile_changed |= profile.sync_texas_holdem_profile(stats);
    }
    let authoritative_shengji_profile = client.0.model().shengji_game().and_then(|game| {
        game.players
            .iter()
            .find(|player| player.id == game.you)
            .and_then(|player| player.game_profiles.shengji.as_ref())
    });
    if let Some(stats) = authoritative_shengji_profile {
        profile_changed |= profile.sync_shengji_profile(stats);
    }
    let authoritative_uno_profile = client.0.model().uno_game().and_then(|game| {
        game.players
            .iter()
            .find(|player| player.id == game.you)
            .and_then(|player| player.game_profiles.uno.as_ref())
    });
    if let Some(stats) = authoritative_uno_profile {
        profile_changed |= profile.sync_uno_profile(stats);
    }
    let authoritative_interaction_profile = client
        .0
        .model()
        .qigui523_game()
        .and_then(|game| {
            game.players
                .iter()
                .find(|player| player.id == game.you)
                .and_then(|player| player.game_profiles.interactions.as_ref())
        })
        .or_else(|| {
            client.0.model().texas_holdem_game().and_then(|game| {
                game.players
                    .iter()
                    .find(|player| player.id == game.you)
                    .and_then(|player| player.game_profiles.interactions.as_ref())
            })
        })
        .or_else(|| {
            client.0.model().shengji_game().and_then(|game| {
                game.players
                    .iter()
                    .find(|player| player.id == game.you)
                    .and_then(|player| player.game_profiles.interactions.as_ref())
            })
        })
        .or_else(|| {
            client.0.model().uno_game().and_then(|game| {
                game.players
                    .iter()
                    .find(|player| player.id == game.you)
                    .and_then(|player| player.game_profiles.interactions.as_ref())
            })
        });
    if let Some(stats) = authoritative_interaction_profile {
        profile_changed |= profile.sync_interaction_profile(stats);
    }
    if profile_changed && let Err(error) = profile.save() {
        form.error = Some(error);
    }
    let accepted_host_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.qigui523_rules().copied())
            .flatten()
            .map(normalize_host_rules)
    });
    if let Some(rules) = accepted_host_rules
        && form.host_rules != rules
    {
        form.host_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    let accepted_texas_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.texas_holdem_rules().copied())
            .flatten()
            .map(normalize_texas_holdem_rules)
    });
    if let Some(rules) = accepted_texas_rules
        && form.texas_holdem_rules != rules
    {
        form.texas_holdem_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    let accepted_shengji_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.shengji_rules().copied())
            .flatten()
            .map(normalize_shengji_rules)
    });
    if let Some(rules) = accepted_shengji_rules
        && form.shengji_rules != rules
    {
        form.shengji_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    let accepted_uno_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.uno_rules().copied())
            .flatten()
            .map(normalize_uno_rules)
    });
    if let Some(rules) = accepted_uno_rules
        && form.uno_rules != rules
    {
        form.uno_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    let accepted_mahjong_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.mahjong_rules().copied())
            .flatten()
            .map(normalize_mahjong_rules)
    });
    if let Some(rules) = accepted_mahjong_rules
        && form.mahjong_rules != rules
    {
        form.mahjong_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    if let Some(game) = client.0.model().shengji_game() {
        let previous_stage = previous_shengji
            .as_ref()
            .and_then(|snapshot| match &snapshot.phase {
                ShengjiPhaseView::FiveTrumpCrossing { stage, .. } => Some(*stage),
                _ => None,
            });
        if let ShengjiPhaseView::FiveTrumpCrossing {
            stage,
            eligible,
            decided,
            ..
        } = &game.phase
            && previous_stage != Some(*stage)
        {
            ui.shengji.selected.clear();
            if *stage == ShengjiFiveTrumpCrossingStage::Deciding
                && eligible.contains(&game.you)
                && !decided.contains(&game.you)
                && let Some(trump) = game.trump
            {
                ui.shengji.selected.extend(
                    game.your_hand
                        .iter()
                        .copied()
                        .filter(|card| trump.is_trump(*card)),
                );
            }
        }
    }
    if client.0.model().room_closed() {
        if was_host || ui.leaving_room {
            form.error = None;
        } else {
            form.error = Some("房主结束了游戏".to_owned());
        }
        ui.leaving_room = false;
        commands.remove_resource::<ClientResource>();
    } else if client.0.model().left_room() {
        form.error = None;
        ui.leaving_room = false;
        commands.remove_resource::<ClientResource>();
    } else if let NetworkState::Failed(error) = client.0.state() {
        form.error = (!ui.leaving_room).then(|| error.clone());
        ui.leaving_room = false;
        commands.remove_resource::<ClientResource>();
    }
    if previous_state == *client.0.state()
        && (only_turn_timer_changed(previous_game.as_ref(), client.0.model().qigui523_game())
            || only_shengji_transient_progress_changed(
                previous_shengji.as_ref(),
                client.0.model().shengji_game(),
            ))
    {
        return;
    }
    ui.dirty = true;
}
