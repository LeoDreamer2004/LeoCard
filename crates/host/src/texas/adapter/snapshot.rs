use super::state::{AdapterError, TexasAdapter};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    PlayerPublicInfo, TexasBlindView, TexasEquity, TexasPhaseView, TexasPlayerView,
    TexasPotAwardView, TexasRevealedHand, TexasRunoutVoteView, TexasSnapshot, TexasSquidView,
};
use leocard_texas_holdem::{TexasPhase, TexasSquidTransfer, evaluate_player_hand};

impl TexasAdapter {
    fn spectator_equities(&self, recipient_index: usize) -> Option<Vec<TexasEquity>> {
        if !self.spectator.available(&self.game, recipient_index)
            || !self.spectator.requested(recipient_index)
        {
            return None;
        }
        Some(
            self.equity_cache
                .values(&self.game)
                .map_or_else(Vec::new, |values| {
                    self.players
                        .iter()
                        .zip(values)
                        .map(|(player, basis_points)| TexasEquity {
                            player: player.id,
                            basis_points,
                        })
                        .collect()
                }),
        )
    }

    pub fn snapshot(&self, recipient: PlayerId) -> Result<TexasSnapshot, AdapterError> {
        let index = self.player_index(recipient)?;
        Ok(self.snapshot_for(recipient, Some(index)))
    }

    pub(in super::super) fn spectator_snapshot(&self, recipient: PlayerId) -> TexasSnapshot {
        self.snapshot_for(recipient, None)
    }

    fn snapshot_for(&self, recipient: PlayerId, recipient_index: Option<usize>) -> TexasSnapshot {
        let recipient_core = recipient_index.map(PlayerId::from_index);
        let revealed_hands = self.revealed_hands();
        let players = self
            .players
            .iter()
            .zip(self.game.players())
            .enumerate()
            .map(|(index, (participant, state))| TexasPlayerView {
                player: PlayerPublicInfo {
                    id: participant.id,
                    profile_id: participant.profile_id,
                    name: participant.name.clone(),
                    avatar: participant.avatar,
                    reference_points: participant.reference_points,
                    completed_games: participant.completed_games,
                    game_profiles: Default::default(),
                },
                seat: participant.seat,
                stack: state.stack(),
                hand_start_stack: self.hand_starting_stacks[index],
                committed_street: state.committed_street(),
                committed_total: state.committed_total(),
                folded: state.folded(),
                all_in: state.all_in(),
                connected: participant.connected,
                auto_play: false,
                ready: false,
            })
            .collect();
        let phase = match self.game.phase() {
            TexasPhase::Betting(street) => TexasPhaseView::Betting { street: *street },
            TexasPhase::ChoosingRunout => TexasPhaseView::ChoosingRunout,
            TexasPhase::Complete(result) => TexasPhaseView::HandComplete {
                showdown: result.showdown,
                awards: result
                    .awards
                    .iter()
                    .map(|award| (1, award))
                    .chain(
                        result
                            .second_run
                            .iter()
                            .flat_map(|run| run.awards.iter().map(|award| (2, award))),
                    )
                    .map(|(run, award)| TexasPotAwardView {
                        run,
                        amount: award.amount,
                        winners: award
                            .winners
                            .iter()
                            .map(|winner| self.protocol_player(*winner))
                            .collect(),
                        winning_category: award.winning_hand.map(|hand| hand.category()),
                    })
                    .collect(),
                table_winner: self
                    .game
                    .table_winner()
                    .map(|winner| self.protocol_player(winner)),
                tournament_complete: self.game.players().iter().any(|player| player.stack() == 0),
                reference_changes: Vec::new(),
            },
        };
        TexasSnapshot {
            match_id: self.match_id,
            hand_number: self.game.hand_number(),
            host_port: self.host_port,
            you: recipient,
            host: self.host,
            players,
            your_hole_cards: recipient_index
                .map(|index| self.game.players()[index].hole_cards().to_vec())
                .unwrap_or_default(),
            revealed_hands,
            community: self.game.community().to_vec(),
            second_community: match self.game.phase() {
                TexasPhase::Complete(result) if result.showdown => result
                    .second_run
                    .as_ref()
                    .map(|run| run.community.clone())
                    .unwrap_or_default(),
                _ => Vec::new(),
            },
            runout_votes: self
                .game
                .runout_votes()
                .iter()
                .map(|(player, twice)| TexasRunoutVoteView {
                    player: self.protocol_player(*player),
                    twice: *twice,
                })
                .collect(),
            spectator_available: recipient_index
                .is_some_and(|index| self.spectator.available(&self.game, index)),
            spectator_win_rates_enabled: recipient_index
                .is_some_and(|index| self.spectator.requested(index)),
            spectator_equities: recipient_index.and_then(|index| self.spectator_equities(index)),
            draw_pile_len: self.game.draw_pile_len() as u16,
            dealer: self.protocol_player(self.game.dealer()),
            small_blind: self.protocol_player(self.game.small_blind()),
            big_blind: self.protocol_player(self.game.big_blind()),
            current_player: self
                .game
                .current_player()
                .map(|player| self.protocol_player(player)),
            blind_to_post: self
                .game
                .blind_to_post()
                .map(|(player, kind, amount)| TexasBlindView {
                    player: self.protocol_player(player),
                    kind,
                    amount,
                }),
            current_bet: self.game.current_bet(),
            minimum_raise_to: self.game.minimum_raise_to(),
            maximum_raise_to: recipient_core
                .and_then(|player| self.game.raise_sizing(player).ok())
                .and_then(|sizing| sizing.maximum()),
            amount_to_call: recipient_core
                .and_then(|player| self.game.amount_to_call(player).ok())
                .unwrap_or(0),
            raise_allowed: recipient_core
                .is_some_and(|player| self.game.raise_allowed(player).unwrap_or(false)),
            all_in_allowed: recipient_core
                .is_some_and(|player| self.game.all_in_allowed(player).unwrap_or(false)),
            pot: self.game.pot(),
            phase,
            squid: TexasSquidView {
                round: self.game.squid().round,
                markers: self
                    .game
                    .squid()
                    .markers
                    .iter()
                    .map(|owner| owner.map(|id| self.protocol_player(id)))
                    .collect(),
                settled: self.game.squid().settled,
                transfers: self
                    .game
                    .squid()
                    .transfers
                    .iter()
                    .map(|transfer| TexasSquidTransfer {
                        from: self.protocol_player(transfer.from),
                        to: self.protocol_player(transfer.to),
                        amount: transfer.amount,
                    })
                    .collect(),
            },
        }
    }

    fn revealed_hands(&self) -> Vec<TexasRevealedHand> {
        let TexasPhase::Complete(result) = self.game.phase() else {
            return Vec::new();
        };
        // 无争议收池时任何玩家都不亮底牌，包括最后留下的赢家。
        if !result.showdown {
            return Vec::new();
        }
        self.game
            .players()
            .iter()
            .filter(|player| {
                !player.folded() && player.hole_cards().len() == self.game.rules().hole_card_count()
            })
            .map(|player| {
                let cards = player.hole_cards().to_vec();
                TexasRevealedHand {
                    player: self.protocol_player(player.id()),
                    cards,
                    second_best: result.second_run.as_ref().and_then(|run| {
                        evaluate_player_hand(player.hole_cards(), &run.community, self.game.rules())
                            .ok()
                    }),
                    best: (self.game.community().len() >= 3)
                        .then(|| {
                            evaluate_player_hand(
                                player.hole_cards(),
                                self.game.community(),
                                self.game.rules(),
                            )
                            .ok()
                        })
                        .flatten(),
                }
            })
            .collect()
    }
}
