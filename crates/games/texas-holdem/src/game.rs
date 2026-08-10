use std::collections::{HashSet, VecDeque};
use std::fmt;

use crate::{Card, EvaluatedHand, HandError, RuleError, RuleSet, build_deck, evaluate_best};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerId(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Street {
    PreFlop,
    Flop,
    Turn,
    River,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BlindKind {
    Small,
    Big,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
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
    id: PlayerId,
    stack: u32,
    hole_cards: Vec<Card>,
    folded: bool,
    all_in: bool,
    committed_street: u32,
    committed_total: u32,
}

impl PlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub const fn stack(&self) -> u32 {
        self.stack
    }

    pub fn hole_cards(&self) -> &[Card] {
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
    pub winners: Vec<PlayerId>,
    /// 无人跟注、其他玩家全部弃牌时不计算牌型。
    pub winning_hand: Option<EvaluatedHand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandResult {
    pub dealer: PlayerId,
    pub showdown: bool,
    pub community: Vec<Card>,
    pub awards: Vec<PotAward>,
    pub final_stacks: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Betting(Street),
    Complete(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    BlindPosted {
        player: PlayerId,
        kind: BlindKind,
        amount: u32,
        next_player: Option<PlayerId>,
    },
    Acted {
        player: PlayerId,
        action: Action,
        next_player: PlayerId,
    },
    StreetAdvanced {
        street: Street,
        current_player: PlayerId,
        community_cards: usize,
    },
    HandComplete(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
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
    RaiseNotReopened,
    PlayerCannotAct(PlayerId),
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
    rules: RuleSet,
    players: Vec<PlayerState>,
    dealer: PlayerId,
    small_blind: PlayerId,
    big_blind: PlayerId,
    deck: VecDeque<Card>,
    community: Vec<Card>,
    current_player: Option<PlayerId>,
    current_bet: u32,
    minimum_raise: u32,
    needs_action: Vec<bool>,
    raise_allowed: Vec<bool>,
    phase: Phase,
    pending_blind: Option<BlindKind>,
    hand_number: u32,
}

impl GameState {
    /// 创建一桌新游戏；`deck[0]` 是本手最先发出的牌。
    pub fn new_with_deck(
        rules: RuleSet,
        first_dealer: PlayerId,
        deck: Vec<Card>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let stacks = vec![u32::from(rules.starting_chips); usize::from(rules.player_count)];
        Self::new_with_stacks(rules, first_dealer, stacks, deck)
    }

    /// 用已有筹码开始一手牌，供连续牌局和确定性的边池测试使用。
    pub fn new_with_stacks(
        rules: RuleSet,
        dealer: PlayerId,
        stacks: Vec<u32>,
        deck: Vec<Card>,
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
                id: PlayerId(index),
                stack,
                hole_cards: Vec::with_capacity(2),
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
            minimum_raise: RuleSet::BIG_BLIND,
            needs_action: vec![false; player_count],
            raise_allowed: vec![false; player_count],
            phase: Phase::Betting(Street::PreFlop),
            pending_blind: None,
            hand_number: 0,
        };
        state.begin_hand(dealer, deck)?;
        Ok(state)
    }

    pub const fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
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
    pub fn raise_allowed(&self, player: PlayerId) -> Result<bool, GameError> {
        self.players
            .get(player.0)
            .ok_or(GameError::InvalidPlayer(player))?;
        Ok(self.raise_allowed[player.0] && self.can_act(player))
    }

    pub fn community(&self) -> &[Card] {
        &self.community
    }

    pub fn draw_pile_len(&self) -> usize {
        self.deck.len()
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn blind_to_post(&self) -> Option<(PlayerId, BlindKind, u32)> {
        let kind = self.pending_blind?;
        let player = self.current_player?;
        let requested = match kind {
            BlindKind::Small => RuleSet::SMALL_BLIND,
            BlindKind::Big => RuleSet::BIG_BLIND,
        };
        Some((player, kind, requested.min(self.players[player.0].stack)))
    }

    pub const fn hand_number(&self) -> u32 {
        self.hand_number
    }

    pub fn pot(&self) -> u32 {
        self.players.iter().map(PlayerState::committed_total).sum()
    }

    pub fn amount_to_call(&self, player: PlayerId) -> Result<u32, GameError> {
        let state = self
            .players
            .get(player.0)
            .ok_or(GameError::InvalidPlayer(player))?;
        Ok(self.current_bet.saturating_sub(state.committed_street))
    }

    pub fn act(&mut self, player: PlayerId, action: Action) -> Result<ActionOutcome, GameError> {
        let previous = self.clone();
        match self.act_inner(player, action) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                *self = previous;
                Err(error)
            }
        }
    }

    fn act_inner(&mut self, player: PlayerId, action: Action) -> Result<ActionOutcome, GameError> {
        if matches!(self.phase, Phase::Complete(_)) {
            return Err(GameError::HandAlreadyComplete);
        }
        let expected = self
            .current_player
            .expect("betting phase has a current player");
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if let Some(kind) = self.pending_blind {
            if action != Action::PostBlind {
                return Err(GameError::MustPostBlind);
            }
            let requested = match kind {
                BlindKind::Small => RuleSet::SMALL_BLIND,
                BlindKind::Big => RuleSet::BIG_BLIND,
            };
            let amount = requested.min(self.players[player.0].stack);
            self.commit(player, amount);
            if kind == BlindKind::Small {
                self.pending_blind = Some(BlindKind::Big);
                self.current_player = Some(self.big_blind);
                return Ok(ActionOutcome::BlindPosted {
                    player,
                    kind,
                    amount,
                    next_player: self.current_player,
                });
            }

            self.pending_blind = None;
            self.current_bet = self
                .players
                .iter()
                .map(PlayerState::committed_street)
                .max()
                .unwrap_or(0);
            self.needs_action.fill(false);
            self.raise_allowed.fill(false);
            for index in 0..self.players.len() {
                if self.can_act(PlayerId(index)) {
                    self.needs_action[index] = true;
                    self.raise_allowed[index] = true;
                }
            }
            self.current_player = self.next_needing_action(self.big_blind);
            if self.current_player.is_none() {
                self.run_out_and_showdown()?;
                let Phase::Complete(result) = &self.phase else {
                    unreachable!("无人可行动时应自动摊牌")
                };
                return Ok(ActionOutcome::HandComplete(result.clone()));
            }
            return Ok(ActionOutcome::BlindPosted {
                player,
                kind,
                amount,
                next_player: self.current_player,
            });
        }
        if action == Action::PostBlind {
            return Err(GameError::NoBlindToPost);
        }
        if !self.can_act(player) {
            return Err(GameError::PlayerCannotAct(player));
        }

        let old_street = self.street();
        let old_community = self.community.len();
        let amount_to_call = self.amount_to_call(player)?;
        self.needs_action[player.0] = false;
        let mut full_raise = false;

        match action {
            Action::PostBlind => unreachable!("盲注动作已在常规下注前处理"),
            Action::Fold => {
                self.players[player.0].folded = true;
                self.raise_allowed[player.0] = false;
            }
            Action::Check => {
                if amount_to_call > 0 {
                    return Err(GameError::CannotCheckWhileFacingBet { amount_to_call });
                }
                // Check 并没有回应任何下注。若后手玩家随后用不足最低下注额的
                // all-in 首次下注，当前玩家仍有权进行一次完整加注。
            }
            Action::Call => {
                if amount_to_call == 0 {
                    return Err(GameError::NothingToCall);
                }
                let payment = amount_to_call.min(self.players[player.0].stack);
                self.commit(player, payment);
                self.raise_allowed[player.0] = false;
            }
            Action::RaiseTo(target) => {
                if !self.raise_allowed[player.0] {
                    return Err(GameError::RaiseNotReopened);
                }
                if target <= self.current_bet {
                    return Err(GameError::RaiseMustExceedCurrentBet {
                        current_bet: self.current_bet,
                        target,
                    });
                }
                let maximum_target =
                    self.players[player.0].committed_street + self.players[player.0].stack;
                if target > maximum_target {
                    return Err(GameError::RaiseExceedsStack {
                        maximum_target,
                        target,
                    });
                }
                let minimum_target = self.current_bet + self.minimum_raise;
                if target < minimum_target {
                    return Err(GameError::RaiseBelowMinimum {
                        minimum_target,
                        target,
                    });
                }
                let payment = target - self.players[player.0].committed_street;
                self.commit(player, payment);
                self.minimum_raise = target - self.current_bet;
                self.current_bet = target;
                full_raise = true;
                // 主动加注已经用掉本次行动权；只有其他玩家后续的完整加注才能
                // 再次开放。否则，紧随其后的不足额 all-in 会错误地允许原加注者
                // 再次加注。
                self.raise_allowed[player.0] = false;
            }
            Action::AllIn => {
                let target = self.players[player.0].committed_street + self.players[player.0].stack;
                if target > self.current_bet && !self.raise_allowed[player.0] {
                    return Err(GameError::RaiseNotReopened);
                }
                let payment = self.players[player.0].stack;
                self.commit(player, payment);
                if target > self.current_bet {
                    let raise_size = target - self.current_bet;
                    full_raise = raise_size >= self.minimum_raise;
                    if full_raise {
                        self.minimum_raise = raise_size;
                    }
                    self.current_bet = target;
                }
                self.raise_allowed[player.0] = false;
            }
        }

        if self.contenders().len() == 1 {
            let result = self.settle_uncontested();
            return Ok(ActionOutcome::HandComplete(result));
        }

        if full_raise {
            for index in 0..self.players.len() {
                if index != player.0 && self.can_act(PlayerId(index)) {
                    self.needs_action[index] = true;
                    self.raise_allowed[index] = true;
                }
            }
        } else {
            for index in 0..self.players.len() {
                if index != player.0
                    && self.can_act(PlayerId(index))
                    && self.players[index].committed_street < self.current_bet
                {
                    self.needs_action[index] = true;
                }
            }
        }

        if let Some(next) = self.next_needing_action(player) {
            self.current_player = Some(next);
            return Ok(ActionOutcome::Acted {
                player,
                action,
                next_player: next,
            });
        }

        self.finish_betting_round()?;
        if let Phase::Complete(result) = &self.phase {
            return Ok(ActionOutcome::HandComplete(result.clone()));
        }
        let current_player = self
            .current_player
            .expect("new street has an acting player");
        Ok(ActionOutcome::StreetAdvanced {
            street: self.street(),
            current_player,
            community_cards: self.community.len() - old_community,
        })
        .inspect(|_| debug_assert_ne!(self.street(), old_street))
    }

    /// 结束后使用一副新洗好的牌开始下一手；庄家按钮顺时针移动到下一名有筹码玩家。
    pub fn start_next_hand(&mut self, deck: Vec<Card>) -> Result<(), GameError> {
        if !matches!(self.phase, Phase::Complete(_)) {
            return Err(GameError::HandStillInProgress);
        }
        validate_deck(self.rules.short_deck, &deck)?;
        if self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .count()
            < 2
        {
            return Err(GameError::NotEnoughFundedPlayers);
        }
        let dealer = self
            .next_funded(self.dealer)
            .expect("at least two funded players remain");
        self.hand_number = self.hand_number.saturating_add(1);
        self.begin_hand(dealer, deck)
    }

    pub fn table_winner(&self) -> Option<PlayerId> {
        let funded = self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        (funded.len() == 1).then_some(funded[0])
    }

    fn begin_hand(&mut self, dealer: PlayerId, deck: Vec<Card>) -> Result<(), GameError> {
        self.dealer = dealer;
        self.deck = deck.into();
        self.community.clear();
        self.current_bet = 0;
        self.minimum_raise = RuleSet::BIG_BLIND;
        self.phase = Phase::Betting(Street::PreFlop);
        self.pending_blind = Some(BlindKind::Small);
        for player in &mut self.players {
            player.hole_cards.clear();
            player.folded = player.stack == 0;
            player.all_in = player.stack == 0;
            player.committed_street = 0;
            player.committed_total = 0;
        }

        let funded = self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .count();
        if funded < 2 {
            return Err(GameError::NotEnoughFundedPlayers);
        }
        if self.players[dealer.0].stack == 0 {
            return Err(GameError::InvalidDealer(dealer));
        }
        self.small_blind = if funded == 2 {
            dealer
        } else {
            self.next_funded(dealer).expect("another funded player")
        };
        self.big_blind = self
            .next_funded(self.small_blind)
            .expect("another funded player");

        let first_dealt = self.next_funded(dealer).expect("another funded player");
        let mut dealt_to = first_dealt;
        for _ in 0..2 {
            loop {
                let card = self
                    .deck
                    .pop_front()
                    .expect("validated deck is large enough");
                self.players[dealt_to.0].hole_cards.push(card);
                dealt_to = self.next_funded(dealt_to).expect("funded player ring");
                if dealt_to == first_dealt {
                    break;
                }
            }
        }

        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        self.current_player = Some(self.small_blind);
        Ok(())
    }

    fn commit(&mut self, player: PlayerId, amount: u32) {
        let state = &mut self.players[player.0];
        debug_assert!(amount <= state.stack);
        state.stack -= amount;
        state.committed_street += amount;
        state.committed_total += amount;
        state.all_in = state.stack == 0;
        if state.all_in {
            self.needs_action[player.0] = false;
        }
    }

    fn finish_betting_round(&mut self) -> Result<(), GameError> {
        if self.actionable_players().len() <= 1 {
            return self.run_out_and_showdown();
        }
        let next_street = match self.street() {
            Street::PreFlop => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => return self.settle_showdown().map(|_| ()),
        };
        self.deal_community_for(next_street);
        self.phase = Phase::Betting(next_street);
        self.current_bet = 0;
        self.minimum_raise = RuleSet::BIG_BLIND;
        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        for player in &mut self.players {
            player.committed_street = 0;
        }
        for index in 0..self.players.len() {
            if self.can_act(PlayerId(index)) {
                self.needs_action[index] = true;
                self.raise_allowed[index] = true;
            }
        }
        self.current_player = self.next_needing_action(self.dealer);
        Ok(())
    }

    fn run_out_and_showdown(&mut self) -> Result<(), GameError> {
        while self.community.len() < 5 {
            let street = match self.community.len() {
                0 => Street::Flop,
                3 => Street::Turn,
                4 => Street::River,
                _ => unreachable!("community is dealt as 0, 3, 4, 5"),
            };
            self.deal_community_for(street);
        }
        self.settle_showdown().map(|_| ())
    }

    fn deal_community_for(&mut self, street: Street) {
        let count = if street == Street::Flop { 3 } else { 1 };
        for _ in 0..count {
            self.community.push(
                self.deck
                    .pop_front()
                    .expect("validated deck is large enough"),
            );
        }
    }

    fn settle_uncontested(&mut self) -> HandResult {
        let winner = self.contenders()[0];
        let amount = self.pot();
        self.players[winner.0].stack += amount;
        let result = HandResult {
            dealer: self.dealer,
            showdown: false,
            community: self.community.clone(),
            awards: vec![PotAward {
                amount,
                winners: vec![winner],
                winning_hand: None,
            }],
            final_stacks: self.players.iter().map(PlayerState::stack).collect(),
        };
        self.current_player = None;
        self.phase = Phase::Complete(result.clone());
        result
    }

    fn settle_showdown(&mut self) -> Result<HandResult, GameError> {
        let contenders = self.contenders();
        let mut evaluated = vec![None; self.players.len()];
        for player in &contenders {
            let mut cards = self.players[player.0].hole_cards.clone();
            cards.extend_from_slice(&self.community);
            evaluated[player.0] = Some(evaluate_best(&cards, &self.rules)?);
        }

        let mut levels = self
            .players
            .iter()
            .map(PlayerState::committed_total)
            .filter(|amount| *amount > 0)
            .collect::<Vec<_>>();
        levels.sort_unstable();
        levels.dedup();
        let mut previous = 0;
        let mut awards = Vec::new();
        for level in levels {
            let participants = self
                .players
                .iter()
                .filter(|player| player.committed_total >= level)
                .count() as u32;
            let amount = (level - previous) * participants;
            previous = level;
            if amount == 0 {
                continue;
            }
            let eligible = contenders
                .iter()
                .copied()
                .filter(|player| self.players[player.0].committed_total >= level)
                .collect::<Vec<_>>();
            let best = eligible
                .iter()
                .filter_map(|player| evaluated[player.0])
                .max()
                .expect("each pot has an eligible contender");
            let mut winners = eligible
                .into_iter()
                .filter(|player| evaluated[player.0] == Some(best))
                .collect::<Vec<_>>();
            winners.sort_by_key(|winner| {
                let distance = self.clockwise_distance(self.dealer, *winner);
                if distance == 0 {
                    self.players.len()
                } else {
                    distance
                }
            });
            let share = amount / winners.len() as u32;
            let remainder = amount % winners.len() as u32;
            for (index, winner) in winners.iter().enumerate() {
                self.players[winner.0].stack += share + u32::from((index as u32) < remainder);
            }
            awards.push(PotAward {
                amount,
                winners,
                winning_hand: Some(best),
            });
        }
        let result = HandResult {
            dealer: self.dealer,
            showdown: true,
            community: self.community.clone(),
            awards,
            final_stacks: self.players.iter().map(PlayerState::stack).collect(),
        };
        self.current_player = None;
        self.phase = Phase::Complete(result.clone());
        Ok(result)
    }

    fn street(&self) -> Street {
        match self.phase {
            Phase::Betting(street) => street,
            Phase::Complete(_) => Street::River,
        }
    }

    fn contenders(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|player| !player.folded)
            .map(|player| player.id)
            .collect()
    }

    fn actionable_players(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|player| !player.folded && !player.all_in)
            .map(|player| player.id)
            .collect()
    }

    fn can_act(&self, player: PlayerId) -> bool {
        self.players
            .get(player.0)
            .is_some_and(|player| !player.folded && !player.all_in)
    }

    fn next_needing_action(&self, after: PlayerId) -> Option<PlayerId> {
        (1..=self.players.len())
            .map(|offset| PlayerId((after.0 + offset) % self.players.len()))
            .find(|player| self.needs_action[player.0] && self.can_act(*player))
    }

    fn next_funded(&self, after: PlayerId) -> Option<PlayerId> {
        (1..=self.players.len())
            .map(|offset| PlayerId((after.0 + offset) % self.players.len()))
            .find(|player| self.players[player.0].stack > 0)
    }

    fn clockwise_distance(&self, from: PlayerId, to: PlayerId) -> usize {
        (to.0 + self.players.len() - from.0) % self.players.len()
    }
}

fn validate_deck(short_deck: bool, deck: &[Card]) -> Result<(), GameError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rank, Suit};

    fn ordered_deck(prefix: &[Card], short_deck: bool) -> Vec<Card> {
        let prefix_set = prefix.iter().copied().collect::<HashSet<_>>();
        let mut deck = prefix.to_vec();
        deck.extend(
            build_deck(short_deck)
                .into_iter()
                .filter(|card| !prefix_set.contains(card)),
        );
        deck
    }

    fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(suit, rank)
    }

    fn post_blinds(state: &mut GameState) {
        let small = state.current_player().expect("小盲应先行动");
        state.act(small, Action::PostBlind).unwrap();
        let big = state.current_player().expect("大盲应随后行动");
        state.act(big, Action::PostBlind).unwrap();
    }

    #[test]
    fn dealer_blinds_and_preflop_action_follow_clockwise_order() {
        let mut state = GameState::new_with_deck(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            build_deck(false),
        )
        .unwrap();
        assert_eq!(state.small_blind(), PlayerId(1));
        assert_eq!(state.big_blind(), PlayerId(2));
        assert_eq!(state.current_player(), Some(PlayerId(1)));
        assert_eq!(state.players()[0].stack(), 20);
        assert_eq!(state.players()[1].stack(), 20);
        assert_eq!(state.players()[2].stack(), 20);
        assert_eq!(
            state.blind_to_post(),
            Some((PlayerId(1), BlindKind::Small, 1))
        );
        assert!(matches!(
            state.act(PlayerId(1), Action::Call),
            Err(GameError::MustPostBlind)
        ));
        state.act(PlayerId(1), Action::PostBlind).unwrap();
        assert_eq!(
            state.blind_to_post(),
            Some((PlayerId(2), BlindKind::Big, 2))
        );
        state.act(PlayerId(2), Action::PostBlind).unwrap();
        assert_eq!(state.current_player(), Some(PlayerId(0)));
        assert_eq!(state.players()[1].stack(), 19);
        assert_eq!(state.players()[2].stack(), 18);
        assert!(
            state
                .players()
                .iter()
                .all(|player| player.hole_cards().len() == 2)
        );
    }

    #[test]
    fn four_betting_rounds_deal_exactly_five_community_cards() {
        let mut state = GameState::new_with_deck(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);
        state.act(PlayerId(0), Action::Call).unwrap();
        state.act(PlayerId(1), Action::Call).unwrap();
        let flop = state.act(PlayerId(2), Action::Check).unwrap();
        assert!(matches!(
            flop,
            ActionOutcome::StreetAdvanced {
                street: Street::Flop,
                community_cards: 3,
                ..
            }
        ));
        assert_eq!(state.current_player(), Some(PlayerId(1)));

        for (street, expected_cards) in [(Street::Turn, 4), (Street::River, 5)] {
            state.act(PlayerId(1), Action::Check).unwrap();
            state.act(PlayerId(2), Action::Check).unwrap();
            let outcome = state.act(PlayerId(0), Action::Check).unwrap();
            assert!(matches!(
                outcome,
                ActionOutcome::StreetAdvanced { street: actual, .. } if actual == street
            ));
            assert_eq!(state.community().len(), expected_cards);
        }
        state.act(PlayerId(1), Action::Check).unwrap();
        state.act(PlayerId(2), Action::Check).unwrap();
        assert!(matches!(
            state.act(PlayerId(0), Action::Check).unwrap(),
            ActionOutcome::HandComplete(_)
        ));
    }

    #[test]
    fn all_in_side_pots_are_awarded_independently() {
        use Suit::{Club, Diamond, Heart, Spade};
        let prefix = [
            c(Rank::King, Spade),
            c(Rank::Queen, Spade),
            c(Rank::Ace, Spade),
            c(Rank::King, Heart),
            c(Rank::Queen, Heart),
            c(Rank::Ace, Heart),
            c(Rank::Two, Club),
            c(Rank::Three, Diamond),
            c(Rank::Seven, Spade),
            c(Rank::Eight, Club),
            c(Rank::Nine, Diamond),
        ];
        let mut state = GameState::new_with_stacks(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            vec![5, 10, 20],
            ordered_deck(&prefix, false),
        )
        .unwrap();
        post_blinds(&mut state);
        state.act(PlayerId(0), Action::AllIn).unwrap();
        state.act(PlayerId(1), Action::AllIn).unwrap();
        let result = match state.act(PlayerId(2), Action::Call).unwrap() {
            ActionOutcome::HandComplete(result) => result,
            other => panic!("expected showdown, got {other:?}"),
        };
        assert_eq!(result.awards.len(), 2);
        assert_eq!(result.awards[0].amount, 15);
        assert_eq!(result.awards[0].winners, vec![PlayerId(0)]);
        assert_eq!(result.awards[1].amount, 10);
        assert_eq!(result.awards[1].winners, vec![PlayerId(1)]);
        assert_eq!(result.final_stacks, vec![15, 10, 10]);
    }

    #[test]
    fn several_distinct_all_ins_create_independently_eligible_side_pots() {
        use Suit::{Club, Diamond, Heart, Spade};
        // 每个较短筹码玩家都拿到比后续玩家更大的口袋对子，因此能够验证：
        // 他只参与不超过自己投入额的底池，不能赢走更深层的边池。
        let prefix = [
            c(Rank::King, Spade),
            c(Rank::Queen, Spade),
            c(Rank::Jack, Spade),
            c(Rank::Ace, Spade),
            c(Rank::King, Heart),
            c(Rank::Queen, Heart),
            c(Rank::Jack, Heart),
            c(Rank::Ace, Heart),
            c(Rank::Two, Club),
            c(Rank::Three, Diamond),
            c(Rank::Seven, Spade),
            c(Rank::Eight, Club),
            c(Rank::Nine, Diamond),
        ];
        let mut state = GameState::new_with_stacks(
            RuleSet {
                player_count: 4,
                ..RuleSet::default()
            },
            PlayerId(0),
            vec![5, 10, 15, 20],
            ordered_deck(&prefix, false),
        )
        .unwrap();
        post_blinds(&mut state);
        state.act(PlayerId(3), Action::AllIn).unwrap();
        state.act(PlayerId(0), Action::AllIn).unwrap();
        state.act(PlayerId(1), Action::AllIn).unwrap();
        let result = match state.act(PlayerId(2), Action::AllIn).unwrap() {
            ActionOutcome::HandComplete(result) => result,
            other => panic!("expected showdown, got {other:?}"),
        };

        assert_eq!(
            result
                .awards
                .iter()
                .map(|award| (award.amount, award.winners.clone()))
                .collect::<Vec<_>>(),
            vec![
                (20, vec![PlayerId(0)]),
                (15, vec![PlayerId(1)]),
                (10, vec![PlayerId(2)]),
                (5, vec![PlayerId(3)]),
            ]
        );
        assert_eq!(result.final_stacks, vec![20, 15, 10, 5]);
    }

    #[test]
    fn folding_everyone_else_ends_the_hand_without_showdown() {
        let mut state = GameState::new_with_deck(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);
        state.act(PlayerId(0), Action::Fold).unwrap();
        let result = match state.act(PlayerId(1), Action::Fold).unwrap() {
            ActionOutcome::HandComplete(result) => result,
            other => panic!("expected immediate win, got {other:?}"),
        };
        assert!(!result.showdown);
        assert_eq!(result.awards[0].winners, vec![PlayerId(2)]);
        assert_eq!(result.final_stacks.iter().sum::<u32>(), 60);
    }

    #[test]
    fn rejected_action_never_mutates_the_authoritative_state() {
        let mut state = GameState::new_with_deck(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);
        let before = state.clone();
        assert!(matches!(
            state.act(PlayerId(0), Action::Check),
            Err(GameError::CannotCheckWhileFacingBet { .. })
        ));
        assert_eq!(state, before);
    }

    #[test]
    fn full_raise_reopens_action_and_enforces_the_minimum_increment() {
        let mut state = GameState::new_with_deck(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);
        assert!(matches!(
            state.act(PlayerId(0), Action::RaiseTo(3)),
            Err(GameError::RaiseBelowMinimum {
                minimum_target: 4,
                ..
            })
        ));
        state.act(PlayerId(0), Action::Call).unwrap();
        state.act(PlayerId(1), Action::RaiseTo(4)).unwrap();
        state.act(PlayerId(2), Action::Call).unwrap();
        assert_eq!(state.current_player(), Some(PlayerId(0)));
        assert!(matches!(
            state.act(PlayerId(0), Action::Call).unwrap(),
            ActionOutcome::StreetAdvanced {
                street: Street::Flop,
                ..
            }
        ));
    }

    #[test]
    fn checking_keeps_raise_right_against_a_short_all_in_opening_bet() {
        let mut state = GameState::new_with_stacks(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            vec![20, 20, 3],
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);

        state.act(PlayerId(0), Action::Call).unwrap();
        state.act(PlayerId(1), Action::Call).unwrap();
        state.act(PlayerId(2), Action::Check).unwrap();

        // 翻牌圈最低完整下注为 2；玩家 2 只剩 1，因此这是不足额的首次下注。
        state.act(PlayerId(1), Action::Check).unwrap();
        state.act(PlayerId(2), Action::AllIn).unwrap();
        state.act(PlayerId(0), Action::Call).unwrap();
        assert_eq!(state.current_player(), Some(PlayerId(1)));

        // 玩家 1 之前只是 check，并未面对下注，仍可完成一次加注到 3。
        assert!(state.act(PlayerId(1), Action::RaiseTo(3)).is_ok());
    }

    #[test]
    fn short_all_in_does_not_reopen_the_original_raisers_action() {
        let mut state = GameState::new_with_stacks(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            vec![20, 20, 5],
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);

        state.act(PlayerId(0), Action::Call).unwrap();
        state.act(PlayerId(1), Action::Call).unwrap();
        state.act(PlayerId(2), Action::Check).unwrap();

        state.act(PlayerId(1), Action::RaiseTo(2)).unwrap();
        // 玩家 2 只剩 3：加到 3 的增量小于最低完整加注幅度 2。
        state.act(PlayerId(2), Action::AllIn).unwrap();
        state.act(PlayerId(0), Action::Call).unwrap();
        assert_eq!(state.current_player(), Some(PlayerId(1)));
        assert!(matches!(
            state.act(PlayerId(1), Action::RaiseTo(5)),
            Err(GameError::RaiseNotReopened)
        ));
    }

    #[test]
    fn dealer_button_moves_to_the_next_funded_player() {
        let mut state = GameState::new_with_deck(
            RuleSet {
                player_count: 3,
                ..RuleSet::default()
            },
            PlayerId(0),
            build_deck(false),
        )
        .unwrap();
        post_blinds(&mut state);
        state.act(PlayerId(0), Action::Fold).unwrap();
        state.act(PlayerId(1), Action::Fold).unwrap();
        state.start_next_hand(build_deck(false)).unwrap();
        assert_eq!(state.dealer(), PlayerId(1));
        assert_eq!(state.hand_number(), 1);
    }
}
