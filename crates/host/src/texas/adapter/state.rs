use super::super::equity::EquityCache;
use super::super::spectator::SpectatorAccess;
use leocard_game_common::PlayerId;
use leocard_protocol::{AvatarId, MatchId, ProfileId, SeatId, TexasViolation};
use leocard_texas_holdem::{
    TexasCard, TexasGameError, TexasGameState, TexasPhase, TexasRuleError, TexasRuleSet,
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
    pub(super) match_id: MatchId,
    pub(super) host_port: u16,
    pub(super) host: PlayerId,
    pub(super) players: Vec<TablePlayer>,
    pub(super) game: TexasGameState,
    pub(super) hand_starting_stacks: Vec<u32>,
    pub(super) equity_cache: EquityCache,
    pub(super) spectator: SpectatorAccess,
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

    pub(in super::super) fn poll_spectator_equities(&self) -> bool {
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

    pub(super) fn player_index(&self, player: PlayerId) -> Result<usize, AdapterError> {
        self.players
            .iter()
            .position(|participant| participant.id == player)
            .ok_or(AdapterError::RecipientNotAtTable(player))
    }

    pub(super) fn core_player(&self, player: PlayerId) -> Result<PlayerId, AdapterError> {
        self.player_index(player).map(PlayerId::from_index)
    }

    pub(super) fn protocol_player(&self, player: PlayerId) -> PlayerId {
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

pub(super) fn map_action_error(error: TexasGameError) -> AdapterError {
    let violation = match error {
        TexasGameError::InvalidPlayer(_) => TexasViolation::InvalidPlayer,
        TexasGameError::NotPlayersTurn { .. } => TexasViolation::NotPlayersTurn,
        TexasGameError::HandAlreadyComplete => TexasViolation::HandAlreadyComplete,
        TexasGameError::PlayerCannotAct(_) => TexasViolation::PlayerCannotAct,
        TexasGameError::MustPostBlind => TexasViolation::MustPostBlind,
        TexasGameError::NotChoosingRunout => TexasViolation::NotChoosingRunout,
        TexasGameError::MustChooseRunout => TexasViolation::MustChooseRunout,
        TexasGameError::RunoutAlreadyChosen(_) => TexasViolation::RunoutAlreadyChosen,
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
