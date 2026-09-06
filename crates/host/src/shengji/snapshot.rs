use super::*;
use leocard_shengji::BottomCopyState;

impl ShengjiSession {
    pub(super) fn snapshot(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::Shengji, GameRules::Shengji(self.rules))
    }

    pub(super) fn broadcast_lobby(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::Shengji, GameRules::Shengji(self.rules), origin)
    }

    pub(super) fn broadcast_game(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
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
                    ServerEvent::GameSnapshot(GameSnapshot::Shengji(self.game_snapshot(player.id))),
                )
            })
            .collect()
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> ShengjiSnapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");
        let core_recipient = to_core_player(recipient);
        let your_hand = game.players()[usize::from(core_recipient.0)].hand.clone();
        let visible_trick = game.current_trick().or_else(|| {
            self.presentation
                .trick
                .as_ref()
                .map(|(trick, _)| trick.clone())
        });
        let trick = visible_trick.map(|trick| ShengjiTrickView {
            leader: from_core_player(trick.leader),
            current_player: from_core_player(game.current_player().unwrap_or(trick.leader)),
            winning_player: from_core_player(trick.winner),
            plays: trick
                .plays
                .into_iter()
                .map(|(player, play)| ShengjiPublicPlay {
                    player: from_core_player(player),
                    play,
                    throw_penalty: self.presentation.throw_penalties[usize::from(player.0)],
                })
                .collect(),
            table_points: trick.points,
        });
        ShengjiSnapshot {
            match_id: self.match_id.expect("started game has a match id"),
            hand_number: self.hand_number,
            host_port: self.room.host_port,
            you: recipient,
            host: self.room.host_player_id().expect("started room has a host"),
            rules: self.rules,
            players: self
                .room
                .players
                .iter()
                .filter(|player| {
                    !player.left || game.players().get(usize::from(player.id.0)).is_some()
                })
                .map(|player| ShengjiPlayerState {
                    id: player.id,
                    profile_id: player.profile_id,
                    name: player.name.clone(),
                    avatar: player.avatar,
                    seat: player.seat.expect("started player has a seat"),
                    hand_len: game
                        .players()
                        .get(usize::from(player.id.0))
                        .map_or(0, |state| state.hand.len() as u8),
                    ready: player.ready,
                    connected: (player.connected || player.is_bot) && !player.left,
                    auto_play: player.auto_play,
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                    game_profiles: player.game_profiles.clone(),
                })
                .collect(),
            your_hand,
            your_exposed_cards: game.bidding().exposed_cards(core_recipient),
            levels: game.teams().levels,
            bidding_level: game.bidding().level(),
            dealer: game.dealer().map(from_core_player),
            trump: game.trump(),
            declaration: self.declaration_view(),
            current_player: (self.presentation.trick.is_none()
                && self.presentation.throw_failure.is_none()
                && self.flow.bottom_flip_remaining.is_none())
            .then(|| game.current_player().map(from_core_player))
            .flatten(),
            trick,
            throw_failure: self.presentation.throw_failure.as_ref().map(|failure| {
                ShengjiThrowFailureView {
                    player: from_core_player(failure.player),
                    attempted: failure.attempted.clone(),
                    forced: failure.forced.clone(),
                    penalty_points: failure.penalty_points,
                    stage: failure.stage,
                }
            }),
            collecting_score: if self.presentation.trick.is_some() {
                match game.phase() {
                    Phase::Finished(result) => pre_kitty_collecting_score(result),
                    _ => game.collecting_score(),
                }
            } else {
                game.collecting_score()
            },
            buried_count: game.buried().len() as u8,
            your_buried: if game.bottom_burier() == Some(core_recipient) {
                game.buried().to_vec()
            } else {
                Vec::new()
            },
            phase: self.phase_view(game, recipient),
        }
    }

    fn phase_view(&self, game: &GameState, recipient: PlayerId) -> ShengjiPhaseView {
        if self.flow.bottom_flip_remaining.is_some() {
            return self.bottom_flip_phase_view();
        }
        if self.presentation.trick.is_some() && matches!(game.phase(), Phase::Finished(_)) {
            return ShengjiPhaseView::Playing;
        }
        match game.phase() {
            Phase::Dealing => {
                let dealt = game
                    .players()
                    .iter()
                    .map(|player| player.hand.len())
                    .sum::<usize>();
                ShengjiPhaseView::Dealing {
                    cards_remaining: (ShengjiRuleSet::PLAYER_COUNT * self.rules.hand_size() - dealt)
                        as u8,
                }
            }
            Phase::BiddingGrace => ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: self
                    .flow
                    .bidding_remaining
                    .unwrap_or(BIDDING_GRACE)
                    .as_millis()
                    .min(u128::from(u16::MAX)) as u16,
                power_outage: game.power_outage_used(),
                confirmed_count: self
                    .flow
                    .bid_pass_confirmed
                    .iter()
                    .filter(|confirmed| **confirmed)
                    .count() as u8,
                you_confirmed: self
                    .flow
                    .bid_pass_confirmed
                    .get(usize::from(recipient.0))
                    .copied()
                    .unwrap_or(false),
            },
            Phase::BottomFlipping => self.bottom_flip_phase_view(),
            Phase::Burying => ShengjiPhaseView::Burying,
            Phase::BottomCopying => ShengjiPhaseView::BottomCopying {
                player: from_core_player(
                    game.bottom_copy()
                        .and_then(BottomCopyState::current)
                        .expect("抄底询问阶段始终有当前玩家"),
                ),
                milliseconds_remaining: self
                    .flow
                    .bottom_copy_remaining
                    .unwrap_or(BOTTOM_COPY_DECISION_TIMEOUT)
                    .as_millis()
                    .min(u128::from(u16::MAX)) as u16,
            },
            Phase::BottomCopyBurying => ShengjiPhaseView::BottomCopyBurying {
                player: from_core_player(
                    game.bottom_copy()
                        .and_then(BottomCopyState::bottom_holder)
                        .expect("抄底重埋阶段始终有持底玩家"),
                ),
            },
            Phase::FiveTrumpCrossing => {
                let crossing = game
                    .five_trump_crossing()
                    .expect("过江阶段始终保留过江状态");
                let players = (0..ShengjiRuleSet::PLAYER_COUNT as u8).map(ShengjiPlayerId);
                let eligible = players
                    .clone()
                    .filter(|player| crossing.eligible(*player))
                    .map(from_core_player)
                    .collect();
                let decided = players
                    .clone()
                    .filter(|player| crossing.decision_made(*player))
                    .map(from_core_player)
                    .collect();
                // 决定阶段只公布谁已经完成选择，不提前泄露其是否过江。
                let crossing_players = if crossing.stage() == FiveTrumpCrossingStage::Returning {
                    players
                        .clone()
                        .filter(|player| crossing.crossing(*player))
                        .map(from_core_player)
                        .collect()
                } else {
                    Vec::new()
                };
                let returned = players
                    .filter(|player| crossing.return_made(*player))
                    .map(from_core_player)
                    .collect();
                ShengjiPhaseView::FiveTrumpCrossing {
                    stage: match crossing.stage() {
                        FiveTrumpCrossingStage::Deciding => ShengjiFiveTrumpCrossingStage::Deciding,
                        FiveTrumpCrossingStage::Returning => {
                            ShengjiFiveTrumpCrossingStage::Returning
                        }
                    },
                    eligible,
                    decided,
                    crossing: crossing_players,
                    returned,
                }
            }
            Phase::Playing => ShengjiPhaseView::Playing,
            Phase::Finished(result) => ShengjiPhaseView::Finished {
                result: self.hand_result_view(result),
                buried: game.buried().to_vec(),
            },
            Phase::RedealRequired => ShengjiPhaseView::Redealing,
        }
    }

    fn bottom_flip_phase_view(&self) -> ShengjiPhaseView {
        ShengjiPhaseView::BottomFlipping {
            reveal: self
                .flow
                .bottom_flip_reveal
                .as_ref()
                .map(bottom_flip_reveal_view),
        }
    }

    pub(super) fn declaration_view(&self) -> Option<ShengjiDeclarationView> {
        let declaration = self.game.as_ref()?.bidding().current()?;
        Some(ShengjiDeclarationView {
            player: from_core_player(declaration.player),
            trump: declaration.trump,
            kind: declaration.kind,
            protected: declaration.protected,
            cards: declaration.cards.clone(),
        })
    }
}
