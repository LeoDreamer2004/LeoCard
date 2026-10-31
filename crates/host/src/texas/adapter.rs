use super::equity::EquityCache;
use super::spectator::SpectatorAccess;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    AvatarId, MatchId, PlayerPublicInfo, ProfileId, SeatId, TexasBlindView, TexasEquity,
    TexasEvent, TexasPhaseView, TexasPlayerView, TexasPotAwardView, TexasRevealedHand,
    TexasSnapshot, TexasSquidView, TexasViolation,
};
use leocard_texas_holdem::{
    TexasAction, TexasActionOutcome, TexasCard, TexasGameError, TexasGameState, TexasPhase,
    TexasRuleError, TexasRuleSet, TexasSquidTransfer, evaluate_player_hand,
};

use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TablePlayer {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub connected: bool,
    pub reference_points: i32,
    pub completed_games: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidRules(TexasRuleError),
    PlayerCount(usize),
    DuplicatePlayer(PlayerId),
    DuplicateSeat(SeatId),
    HostNotAtTable(PlayerId),
    RecipientNotAtTable(PlayerId),
    Game(TexasGameError),
    Violation(TexasViolation),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::PlayerCount(actual) => {
                write!(f, "德州扑克需要 3..=8 名玩家，实际为 {actual}")
            }
            Self::DuplicatePlayer(player) => write!(f, "玩家 ID {:?} 重复", player),
            Self::DuplicateSeat(seat) => write!(f, "座位 {:?} 重复", seat),
            Self::HostNotAtTable(player) => write!(f, "房主 {:?} 不在牌桌中", player),
            Self::RecipientNotAtTable(player) => {
                write!(f, "快照接收者 {:?} 不在牌桌中", player)
            }
            Self::Game(error) => error.fmt(f),
            Self::Violation(violation) => write!(f, "德州扑克动作被拒绝：{violation:?}"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<TexasRuleError> for AdapterError {
    fn from(value: TexasRuleError) -> Self {
        Self::InvalidRules(value)
    }
}

#[derive(Clone, Debug)]
pub struct TexasAdapter {
    match_id: MatchId,
    host_port: u16,
    host: PlayerId,
    players: Vec<TablePlayer>,
    game: TexasGameState,
    hand_starting_stacks: Vec<u32>,
    equity_cache: EquityCache,
    spectator: SpectatorAccess,
}

impl TexasAdapter {
    /// `players` 会按座位编号排序，排序后的顺序就是核心中的顺时针顺序。
    /// `deck[0]` 是第一张发出的牌。
    pub fn new(
        match_id: MatchId,
        host_port: u16,
        host: PlayerId,
        mut players: Vec<TablePlayer>,
        rules: TexasRuleSet,
        first_dealer: PlayerId,
        deck: Vec<TexasCard>,
    ) -> Result<Self, AdapterError> {
        if !(TexasRuleSet::MIN_PLAYERS as usize..=TexasRuleSet::MAX_PLAYERS as usize)
            .contains(&players.len())
        {
            return Err(AdapterError::PlayerCount(players.len()));
        }
        validate_unique_players(&players)?;
        players.sort_by_key(|player| player.seat.0);
        if !players.iter().any(|player| player.id == host) {
            return Err(AdapterError::HostNotAtTable(host));
        }
        let dealer = players
            .iter()
            .position(|player| player.id == first_dealer)
            .map(PlayerId::from_index)
            .ok_or(AdapterError::RecipientNotAtTable(first_dealer))?;
        let effective_rules = TexasRuleSet {
            player_count: players.len() as u8,
            squid_markers: rules.squid_markers.min(players.len() as u8 - 1),
            ..rules
        }
        .validate()?;
        let game = TexasGameState::new_with_squid_seed(effective_rules, dealer, deck, match_id.0)
            .map_err(AdapterError::Game)?;
        let hand_starting_stacks = game.players().iter().map(|player| player.stack()).collect();
        let spectator = SpectatorAccess::new(effective_rules.player_count);
        Ok(Self {
            match_id,
            host_port,
            host,
            players,
            game,
            spectator,
            hand_starting_stacks,
            equity_cache: EquityCache::default(),
        })
    }

    pub const fn rules(&self) -> &TexasRuleSet {
        self.game.rules()
    }

    pub const fn match_id(&self) -> MatchId {
        self.match_id
    }

    pub fn players(&self) -> &[TablePlayer] {
        &self.players
    }

    pub const fn game(&self) -> &TexasGameState {
        &self.game
    }

    pub fn current_player(&self) -> Option<PlayerId> {
        self.game
            .current_player()
            .map(|player| self.protocol_player(player))
    }

    pub fn table_winner(&self) -> Option<PlayerId> {
        self.game
            .table_winner()
            .map(|player| self.protocol_player(player))
    }

    pub fn set_connected(&mut self, player: PlayerId, connected: bool) -> Result<(), AdapterError> {
        let participant = self
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .ok_or(AdapterError::RecipientNotAtTable(player))?;
        participant.connected = connected;
        Ok(())
    }

    pub fn act(
        &mut self,
        player: PlayerId,
        action: TexasAction,
    ) -> Result<Vec<TexasEvent>, AdapterError> {
        let core_player = self.core_player(player)?;
        let community_before = self.game.community().len();
        let committed_before = self.game.players()[core_player.index()].committed_total();
        let outcome = self
            .game
            .act(core_player, action)
            .map_err(map_action_error)?;
        if matches!(action, TexasAction::Fold) {
            self.spectator
                .record_fold(core_player.index(), community_before);
        }
        if self.game.community().len() != community_before || matches!(action, TexasAction::Fold) {
            self.equity_cache.clear();
        }
        let amount = self.game.players()[core_player.index()]
            .committed_total()
            .saturating_sub(committed_before);
        let mut events = vec![TexasEvent::ActionApplied {
            player,
            action,
            amount,
            statistics: self
                .game
                .last_action_statistics()
                .expect("an accepted action has statistics"),
        }];
        match outcome {
            TexasActionOutcome::BlindPosted { .. } => {}
            TexasActionOutcome::Acted { .. } => {}
            TexasActionOutcome::StreetAdvanced { street, .. } => {
                events.push(TexasEvent::StreetAdvanced {
                    street,
                    dealt: self.game.community()[community_before..].to_vec(),
                });
            }
            TexasActionOutcome::HandComplete(result) => {
                if let Some((player, statistics)) = self.game.squid().award_statistics() {
                    events.push(TexasEvent::SquidAwarded {
                        player: self.protocol_player(player),
                        statistics,
                    });
                }
                events.extend(self.players.iter().enumerate().map(|(index, participant)| {
                    TexasEvent::HandAnalyzed {
                        player: participant.id,
                        statistics: Box::new(
                            self.game
                                .hand_statistics(PlayerId::from_index(index))
                                .expect("a settled hand has statistics"),
                        ),
                    }
                }));
                events.push(TexasEvent::HandFinished {
                    showdown: result.showdown,
                });
            }
        }
        Ok(events)
    }

    /// 保留当前筹码并顺时针移动庄家按钮，开始下一手牌。
    pub fn start_next_hand(&mut self, deck: Vec<TexasCard>) -> Result<(), AdapterError> {
        let starting_stacks = self
            .game
            .players()
            .iter()
            .map(|player| player.stack())
            .collect();
        self.game
            .start_next_hand(deck)
            .map_err(AdapterError::Game)?;
        self.hand_starting_stacks = starting_stacks;
        self.spectator.start_hand();
        self.equity_cache.clear();
        Ok(())
    }

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

    pub(super) fn poll_spectator_equities(&self) -> bool {
        if !matches!(self.game.phase(), TexasPhase::Betting(_)) {
            self.equity_cache.clear();
            return false;
        }
        self.equity_cache.take_ready()
    }

    pub fn set_spectator_win_rates(
        &mut self,
        player: PlayerId,
        enabled: bool,
    ) -> Result<(), AdapterError> {
        let index = self.player_index(player)?;
        self.spectator.set_requested(index, enabled);
        Ok(())
    }

    pub fn snapshot(&self, recipient: PlayerId) -> Result<TexasSnapshot, AdapterError> {
        let index = self.player_index(recipient)?;
        Ok(self.snapshot_for(recipient, Some(index)))
    }

    pub(super) fn spectator_snapshot(&self, recipient: PlayerId) -> TexasSnapshot {
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
            TexasPhase::Complete(result) => TexasPhaseView::HandComplete {
                showdown: result.showdown,
                awards: result
                    .awards
                    .iter()
                    .map(|award| TexasPotAwardView {
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

    fn player_index(&self, player: PlayerId) -> Result<usize, AdapterError> {
        self.players
            .iter()
            .position(|participant| participant.id == player)
            .ok_or(AdapterError::RecipientNotAtTable(player))
    }

    fn core_player(&self, player: PlayerId) -> Result<PlayerId, AdapterError> {
        self.player_index(player).map(PlayerId::from_index)
    }

    fn protocol_player(&self, player: PlayerId) -> PlayerId {
        self.players[player.index()].id
    }
}

fn validate_unique_players(players: &[TablePlayer]) -> Result<(), AdapterError> {
    let mut ids = HashSet::with_capacity(players.len());
    let mut seats = HashSet::with_capacity(players.len());
    for player in players {
        if !ids.insert(player.id) {
            return Err(AdapterError::DuplicatePlayer(player.id));
        }
        if !seats.insert(player.seat) {
            return Err(AdapterError::DuplicateSeat(player.seat));
        }
    }
    Ok(())
}

fn map_action_error(error: TexasGameError) -> AdapterError {
    let violation = match error {
        TexasGameError::InvalidPlayer(_) => TexasViolation::InvalidPlayer,
        TexasGameError::NotPlayersTurn { .. } => TexasViolation::NotPlayersTurn,
        TexasGameError::HandAlreadyComplete => TexasViolation::HandAlreadyComplete,
        TexasGameError::PlayerCannotAct(_) => TexasViolation::PlayerCannotAct,
        TexasGameError::MustPostBlind => TexasViolation::MustPostBlind,
        TexasGameError::NoBlindToPost => TexasViolation::NoBlindToPost,
        TexasGameError::CannotCheckWhileFacingBet { amount_to_call } => {
            TexasViolation::CannotCheckWhileFacingBet { amount_to_call }
        }
        TexasGameError::NothingToCall => TexasViolation::NothingToCall,
        TexasGameError::RaiseMustExceedCurrentBet {
            current_bet,
            target,
        } => TexasViolation::RaiseMustExceedCurrentBet {
            current_bet,
            target,
        },
        TexasGameError::RaiseBelowMinimum {
            minimum_target,
            target,
        } => TexasViolation::RaiseBelowMinimum {
            minimum_target,
            target,
        },
        TexasGameError::RaiseExceedsStack {
            maximum_target,
            target,
        } => TexasViolation::RaiseExceedsStack {
            maximum_target,
            target,
        },
        TexasGameError::RaiseNotReopened => TexasViolation::RaiseNotReopened,
        TexasGameError::RaiseExceedsLimit {
            maximum_target,
            target,
        } => TexasViolation::RaiseExceedsLimit {
            maximum_target,
            target,
        },
        TexasGameError::RaiseLimitReached => TexasViolation::RaiseLimitReached,
        other => return AdapterError::Game(other),
    };
    AdapterError::Violation(violation)
}
