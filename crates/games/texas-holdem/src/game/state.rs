use super::TexasBettingRecord;
use crate::statistics::GameStatistics;
use crate::{
    TexasCard, TexasEvaluatedHand, TexasHandError, TexasRuleError, TexasRuleSet, TexasSquidState,
    build_deck,
};
use leocard_game_common::{DeckError, PlayerId, validate_deck as validate_physical_deck};
use std::{collections::VecDeque, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasStreet {
    PreFlop,
    Flop,
    Turn,
    River,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasBlindKind {
    Small,
    Big,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasAction {
    PostBlind,
    Fold,
    Check,
    Call,
    /// 加注到本轮累计投入的目标值，而不是额外增加多少。
    RaiseTo(u32),
    AllIn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TexasPlayerState {
    pub(super) id: PlayerId,
    pub(super) stack: u32,
    pub(super) hole_cards: Vec<TexasCard>,
    pub(super) folded: bool,
    pub(super) all_in: bool,
    pub(super) committed_street: u32,
    pub(super) committed_total: u32,
}

impl TexasPlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub const fn stack(&self) -> u32 {
        self.stack
    }

    pub fn hole_cards(&self) -> &[TexasCard] {
        &self.hole_cards
    }

    pub const fn folded(&self) -> bool {
        self.folded
    }

    pub const fn all_in(&self) -> bool {
        self.all_in
    }

    pub const fn committed_street(&self) -> u32 {
        self.committed_street
    }

    pub const fn committed_total(&self) -> u32 {
        self.committed_total
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TexasPotAward {
    pub amount: u32,
    pub winners: Vec<PlayerId>,
    /// 无人跟注、其他玩家全部弃牌时不计算牌型。
    pub winning_hand: Option<TexasEvaluatedHand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TexasHandResult {
    pub dealer: PlayerId,
    pub showdown: bool,
    pub community: Vec<TexasCard>,
    pub awards: Vec<TexasPotAward>,
    pub final_stacks: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TexasPhase {
    Betting(TexasStreet),
    Complete(TexasHandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TexasActionOutcome {
    BlindPosted {
        player: PlayerId,
        kind: TexasBlindKind,
        amount: u32,
        next_player: Option<PlayerId>,
    },
    Acted {
        player: PlayerId,
        action: TexasAction,
        next_player: PlayerId,
    },
    StreetAdvanced {
        street: TexasStreet,
        current_player: PlayerId,
        community_cards: usize,
    },
    HandComplete(TexasHandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TexasGameError {
    InvalidRules(TexasRuleError),
    InvalidDealer(PlayerId),
    InvalidStacks {
        expected: usize,
        actual: usize,
    },
    NotEnoughFundedPlayers,
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(PlayerId),
    NotPlayersTurn {
        expected: PlayerId,
        actual: PlayerId,
    },
    HandAlreadyComplete,
    HandStillInProgress,
    CannotCheckWhileFacingBet {
        amount_to_call: u32,
    },
    NothingToCall,
    RaiseMustExceedCurrentBet {
        current_bet: u32,
        target: u32,
    },
    RaiseBelowMinimum {
        minimum_target: u32,
        target: u32,
    },
    RaiseExceedsStack {
        maximum_target: u32,
        target: u32,
    },
    RaiseInvalidIncrement {
        increment: u32,
        target: u32,
    },
    RaiseNotReopened,
    RaiseExceedsLimit {
        maximum_target: u32,
        target: u32,
    },
    RaiseLimitReached,
    PlayerCannotAct(PlayerId),
    MustPostBlind,
    NoBlindToPost,
    HandEvaluation(TexasHandError),
}

impl fmt::Display for TexasGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDealer(player) => write!(f, "庄家玩家 {:?} 不存在", player),
            Self::InvalidStacks { expected, actual } => {
                write!(f, "筹码数量应对应 {expected} 名玩家，实际为 {actual}")
            }
            Self::NotEnoughFundedPlayers => f.write_str("至少需要两名仍有筹码的玩家"),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或包含当前模式禁牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}；当前应由 {:?} 操作", actual, expected)
            }
            Self::HandAlreadyComplete => f.write_str("本手牌已经结束"),
            Self::HandStillInProgress => f.write_str("当前手牌尚未结束"),
            Self::CannotCheckWhileFacingBet { amount_to_call } => {
                write!(f, "仍需跟注 {amount_to_call}，不能过牌")
            }
            Self::NothingToCall => f.write_str("当前无需跟注，应选择过牌"),
            Self::RaiseMustExceedCurrentBet {
                current_bet,
                target,
            } => write!(f, "加注目标 {target} 必须大于当前下注 {current_bet}"),
            Self::RaiseBelowMinimum {
                minimum_target,
                target,
            } => write!(f, "最低应加注到 {minimum_target}，实际目标为 {target}"),
            Self::RaiseExceedsStack {
                maximum_target,
                target,
            } => write!(f, "最多只能下注到 {maximum_target}，实际目标为 {target}"),
            Self::RaiseInvalidIncrement { increment, target } => {
                write!(
                    f,
                    "普通加注增量必须为 {increment} 的倍数，实际目标为 {target}"
                )
            }
            Self::RaiseNotReopened => f.write_str("不足额全下没有重新开放加注权"),
            Self::RaiseExceedsLimit {
                maximum_target,
                target,
            } => {
                write!(
                    f,
                    "固定限注本次最多加注到 {maximum_target}，实际目标为 {target}"
                )
            }
            Self::RaiseLimitReached => f.write_str("本街已达到一次下注加三次加注的上限"),
            Self::PlayerCannotAct(player) => write!(f, "玩家 {:?} 已弃牌、全下或出局", player),
            Self::MustPostBlind => f.write_str("当前只能下盲注"),
            Self::NoBlindToPost => f.write_str("当前没有待下的盲注"),
            Self::HandEvaluation(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for TexasGameError {}

impl From<DeckError> for TexasGameError {
    fn from(value: DeckError) -> Self {
        match value {
            DeckError::InvalidSize { expected, actual } => {
                Self::InvalidDeckSize { expected, actual }
            }
            DeckError::InvalidContents => Self::InvalidDeckContents,
        }
    }
}

impl From<TexasRuleError> for TexasGameError {
    fn from(value: TexasRuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<TexasHandError> for TexasGameError {
    fn from(value: TexasHandError) -> Self {
        Self::HandEvaluation(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TexasGameState {
    pub(super) betting_history: Vec<TexasBettingRecord>,
    pub(super) rules: TexasRuleSet,
    pub(super) players: Vec<TexasPlayerState>,
    pub(super) dealer: PlayerId,
    pub(super) small_blind: PlayerId,
    pub(super) big_blind: PlayerId,
    pub(super) deck: VecDeque<TexasCard>,
    pub(super) community: Vec<TexasCard>,
    pub(super) current_player: Option<PlayerId>,
    pub(super) current_bet: u32,
    pub(super) minimum_raise: u32,
    pub(super) full_bets: u8,
    pub(super) needs_action: Vec<bool>,
    pub(super) raise_allowed: Vec<bool>,
    pub(super) phase: TexasPhase,
    pub(super) pending_blind: Option<TexasBlindKind>,
    pub(super) hand_number: u32,
    pub(crate) statistics: GameStatistics,
    pub(super) squid: TexasSquidState,
}

impl TexasGameState {
    pub const fn squid(&self) -> &TexasSquidState {
        &self.squid
    }

    /// 创建一桌新游戏；`deck[0]` 是本手最先发出的牌。
    pub fn new_with_deck(
        rules: TexasRuleSet,
        first_dealer: PlayerId,
        deck: Vec<TexasCard>,
    ) -> Result<Self, TexasGameError> {
        let rules = rules.validate()?;
        let stacks = vec![u32::from(rules.starting_chips); usize::from(rules.player_count)];
        Self::new_with_stacks(rules, first_dealer, stacks, deck)
    }

    /// Use the public match identity for marker appearances and weighted squid payouts.
    pub fn new_with_squid_seed(
        rules: TexasRuleSet,
        first_dealer: PlayerId,
        deck: Vec<TexasCard>,
        seed: [u8; 16],
    ) -> Result<Self, TexasGameError> {
        let mut game = Self::new_with_deck(rules, first_dealer, deck)?;
        game.squid = TexasSquidState::new(rules.squid_markers, seed, rules.deepseek_squid_double);
        Ok(game)
    }

    /// 用已有筹码开始一手牌，供连续牌局和确定性的边池测试使用。
    pub fn new_with_stacks(
        rules: TexasRuleSet,
        dealer: PlayerId,
        stacks: Vec<u32>,
        deck: Vec<TexasCard>,
    ) -> Result<Self, TexasGameError> {
        let rules = rules.validate()?;
        let player_count = usize::from(rules.player_count);
        if dealer.index() >= player_count {
            return Err(TexasGameError::InvalidDealer(dealer));
        }
        if stacks.len() != player_count {
            return Err(TexasGameError::InvalidStacks {
                expected: player_count,
                actual: stacks.len(),
            });
        }
        validate_deck(rules.short_deck, &deck)?;
        if stacks.iter().filter(|stack| **stack > 0).count() < 2 {
            return Err(TexasGameError::NotEnoughFundedPlayers);
        }
        let players = stacks
            .into_iter()
            .enumerate()
            .map(|(index, stack)| TexasPlayerState {
                id: PlayerId::from_index(index),
                stack,
                hole_cards: Vec::with_capacity(rules.hole_card_count()),
                folded: stack == 0,
                all_in: stack == 0,
                committed_street: 0,
                committed_total: 0,
            })
            .collect::<Vec<_>>();
        let mut state = Self {
            squid: TexasSquidState::new(rules.squid_markers, [0; 16], rules.deepseek_squid_double),
            betting_history: Vec::new(),
            rules,
            players,
            dealer,
            small_blind: dealer,
            big_blind: dealer,
            deck: VecDeque::new(),
            community: Vec::with_capacity(5),
            current_player: None,
            current_bet: 0,
            minimum_raise: TexasRuleSet::BIG_BLIND,
            full_bets: 0,
            needs_action: vec![false; player_count],
            raise_allowed: vec![false; player_count],
            phase: TexasPhase::Betting(TexasStreet::PreFlop),
            pending_blind: None,
            hand_number: 0,
            statistics: GameStatistics::new(rules.player_count),
        };
        state.begin_hand(dealer, deck)?;
        Ok(state)
    }

    pub const fn rules(&self) -> &TexasRuleSet {
        &self.rules
    }

    pub fn betting_history(&self) -> &[TexasBettingRecord] {
        &self.betting_history
    }

    pub fn players(&self) -> &[TexasPlayerState] {
        &self.players
    }

    pub const fn dealer(&self) -> PlayerId {
        self.dealer
    }

    pub const fn small_blind(&self) -> PlayerId {
        self.small_blind
    }

    pub const fn big_blind(&self) -> PlayerId {
        self.big_blind
    }

    pub const fn current_player(&self) -> Option<PlayerId> {
        self.current_player
    }

    pub const fn current_bet(&self) -> u32 {
        self.current_bet
    }

    /// 无限注为最后一次完整加注幅度；固定限注为当前街的固定额度。
    pub const fn minimum_raise(&self) -> u32 {
        self.minimum_raise
    }

    /// 当前玩家完成一次合法完整加注时，最低需要把本街累计下注提高到的数值。
    pub const fn minimum_raise_to(&self) -> u32 {
        self.current_bet + self.minimum_raise
    }

    /// 玩家当前是否仍有加注权。是否轮到该玩家行动需另行检查
    /// [`TexasGameState::current_player`]。
    pub fn raise_allowed(&self, player: PlayerId) -> Result<bool, TexasGameError> {
        self.players
            .get(player.index())
            .ok_or(TexasGameError::InvalidPlayer(player))?;
        Ok(self.raise_allowed[player.index()]
            && self.can_act(player)
            && !self.fixed_limit_reached())
    }

    pub fn community(&self) -> &[TexasCard] {
        &self.community
    }

    pub fn draw_pile_len(&self) -> usize {
        self.deck.len()
    }

    pub const fn phase(&self) -> &TexasPhase {
        &self.phase
    }

    pub fn blind_to_post(&self) -> Option<(PlayerId, TexasBlindKind, u32)> {
        let kind = self.pending_blind?;
        let player = self.current_player?;
        let requested = match kind {
            TexasBlindKind::Small => TexasRuleSet::SMALL_BLIND,
            TexasBlindKind::Big => TexasRuleSet::BIG_BLIND,
        };
        Some((
            player,
            kind,
            requested.min(self.players[player.index()].stack),
        ))
    }

    pub const fn hand_number(&self) -> u32 {
        self.hand_number
    }

    pub fn pot(&self) -> u32 {
        self.players
            .iter()
            .map(TexasPlayerState::committed_total)
            .sum()
    }

    pub fn amount_to_call(&self, player: PlayerId) -> Result<u32, TexasGameError> {
        let state = self
            .players
            .get(player.index())
            .ok_or(TexasGameError::InvalidPlayer(player))?;
        Ok(self.current_bet.saturating_sub(state.committed_street))
    }
}

pub(super) fn validate_deck(short_deck: bool, deck: &[TexasCard]) -> Result<(), TexasGameError> {
    validate_physical_deck(deck, &build_deck(short_deck)).map_err(TexasGameError::from)
}
