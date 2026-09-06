use super::{QiGui523Session, from_core_player, merge_qigui523_play_stats, record_qigui523_play};
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    GameEvent, GameKind, GamePhaseView, GameRules, GameSnapshot, LobbySnapshot, PlayerId,
    PlayerPublicState, PlayerReferenceChange, PlayerScore, PublicPlay, PublicPlayRecord,
    QiGui523Event, QiGui523ProfileStats, QiGui523Snapshot, RejectReason, RequestId, RevealedHand,
    ServerEvent, StartingCardView, TrickView,
};
use leocard_qigui523::{Phase, PlayRecord, reference_point_deltas};

impl QiGui523Session {
    pub(super) fn snapshot(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::QiGui523(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.delivery(connection, Some(request_id), event)]
    }

    pub(super) fn broadcast_lobby(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::QiGui523, GameRules::QiGui523(self.rules), origin)
    }

    pub(super) fn broadcast_game(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.players
            .iter()
            .filter(|player| player.connected)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                self.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::QiGui523(
                        self.game_snapshot(player.id),
                    )),
                )
            })
            .collect()
    }

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
            return self
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| self.delivery(player.connection, None, ServerEvent::RoomClosed))
                .collect();
        }

        let mut deliveries = Vec::new();
        for (_, name) in departed {
            deliveries.extend(
                self.players
                    .iter()
                    .filter(|player| player.connected && !player.left)
                    .map(|player| {
                        self.delivery(
                            player.connection,
                            None,
                            ServerEvent::PlayerLeft { name: name.clone() },
                        )
                    }),
            );
        }
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }

    pub(super) fn apply_finished_reference_points(&mut self) {
        if self.finished_reference_changes.is_some() {
            return;
        }
        let Some(scores) = self.game.as_ref().and_then(|game| match game.phase() {
            Phase::Finished(result) => Some(result.scores.clone()),
            Phase::Playing => None,
        }) else {
            return;
        };
        let deltas = reference_point_deltas(&scores)
            .expect("a running game always contains between two and six players");
        let mut changes = Vec::with_capacity(self.players.len());
        let match_profile_stats = self.match_profile_stats.clone();
        self.room.prepare_rematch();
        for (index, ((player, delta), score)) in self
            .players
            .iter_mut()
            .zip(deltas)
            .zip(scores.iter().copied())
            .enumerate()
        {
            player.reference_points = player.reference_points.saturating_add(i32::from(delta));
            player.completed_games = player.completed_games.saturating_add(1);
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
            changes.push(PlayerReferenceChange {
                player: player.id,
                profile_id: player.profile_id,
                delta,
            });
        }
        self.finished_reference_changes = Some(changes);
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
        self.players
            .iter()
            .filter(|participant| participant.connected && !participant.left)
            .map(|participant| {
                self.delivery(
                    participant.connection,
                    None,
                    ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect {
                        player,
                        play: play.clone(),
                    })),
                )
            })
            .collect()
    }

    pub(super) fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::QiGui523, GameRules::QiGui523(self.rules))
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> QiGui523Snapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");
        let recipient_index = usize::from(recipient.0);
        let players = game
            .players()
            .iter()
            .zip(&self.players)
            .map(|(state, participant)| PlayerPublicState {
                id: participant.id,
                profile_id: participant.profile_id,
                name: participant.name.clone(),
                avatar: participant.avatar,
                seat: participant
                    .seat
                    .expect("all game participants have selected seats"),
                hand_len: state.hand().len() as u16,
                score: state.score(),
                ready: participant.ready,
                connected: participant.connected || participant.is_bot,
                auto_play: participant.auto_play,
                reference_points: participant.reference_points,
                completed_games: participant.completed_games,
                game_profiles: participant.game_profiles.clone(),
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
