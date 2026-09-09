mod actions;
mod lifecycle;
#[cfg(test)]
mod tests;

use crate::{
    EvaluatedHand, HandError, RuleError, TexasHoldemCard, TexasHoldemRuleSet, build_deck,
    evaluate_player_hand,
};
use std::collections::{HashSet, VecDeque};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasHoldemPlayerId(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasHoldemStreet {
    PreFlop,
    Flop,
    Turn,
    River,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasHoldemBlindKind {
    Small,
    Big,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasHoldemAction {
    PostBlind,
    Fold,
    Check,
    Call,
    /// 加注到本轮累计投入的目标值，而不是额外增加多少。
    RaiseTo(u32),
    AllIn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    id: TexasHoldemPlayerId,
    stack: u32,
    hole_cards: Vec<TexasHoldemCard>,
    folded: bool,
    all_in: bool,
    committed_street: u32,
    committed_total: u32,
}

impl PlayerState {
    pub const fn id(&self) -> TexasHoldemPlayerId {
        self.id
    }

    pub const fn stack(&self) -> u32 {
        self.stack
    }

    pub fn hole_cards(&self) -> &[TexasHoldemCard] {
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
pub struct PotAward {
    pub amount: u32,
    pub winners: Vec<TexasHoldemPlayerId>,
    /// 无人跟注、其他玩家全部弃牌时不计算牌型。
    pub winning_hand: Option<EvaluatedHand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandResult {
    pub dealer: TexasHoldemPlayerId,
    pub showdown: bool,
    pub community: Vec<TexasHoldemCard>,
    pub awards: Vec<PotAward>,
    pub final_stacks: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Betting(TexasHoldemStreet),
    Complete(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    BlindPosted {
        player: TexasHoldemPlayerId,
        kind: TexasHoldemBlindKind,
        amount: u32,
        next_player: Option<TexasHoldemPlayerId>,
    },
    Acted {
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
        next_player: TexasHoldemPlayerId,
    },
    StreetAdvanced {
        street: TexasHoldemStreet,
        current_player: TexasHoldemPlayerId,
        community_cards: usize,
    },
    HandComplete(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
    InvalidDealer(TexasHoldemPlayerId),
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
    InvalidPlayer(TexasHoldemPlayerId),
    NotPlayersTurn {
        expected: TexasHoldemPlayerId,
        actual: TexasHoldemPlayerId,
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
    RaiseNotReopened,
    PlayerCannotAct(TexasHoldemPlayerId),
    MustPostBlind,
    NoBlindToPost,
    HandEvaluation(HandError),
}

impl fmt::Display for GameError {
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
            Self::RaiseNotReopened => f.write_str("不足额全下没有重新开放加注权"),
            Self::PlayerCannotAct(player) => write!(f, "玩家 {:?} 已弃牌、全下或出局", player),
            Self::MustPostBlind => f.write_str("当前只能下盲注"),
            Self::NoBlindToPost => f.write_str("当前没有待下的盲注"),
            Self::HandEvaluation(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<HandError> for GameError {
    fn from(value: HandError) -> Self {
        Self::HandEvaluation(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: TexasHoldemRuleSet,
    players: Vec<PlayerState>,
    dealer: TexasHoldemPlayerId,
    small_blind: TexasHoldemPlayerId,
    big_blind: TexasHoldemPlayerId,
    deck: VecDeque<TexasHoldemCard>,
    community: Vec<TexasHoldemCard>,
    current_player: Option<TexasHoldemPlayerId>,
    current_bet: u32,
    minimum_raise: u32,
    needs_action: Vec<bool>,
    raise_allowed: Vec<bool>,
    phase: Phase,
    pending_blind: Option<TexasHoldemBlindKind>,
    hand_number: u32,
}

impl GameState {
    /// 创建一桌新游戏；`deck[0]` 是本手最先发出的牌。
    pub fn new_with_deck(
        rules: TexasHoldemRuleSet,
        first_dealer: TexasHoldemPlayerId,
        deck: Vec<TexasHoldemCard>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let stacks = vec![u32::from(rules.starting_chips); usize::from(rules.player_count)];
        Self::new_with_stacks(rules, first_dealer, stacks, deck)
    }

    /// 用已有筹码开始一手牌，供连续牌局和确定性的边池测试使用。
    pub fn new_with_stacks(
        rules: TexasHoldemRuleSet,
        dealer: TexasHoldemPlayerId,
        stacks: Vec<u32>,
        deck: Vec<TexasHoldemCard>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let player_count = usize::from(rules.player_count);
        if dealer.0 >= player_count {
            return Err(GameError::InvalidDealer(dealer));
        }
        if stacks.len() != player_count {
            return Err(GameError::InvalidStacks {
                expected: player_count,
                actual: stacks.len(),
            });
        }
        validate_deck(rules.short_deck, &deck)?;
        if stacks.iter().filter(|stack| **stack > 0).count() < 2 {
            return Err(GameError::NotEnoughFundedPlayers);
        }
        let players = stacks
            .into_iter()
            .enumerate()
            .map(|(index, stack)| PlayerState {
                id: TexasHoldemPlayerId(index),
                stack,
                hole_cards: Vec::with_capacity(rules.hole_card_count()),
                folded: stack == 0,
                all_in: stack == 0,
                committed_street: 0,
                committed_total: 0,
            })
            .collect::<Vec<_>>();
        let mut state = Self {
            rules,
            players,
            dealer,
            small_blind: dealer,
            big_blind: dealer,
            deck: VecDeque::new(),
            community: Vec::with_capacity(5),
            current_player: None,
            current_bet: 0,
            minimum_raise: TexasHoldemRuleSet::BIG_BLIND,
            needs_action: vec![false; player_count],
            raise_allowed: vec![false; player_count],
            phase: Phase::Betting(TexasHoldemStreet::PreFlop),
            pending_blind: None,
            hand_number: 0,
        };
        state.begin_hand(dealer, deck)?;
        Ok(state)
    }

    pub const fn rules(&self) -> &TexasHoldemRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub const fn dealer(&self) -> TexasHoldemPlayerId {
        self.dealer
    }

    pub const fn small_blind(&self) -> TexasHoldemPlayerId {
        self.small_blind
    }

    pub const fn big_blind(&self) -> TexasHoldemPlayerId {
        self.big_blind
    }

    pub const fn current_player(&self) -> Option<TexasHoldemPlayerId> {
        self.current_player
    }

    pub const fn current_bet(&self) -> u32 {
        self.current_bet
    }

    /// 当前最后一次完整加注的幅度；新下注街开始时等于大盲注。
    pub const fn minimum_raise(&self) -> u32 {
        self.minimum_raise
    }

    /// 当前玩家完成一次合法完整加注时，最低需要把本街累计下注提高到的数值。
    pub const fn minimum_raise_to(&self) -> u32 {
        self.current_bet + self.minimum_raise
    }

    /// 玩家当前是否仍有加注权。是否轮到该玩家行动需另行检查
    /// [`GameState::current_player`]。
    pub fn raise_allowed(&self, player: TexasHoldemPlayerId) -> Result<bool, GameError> {
        self.players
            .get(player.0)
            .ok_or(GameError::InvalidPlayer(player))?;
        Ok(self.raise_allowed[player.0] && self.can_act(player))
    }

    pub fn community(&self) -> &[TexasHoldemCard] {
        &self.community
    }

    pub fn draw_pile_len(&self) -> usize {
        self.deck.len()
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn blind_to_post(&self) -> Option<(TexasHoldemPlayerId, TexasHoldemBlindKind, u32)> {
        let kind = self.pending_blind?;
        let player = self.current_player?;
        let requested = match kind {
            TexasHoldemBlindKind::Small => TexasHoldemRuleSet::SMALL_BLIND,
            TexasHoldemBlindKind::Big => TexasHoldemRuleSet::BIG_BLIND,
        };
        Some((player, kind, requested.min(self.players[player.0].stack)))
    }

    pub const fn hand_number(&self) -> u32 {
        self.hand_number
    }

    pub fn pot(&self) -> u32 {
        self.players.iter().map(PlayerState::committed_total).sum()
    }

    pub fn amount_to_call(&self, player: TexasHoldemPlayerId) -> Result<u32, GameError> {
        let state = self
            .players
            .get(player.0)
            .ok_or(GameError::InvalidPlayer(player))?;
        Ok(self.current_bet.saturating_sub(state.committed_street))
    }
}

fn validate_deck(short_deck: bool, deck: &[TexasHoldemCard]) -> Result<(), GameError> {
    let expected = build_deck(short_deck);
    if deck.len() != expected.len() {
        return Err(GameError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}
