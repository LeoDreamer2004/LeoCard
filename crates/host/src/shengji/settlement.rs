use super::*;
use leocard_protocol::{
    GameEvent, GameViolation, PlayerReferenceChange, RejectReason, RequestId, ServerEvent,
    ShengjiEvent, ShengjiHandResultView, ShengjiProfileStats, ShengjiPublicPlay,
    ShengjiThrowFailureStage,
};
use leocard_shengji::ShengjiTeamId;
use leocard_shengji::{
    ActionOutcome, FiveTrumpCrossingStage, GameError, GameState, HandResult, Phase, ShengjiBidKind,
    ShengjiClassifiedPlay, ShengjiPlayerId, ShengjiRuleSet,
};

impl ShengjiSession {
    pub(super) fn record_current_declaration(&mut self) {
        let Some((player, kind)) = self
            .game
            .as_ref()
            .and_then(|game| game.bidding().current())
            .map(|declaration| (declaration.player, declaration.kind))
        else {
            return;
        };
        let Some(stats) = self.statistics.profiles.get_mut(usize::from(player.0)) else {
            return;
        };
        match kind {
            ShengjiBidKind::Initial => stats.declaration_games = 1,
            ShengjiBidKind::Counter | ShengjiBidKind::SelfCounter => stats.counter_games = 1,
            ShengjiBidKind::Protect => {}
        }
    }

    pub(super) fn record_profile_outcome(&mut self, outcome: &ActionOutcome) {
        match outcome {
            ActionOutcome::BottomCopyDecision {
                player,
                copied: true,
                ..
            } => {
                if let Some(stats) = self.statistics.profiles.get_mut(usize::from(player.0)) {
                    stats.counter_games = 1;
                }
            }
            ActionOutcome::FiveTrumpCrossingDecision {
                player,
                crossing: true,
                ..
            } => {
                if let Some(stats) = self.statistics.profiles.get_mut(usize::from(player.0)) {
                    stats.crossing_games = 1;
                }
            }
            ActionOutcome::Played { player, .. } | ActionOutcome::ThrowFailed { player, .. } => {
                let play =
                    self.game
                        .as_ref()
                        .and_then(GameState::current_trick)
                        .and_then(|trick| {
                            trick.plays.last().map(|(_, play)| {
                                (play.clone(), trick.leader == *player, trick.winner)
                            })
                        });
                if let Some((play, is_lead, winner)) = play {
                    self.record_profile_play(*player, &play, is_lead, winner == *player);
                }
            }
            ActionOutcome::TrickComplete(trick) => {
                if let Some((player, play)) = trick.plays.last() {
                    self.record_profile_play(
                        *player,
                        play,
                        trick.leader == *player,
                        trick.winner == *player,
                    );
                }
            }
            ActionOutcome::HandComplete(_) => {
                let play = self
                    .game
                    .as_ref()
                    .and_then(|game| game.history().last())
                    .and_then(|trick| {
                        trick.plays.last().map(|(player, play)| {
                            (*player, play.clone(), trick.leader, trick.winner)
                        })
                    });
                if let Some((player, play, leader, winner)) = play {
                    self.record_profile_play(player, &play, leader == player, winner == player);
                }
            }
            _ => {}
        }
    }

    pub(super) fn record_profile_play(
        &mut self,
        player: ShengjiPlayerId,
        play: &ShengjiClassifiedPlay,
        is_lead: bool,
        is_winning: bool,
    ) {
        let Some(stats) = self.statistics.profiles.get_mut(usize::from(player.0)) else {
            return;
        };
        stats.plays = stats.plays.saturating_add(1);
        if is_winning {
            stats.winning_plays = stats.winning_plays.saturating_add(1);
        }
        if is_lead && play.is_throw() {
            stats.play_category_counts[4] = stats.play_category_counts[4].saturating_add(1);
            stats.longest_throw = stats
                .longest_throw
                .max(play.cards.len().min(usize::from(u16::MAX)) as u16);
            for component in &play.components {
                record_shengji_component(stats, component);
            }
        } else {
            record_shengji_component(stats, play.strongest_component());
        }
    }

    pub(super) fn events_for_outcome(&self, outcome: &ActionOutcome) -> Vec<ShengjiEvent> {
        match outcome {
            ActionOutcome::BuryComplete { leader } => vec![ShengjiEvent::CardsBuried {
                dealer: from_core_player(*leader),
            }],
            ActionOutcome::BottomCopyDecision { copied: true, .. } => self
                .declaration_view()
                .map(|declaration| vec![ShengjiEvent::BottomCopied { declaration }])
                .unwrap_or_default(),
            ActionOutcome::BottomCopyBuryComplete { player, .. } => {
                vec![ShengjiEvent::CardsBuried {
                    dealer: from_core_player(*player),
                }]
            }
            ActionOutcome::FiveTrumpCrossingDecision {
                decisions_complete: true,
                ..
            } => self
                .game
                .as_ref()
                .and_then(GameState::five_trump_crossing)
                .filter(|state| state.stage() == FiveTrumpCrossingStage::Returning)
                .map(|state| ShengjiEvent::FiveTrumpCrossingStarted {
                    players: (0..ShengjiRuleSet::PLAYER_COUNT as u8)
                        .map(ShengjiPlayerId)
                        .filter(|player| state.crossing(*player))
                        .map(from_core_player)
                        .collect(),
                })
                .into_iter()
                .collect(),
            ActionOutcome::FiveTrumpCrossingReturn {
                player,
                crossing_complete,
            } => vec![ShengjiEvent::FiveTrumpCrossingReturned {
                player: from_core_player(*player),
                complete: *crossing_complete,
            }],
            ActionOutcome::Played { player, .. } => self
                .current_public_play(*player, 0)
                .map(|(play, is_lead)| vec![ShengjiEvent::CardsPlayed { play, is_lead }])
                .unwrap_or_default(),
            ActionOutcome::ThrowFailed { .. } => Vec::new(),
            ActionOutcome::TrickComplete(trick) => {
                completed_trick_events(trick, self.game.as_ref().unwrap().collecting_score())
            }
            ActionOutcome::HandComplete(result) => {
                let game = self.game.as_ref().unwrap();
                let mut events = game
                    .history()
                    .last()
                    .map(|trick| completed_trick_events(trick, pre_kitty_collecting_score(result)))
                    .unwrap_or_default();
                events.push(ShengjiEvent::HandFinished {
                    result: self.hand_result_view(result),
                    buried: game.buried().to_vec(),
                });
                events
            }
            _ => Vec::new(),
        }
    }

    fn current_public_play(
        &self,
        player: ShengjiPlayerId,
        throw_penalty: u16,
    ) -> Option<(ShengjiPublicPlay, bool)> {
        let trick = self.game.as_ref()?.current_trick()?;
        let play = trick.plays.last()?.1.clone();
        let is_lead = trick.plays.len() == 1;
        Some((
            ShengjiPublicPlay {
                player: from_core_player(player),
                play,
                throw_penalty,
            },
            is_lead,
        ))
    }

    pub(super) fn after_game_outcome(&mut self, outcome: &ActionOutcome) {
        match outcome {
            ActionOutcome::BuryComplete { .. } => {
                self.flow.bottom_copy_remaining = self
                    .game
                    .as_ref()
                    .is_some_and(|game| matches!(game.phase(), Phase::BottomCopying))
                    .then_some(BOTTOM_COPY_DECISION_TIMEOUT);
            }
            ActionOutcome::BottomCopyDecision { copied, .. } => {
                self.flow.bottom_copy_remaining = (!*copied
                    && self
                        .game
                        .as_ref()
                        .is_some_and(|game| matches!(game.phase(), Phase::BottomCopying)))
                .then_some(BOTTOM_COPY_DECISION_TIMEOUT);
            }
            ActionOutcome::BottomCopyBuryComplete { .. } => {
                self.flow.bottom_copy_remaining = self
                    .game
                    .as_ref()
                    .is_some_and(|game| matches!(game.phase(), Phase::BottomCopying))
                    .then_some(BOTTOM_COPY_DECISION_TIMEOUT);
            }
            ActionOutcome::ThrowFailed {
                player,
                attempted,
                forced,
                penalty_points,
                ..
            } => {
                self.presentation.throw_penalties[usize::from(player.0)] = *penalty_points;
                self.presentation.throw_failure = Some(HeldThrowFailure {
                    player: *player,
                    attempted: attempted.clone(),
                    forced: forced.clone(),
                    penalty_points: *penalty_points,
                    stage: ShengjiThrowFailureStage::Showing,
                    remaining: THROW_FAILURE_SHOW_DURATION,
                });
            }
            ActionOutcome::TrickComplete(trick) => {
                self.presentation.trick = Some((trick.clone(), TRICK_HOLD_DURATION));
            }
            ActionOutcome::HandComplete(result) => {
                self.presentation.trick = self
                    .game
                    .as_ref()
                    .and_then(|game| game.history().last().cloned())
                    .map(|trick| (trick, TRICK_HOLD_DURATION));
                self.teams.levels = result.levels;
                self.next_dealer = Some(result.next_dealer);
                self.apply_finished_reference_points(result);
                self.room.prepare_rematch();
            }
            _ => {}
        }
    }

    pub(super) fn apply_finished_reference_points(&mut self, result: &HandResult) {
        if self.statistics.finished_reference_changes.is_some() {
            return;
        }
        let magnitude = finished_reference_point_magnitude(result);
        let hand_profile_stats = self.statistics.profiles.clone();
        let bottom_burier = self.game.as_ref().and_then(GameState::bottom_burier);
        let mut changes = Vec::with_capacity(ShengjiRuleSet::PLAYER_COUNT);
        for participant in &mut self.room.players {
            if usize::from(participant.id.0) >= ShengjiRuleSet::PLAYER_COUNT {
                continue;
            }
            let team = ShengjiTeamId(participant.id.0 % 2);
            let delta = if team == result.promoted_team {
                magnitude
            } else {
                -magnitude
            };
            participant.reference_points = participant
                .reference_points
                .saturating_add(i32::from(delta));
            participant.completed_games = participant.completed_games.saturating_add(1);
            let aggregate = participant
                .game_profiles
                .shengji
                .get_or_insert_with(ShengjiProfileStats::default);
            aggregate.completed_games = aggregate.completed_games.saturating_add(1);
            aggregate.total_reference_delta = aggregate
                .total_reference_delta
                .saturating_add(i64::from(delta));
            if team == result.dealer_team {
                aggregate.dealer_team_games = aggregate.dealer_team_games.saturating_add(1);
                aggregate.dealer_team_score = aggregate
                    .dealer_team_score
                    .saturating_add(u64::from(result.collecting_score));
                if result.kitty_multiplier == 0 {
                    aggregate.defended_kitty_games =
                        aggregate.defended_kitty_games.saturating_add(1);
                }
            } else {
                aggregate.collecting_team_games = aggregate.collecting_team_games.saturating_add(1);
                aggregate.collecting_team_score = aggregate
                    .collecting_team_score
                    .saturating_add(u64::from(result.collecting_score));
                if result.kitty_multiplier > 0 {
                    aggregate.captured_kitty_games =
                        aggregate.captured_kitty_games.saturating_add(1);
                }
            }
            if to_core_player(participant.id) == result.dealer {
                aggregate.dealer_games = aggregate.dealer_games.saturating_add(1);
            }
            if bottom_burier == Some(to_core_player(participant.id)) {
                aggregate.buried_games = aggregate.buried_games.saturating_add(1);
                aggregate.buried_points = aggregate
                    .buried_points
                    .saturating_add(u64::from(result.kitty_points));
            }
            if let Some(current) = hand_profile_stats.get(usize::from(participant.id.0)) {
                merge_shengji_profile_stats(aggregate, current);
            }
            changes.push(PlayerReferenceChange {
                player: participant.id,
                profile_id: participant.profile_id,
                delta,
            });
        }
        self.statistics.finished_settlement_id = Some(new_match_id());
        self.statistics.finished_reference_changes = Some(changes);
    }

    pub(super) fn hand_result_view(&self, result: &HandResult) -> ShengjiHandResultView {
        ShengjiHandResultView {
            settlement_id: self
                .statistics
                .finished_settlement_id
                .expect("完成的小局已经生成独立结算标识"),
            dealer: from_core_player(result.dealer),
            dealer_team: result.dealer_team,
            collecting_team: result.collecting_team,
            trick_points: result.trick_points,
            penalty_adjustment: result.penalty_adjustment,
            kitty_points: result.kitty_points,
            kitty_multiplier: result.kitty_multiplier,
            collecting_score: result.collecting_score,
            promoted_team: result.promoted_team,
            promoted_steps: result.promoted_steps,
            next_dealer: from_core_player(result.next_dealer),
            levels: result.levels,
            reference_changes: self
                .statistics
                .finished_reference_changes
                .clone()
                .expect("完成的小局已经计算平台积分变化"),
        }
    }

    pub(super) fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: GameError,
    ) -> Vec<Delivery> {
        self.room.reject(
            connection,
            request_id,
            RejectReason::GameViolation(GameViolation::Shengji(game_violation(error))),
        )
    }

    pub(super) fn broadcast_events(&self, events: Vec<ShengjiEvent>) -> Vec<Delivery> {
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
                            ServerEvent::GameEvent(GameEvent::Shengji(event.clone())),
                        )
                    })
            })
            .collect()
    }
}
