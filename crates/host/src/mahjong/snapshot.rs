use super::*;

impl MahjongSession {
    pub(super) fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::Mahjong, GameRules::Mahjong(self.rules))
    }

    pub(super) fn broadcast_lobby(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::Mahjong, GameRules::Mahjong(self.rules), origin)
    }

    pub(super) fn broadcast_game(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        assert!(
            self.game.is_some(),
            "game broadcast requires a Mahjong game"
        );
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                self.room.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::Mahjong(self.game_snapshot(player.id))),
                )
            })
            .collect()
    }

    pub(super) fn broadcast_events(&self, events: Vec<MahjongEvent>) -> Vec<Delivery> {
        events
            .into_iter()
            .flat_map(|event| {
                self.room
                    .players
                    .iter()
                    .filter(|player| player.connected && !player.left)
                    .map(move |player| {
                        self.room.delivery(
                            player.connection,
                            None,
                            ServerEvent::GameEvent(GameEvent::Mahjong(event.clone())),
                        )
                    })
            })
            .collect()
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> MahjongSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("a Mahjong snapshot requires a game");
        let core_recipient = to_core_player(recipient);
        let players = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|participant| {
                let core_id = to_core_player(participant.id);
                let public = game
                    .public_player(core_recipient, core_id)
                    .expect("room and Mahjong players stay aligned");
                MahjongPlayerState {
                    id: participant.id,
                    profile_id: participant.profile_id,
                    name: participant.name.clone(),
                    avatar: participant.avatar,
                    seat: participant.seat.expect("started players retain seats"),
                    seat_wind: game.seat_wind(core_id).expect("started player has a wind"),
                    concealed_count: public.concealed_count as u8,
                    revealed_hand: public.revealed_hand,
                    melds: public
                        .melds
                        .into_iter()
                        .map(|meld| MahjongPublicMeldView {
                            kind: meld.kind,
                            tile: meld.tile,
                            claimed_from: meld.claimed_from.map(from_core_player),
                        })
                        .collect(),
                    flowers: public.flowers,
                    dead_hand: public.dead_hand,
                    ready: participant.ready,
                    connected: participant.connected || participant.is_bot,
                    reference_points: participant.reference_points,
                    completed_games: participant.completed_games,
                    game_profiles: participant.game_profiles.clone(),
                }
            })
            .collect();
        let own = game
            .player(core_recipient)
            .expect("room and Mahjong players stay aligned");
        let pending_claim = match game.phase() {
            Phase::WaitingForClaims(pending) => Some(MahjongPendingClaimView {
                source: from_core_player(pending.source_player()),
                tile: pending.tile(),
                robbing_kong: pending.is_robbing_kong_window(),
                your_options: pending
                    .options_for(core_recipient)
                    .unwrap_or_default()
                    .to_vec(),
                your_response: pending.response_from(core_recipient),
                waiting_for: pending
                    .waiting_for()
                    .into_iter()
                    .map(from_core_player)
                    .collect(),
            }),
            Phase::Dealing { .. }
            | Phase::ReplacingFlower { .. }
            | Phase::Playing
            | Phase::Finished(_) => None,
        };
        let can_declare_kong = matches!(game.phase(), Phase::Playing)
            && game.current_player() == core_recipient
            && game.wall_len() > 0;
        let concealed_kong_options = if can_declare_kong {
            let mut kinds = own
                .hand()
                .iter()
                .map(|tile| tile.kind())
                .collect::<Vec<_>>();
            kinds.sort_unstable();
            kinds
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .into_iter()
                .filter(|kind| kinds.iter().filter(|held| **held == *kind).count() == 4)
                .collect()
        } else {
            Vec::new()
        };
        let pung_kinds = own
            .melds()
            .iter()
            .filter(|meld| meld.kind() == MahjongMeldKind::Pung)
            .map(|meld| meld.tile())
            .collect::<HashSet<_>>();
        let added_kong_options = if can_declare_kong {
            own.hand()
                .iter()
                .copied()
                .filter(|tile| pung_kinds.contains(&tile.kind()))
                .collect()
        } else {
            Vec::new()
        };
        let phase = match game.phase() {
            Phase::Dealing { batch } => MahjongPhaseView::Dealing { batch: *batch },
            Phase::ReplacingFlower { player } => MahjongPhaseView::ReplacingFlower {
                player: from_core_player(*player),
            },
            Phase::Playing => MahjongPhaseView::Playing,
            Phase::WaitingForClaims(_) => MahjongPhaseView::WaitingForClaims,
            Phase::Finished(result) => MahjongPhaseView::Finished {
                result: hand_result_view(result),
            },
        };
        MahjongSnapshot {
            match_id: self.match_id.expect("started Mahjong game has a match id"),
            host_port: self.room.host_port,
            you: recipient,
            host: self
                .room
                .host_player_id()
                .expect("started Mahjong room retains a host"),
            rules: self.rules,
            players,
            your_hand: own.hand().to_vec(),
            your_drawn_tile: (game.current_player() == core_recipient)
                .then(|| game.last_drawn())
                .flatten(),
            discards: game
                .discards()
                .iter()
                .map(|discard| MahjongDiscardView {
                    player: from_core_player(discard.player),
                    tile: discard.tile,
                    claimed_by: discard.claimed_by.map(from_core_player),
                })
                .collect(),
            dealer: from_core_player(game.dealer()),
            prevalent_wind: game.prevalent_wind(),
            sequence_index: game.sequence_index(),
            current_player: from_core_player(game.current_player()),
            wall_len: game.wall_len() as u16,
            match_scores: *game.match_scores(),
            pending_claim,
            can_self_draw: game.self_draw_available(core_recipient).unwrap_or(false),
            concealed_kong_options,
            added_kong_options,
            phase,
        }
    }

    pub(super) fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: &GameError,
    ) -> Vec<Delivery> {
        let violation = match error {
            GameError::InvalidPlayer(_) => MahjongViolation::InvalidPlayer,
            GameError::NotPlayersTurn { .. } => MahjongViolation::NotPlayersTurn,
            GameError::WrongPhase => MahjongViolation::WrongPhase,
            GameError::TileNotInHand(_) => MahjongViolation::TileNotInHand,
            GameError::InvalidClaim => MahjongViolation::InvalidClaim,
            GameError::AlreadyResponded => MahjongViolation::AlreadyResponded,
            GameError::CannotWin => MahjongViolation::CannotWin,
            GameError::CannotKong => MahjongViolation::CannotKong,
            GameError::InvalidHandReplacement => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidDeveloperHand,
                );
            }
            GameError::InvalidRules(_)
            | GameError::InvalidDeckSize { .. }
            | GameError::InvalidDeckContents
            | GameError::Score(_) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
                );
            }
        };
        self.room.reject(
            connection,
            request_id,
            RejectReason::GameViolation(GameViolation::Mahjong(violation)),
        )
    }
}
