use super::{QiGui523Session, from_core_player, merge_qigui523_play_stats, record_qigui523_play};
use crate::lifecycle::HostedGameLifecycle;
use crate::player::settle_completed_match_profiles_once;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    GamePhaseView, PlayerId, PlayerPublicState, PlayerScore, PublicPlay, PublicPlayRecord,
    QiGui523Event, QiGui523ProfileStats, QiGui523Snapshot, RequestId, RevealedHand, ServerEvent,
    StartingCardView, TrickView,
};
use leocard_qigui523::{Phase, PlayRecord, reference_point_deltas};

impl QiGui523Session {
    pub(super) fn broadcast_game_after_update(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.apply_finished_reference_points();
        let finished = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)));
        if !finished {
            return self.broadcast_game(origin);
        }

        let departed = self
            .players
            .iter()
            .filter(|player| !player.connected && !player.is_bot && !player.left)
            .map(|player| (player.connection, player.name.clone()))
            .collect::<Vec<_>>();
        if departed.is_empty() {
            return self.broadcast_game(origin);
        }
        for player in &mut self.players {
            if !player.connected && !player.is_bot {
                player.ready = false;
                player.left = true;
            }
        }

        if self
            .host_connection
            .is_some_and(|host| departed.iter().any(|(connection, _)| *connection == host))
        {
            self.closed = true;
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
        }

        let mut deliveries = Vec::new();
        for (_, name) in departed {
            deliveries.extend(
                self.room
                    .broadcast_event(None, ServerEvent::PlayerLeft { name }),
            );
        }
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }

    pub(super) fn apply_finished_reference_points(&mut self) {
        let Some(scores) = self.game.as_ref().and_then(|game| match game.phase() {
            Phase::Finished(result) => Some(result.scores.clone()),
            Phase::Playing => None,
        }) else {
            return;
        };
        let deltas = reference_point_deltas(&scores)
            .expect("a running game always contains between two and six players");
        let settlements = self
            .players
            .iter()
            .zip(deltas)
            .map(|(player, delta)| (player.id, delta))
            .collect::<Vec<_>>();
        let match_profile_stats = self.match_profile_stats.clone();
        let applied = settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            &mut self.room,
            settlements,
            |player, delta| {
                let index = usize::from(player.id.0);
                let score = scores[index];
                let placement = 1 + scores.iter().filter(|other| **other > score).count();
                let aggregate = player
                    .game_profiles
                    .qigui523
                    .get_or_insert_with(QiGui523ProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_score = aggregate.total_score.saturating_add(u64::from(score));
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                if let Some(count) = aggregate.placement_counts.get_mut(placement - 1) {
                    *count = count.saturating_add(1);
                }
                if let Some(current) = match_profile_stats.get(index) {
                    merge_qigui523_play_stats(aggregate, current);
                }
            },
        );
        if applied {
            self.room.prepare_rematch();
        }
    }

    pub(super) fn broadcast_game_after_action(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
        effect: Option<(PlayerId, PublicPlay)>,
    ) -> Vec<Delivery> {
        if let Some((player, play)) = effect.as_ref()
            && let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0))
        {
            record_qigui523_play(stats, &play.kind);
        }
        let mut deliveries = effect
            .map(|effect| self.broadcast_play_effect(effect))
            .unwrap_or_default();
        deliveries.extend(self.broadcast_game_after_update(origin));
        deliveries
    }

    pub(super) fn broadcast_play_effect(&self, effect: (PlayerId, PublicPlay)) -> Vec<Delivery> {
        let (player, play) = effect;
        self.room
            .broadcast_game_events([QiGui523Event::PlayEffect { player, play }])
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> QiGui523Snapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");
        let recipient_index = usize::from(recipient.0);
        let players = game
            .players()
            .iter()
            .zip(&self.players)
            .map(|(state, participant)| {
                let public = participant.public_metadata();
                PlayerPublicState {
                    id: public.id,
                    profile_id: public.profile_id,
                    name: public.name,
                    avatar: public.avatar,
                    seat: public
                        .seat
                        .expect("all game participants have selected seats"),
                    hand_len: state.hand().len() as u16,
                    score: state.score(),
                    ready: public.ready,
                    connected: public.connected,
                    auto_play: public.auto_play,
                    reference_points: public.reference_points,
                    completed_games: public.completed_games,
                    game_profiles: public.game_profiles,
                }
            })
            .collect();
        let starting = game.starting_card();

        QiGui523Snapshot {
            match_id: self.match_id.expect("a running game has a match id"),
            host_port: self.host_port,
            you: recipient,
            host: self
                .host_connection
                .and_then(|connection| {
                    self.players
                        .iter()
                        .find(|player| player.connection == connection)
                        .map(|player| player.id)
                })
                .expect("a running game has a room host"),
            players,
            your_hand: game.players()[recipient_index].hand().to_vec(),
            draw_pile_len: game.draw_pile_len() as u16,
            starting_card: StartingCardView {
                player: from_core_player(starting.player),
                card: starting.card,
            },
            trick: game.trick().map(|trick| TrickView {
                leader: from_core_player(trick.leader()),
                current_player: from_core_player(trick.current_player()),
                winning_player: trick.winning_player().map(from_core_player),
                winning_play: trick.winning_play().map(|play| PublicPlay {
                    kind: play.kind().clone(),
                    cards: play.cards().to_vec(),
                }),
                records: trick
                    .records()
                    .iter()
                    .map(|record| match record {
                        PlayRecord::Played { player, play } => PublicPlayRecord::Played {
                            player: from_core_player(*player),
                            play: PublicPlay {
                                kind: play.kind().clone(),
                                cards: play.cards().to_vec(),
                            },
                        },
                        PlayRecord::Passed { player } => PublicPlayRecord::Passed {
                            player: from_core_player(*player),
                        },
                    })
                    .collect(),
                table_points: trick.table_points(),
            }),
            turn_timer: self.turn_timer_view(),
            phase: match game.phase() {
                Phase::Playing => GamePhaseView::Playing,
                Phase::Finished(result) => GamePhaseView::Finished {
                    match_id: self.match_id.expect("a running game has a match id"),
                    finisher: from_core_player(result.finisher),
                    scores: result
                        .scores
                        .iter()
                        .enumerate()
                        .map(|(index, score)| PlayerScore {
                            player: PlayerId(index as u8),
                            score: *score,
                        })
                        .collect(),
                    remaining_hands: game
                        .players()
                        .iter()
                        .enumerate()
                        .map(|(index, player)| RevealedHand {
                            player: PlayerId(index as u8),
                            cards: player.hand().to_vec(),
                        })
                        .collect(),
                    reference_changes: self
                        .finished_reference_changes
                        .clone()
                        .expect("finished points are applied before broadcasting a snapshot"),
                    captured_hand_points: result.captured_hand_points,
                },
            },
        }
    }
}
