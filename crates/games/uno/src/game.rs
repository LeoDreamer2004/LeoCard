use std::collections::{HashSet, VecDeque};
use std::fmt;

use crate::rating::{placements_with_eliminations, reference_point_deltas_for_placements};
use crate::{Card, Color, Face, FlipSide, RuleError, RuleSet, build_deck_for_rules};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    id: PlayerId,
    hand: Vec<Card>,
    eliminated: bool,
}

impl PlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[Card] {
        &self.hand
    }

    pub fn hand_score(&self) -> u16 {
        self.hand.iter().map(|card| card.score()).sum()
    }

    pub const fn eliminated(&self) -> bool {
        self.eliminated
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

impl Direction {
    const fn reversed(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PendingDrawKind {
    DrawTwo,
    WildDrawFour,
    Stack,
    NoMercy(u8),
    FlipDrawOne,
    FlipWildDrawTwo,
    FlipDrawFive,
    FlipWildDrawColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TurnState {
    pub current_player: PlayerId,
    pub direction: Direction,
    pub current_color: Option<Color>,
    pub top_card: Card,
    pub pending_draw: u16,
    pub pending_kind: Option<PendingDrawKind>,
    pub pending_draw_source: Option<PlayerId>,
    pub challenge_offender: Option<PlayerId>,
    pub drawn_card: Option<Card>,
    pub pending_skip: u16,
    pub skipped_turns_remaining: u16,
    pub pending_swap: Option<PendingSwap>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingSwap {
    SwapOneTarget { player: PlayerId },
    SwapOneGive { player: PlayerId, target: PlayerId },
    ForceTrade { player: PlayerId },
    ChooseColor { player: PlayerId },
    SevenSwap { player: PlayerId },
    ColorRoulette { player: PlayerId },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingSwapState {
    SwapOneTarget {
        player: PlayerId,
        declared_uno: bool,
    },
    SwapOneGive {
        player: PlayerId,
        target: PlayerId,
        declared_uno: bool,
    },
    ForceTrade {
        player: PlayerId,
    },
    ChooseColor {
        player: PlayerId,
    },
    SevenSwap {
        player: PlayerId,
    },
    ColorRoulette {
        player: PlayerId,
    },
}

impl PendingSwapState {
    const fn public(self) -> PendingSwap {
        match self {
            Self::SwapOneTarget { player, .. } => PendingSwap::SwapOneTarget { player },
            Self::SwapOneGive { player, target, .. } => PendingSwap::SwapOneGive { player, target },
            Self::ForceTrade { player } => PendingSwap::ForceTrade { player },
            Self::ChooseColor { player } => PendingSwap::ChooseColor { player },
            Self::SevenSwap { player } => PendingSwap::SevenSwap { player },
            Self::ColorRoulette { player } => PendingSwap::ColorRoulette { player },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayedEffect {
    HandRefreshed { count: u16 },
    HandsPassed { direction: Direction },
    DrawReflected { player: PlayerId, cards: Vec<Card> },
    StackNumberRevealed { cards: Vec<Card>, value: u8 },
    CardsDiscarded { cards: Vec<Card> },
    Flipped { side: FlipSide },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameResult {
    pub winner: PlayerId,
    pub hand_scores: Vec<u16>,
    pub placements: Vec<u8>,
    pub reference_deltas: Vec<i16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Playing,
    Finished(GameResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChallengeResult {
    Successful,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    ColorChosen {
        player: PlayerId,
        color: Color,
    },
    Played {
        player: PlayerId,
        card: Card,
        next_player: PlayerId,
        effect: Option<PlayedEffect>,
    },
    SwapOneCardTaken {
        player: PlayerId,
        target: PlayerId,
    },
    SwapOneCompleted {
        player: PlayerId,
        target: PlayerId,
        next_player: PlayerId,
    },
    HandsTraded {
        player: PlayerId,
        first: PlayerId,
        second: PlayerId,
    },
    DrewCards {
        player: PlayerId,
        cards: Vec<Card>,
        playable: Option<Card>,
        next_player: PlayerId,
    },
    PassedAfterDraw {
        player: PlayerId,
        next_player: PlayerId,
    },
    PenaltyDrawn {
        player: PlayerId,
        cards: Vec<Card>,
        next_player: PlayerId,
    },
    ChallengeResolved {
        challenger: PlayerId,
        offender: PlayerId,
        result: ChallengeResult,
        penalized: PlayerId,
        cards: Vec<Card>,
        next_player: PlayerId,
    },
    UnoCalled {
        player: PlayerId,
    },
    UnoReported {
        reporter: PlayerId,
        target: PlayerId,
        cards: Vec<Card>,
    },
    SkipResolved {
        player: PlayerId,
        cards: Vec<Card>,
        remaining: u16,
        next_player: PlayerId,
    },
    ColorRouletteResolved {
        player: PlayerId,
        color: Color,
        cards: Vec<Card>,
        next_player: PlayerId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(PlayerId),
    PlayerEliminated(PlayerId),
    NotPlayersTurn {
        expected: PlayerId,
        actual: PlayerId,
    },
    GameAlreadyFinished,
    InitialColorChoiceRequired,
    InitialColorAlreadyChosen,
    CardNotInHand(Card),
    CardDoesNotMatch,
    ColorRequired,
    UnexpectedColor,
    MustPlayDrawnCard(Card),
    MustResolveDrawPenalty,
    NoDrawPenalty,
    CannotStack(Card),
    CannotChallenge,
    MustDrawBeforePassing,
    MustResolveSkip,
    NoSkipToResolve,
    UnoCalloutDisabled,
    CannotCallUno(PlayerId),
    MustPlayAfterUno,
    CannotReportSelf,
    PlayerNotReportable(PlayerId),
    CannotPlayTogether,
    CannotJumpIn,
    MustResolveSwapEffect,
    NoSwapEffect,
    InvalidSwapTargets,
    DrawPileExhausted,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::PlayerEliminated(player) => write!(f, "玩家 {:?} 已被淘汰", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}；当前应由 {:?} 操作", actual, expected)
            }
            Self::GameAlreadyFinished => f.write_str("本局已经结束"),
            Self::InitialColorChoiceRequired => f.write_str("先手必须先为万能牌选择颜色"),
            Self::InitialColorAlreadyChosen => f.write_str("当前不需要选择初始颜色"),
            Self::CardNotInHand(card) => write!(f, "玩家手中没有这张牌：{card}"),
            Self::CardDoesNotMatch => f.write_str("这张牌与当前颜色、数字或符号不匹配"),
            Self::ColorRequired => f.write_str("万能牌必须指定后续颜色"),
            Self::UnexpectedColor => f.write_str("非万能牌不能指定后续颜色"),
            Self::MustPlayDrawnCard(card) => write!(f, "摸牌后只能打出刚摸到的牌：{card}"),
            Self::MustResolveDrawPenalty => f.write_str("必须叠加罚牌、质疑或接受累计罚牌"),
            Self::NoDrawPenalty => f.write_str("当前没有待结算的罚牌"),
            Self::CannotStack(card) => write!(f, "当前不能叠加这张牌：{card}"),
            Self::CannotChallenge => f.write_str("当前没有可质疑的万能摸四"),
            Self::MustDrawBeforePassing => f.write_str("只有摸到可出的牌后才能选择结束回合"),
            Self::MustResolveSkip => f.write_str("必须叠加禁手牌或接受禁手"),
            Self::NoSkipToResolve => f.write_str("当前没有待结算的禁手"),
            Self::UnoCalloutDisabled => f.write_str("本房间未启用 UNO 宣告和检举"),
            Self::CannotCallUno(player) => write!(f, "玩家 {:?} 当前不能宣告 UNO", player),
            Self::MustPlayAfterUno => f.write_str("宣告 UNO 后必须打出倒数第二张牌"),
            Self::CannotReportSelf => f.write_str("不能检举自己"),
            Self::PlayerNotReportable(player) => write!(f, "玩家 {:?} 当前不可被检举", player),
            Self::CannotPlayTogether => f.write_str("这些牌当前不能一次打出"),
            Self::CannotJumpIn => f.write_str("当前不能抢出这张牌"),
            Self::MustResolveSwapEffect => f.write_str("必须先完成当前换牌效果"),
            Self::NoSwapEffect => f.write_str("当前没有待结算的换牌效果"),
            Self::InvalidSwapTargets => f.write_str("换牌目标无效"),
            Self::DrawPileExhausted => f.write_str("没有可继续摸取的牌"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ChallengeState {
    offender: PlayerId,
    was_legal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: RuleSet,
    players: Vec<PlayerState>,
    draw_pile: VecDeque<Card>,
    discard_pile: Vec<Card>,
    current_player: PlayerId,
    direction: Direction,
    current_color: Option<Color>,
    pending_draw: u16,
    pending_kind: Option<PendingDrawKind>,
    pending_draw_source: Option<PlayerId>,
    pending_draw_colors: Vec<Color>,
    challenge: Option<ChallengeState>,
    drawn_card: Option<Card>,
    pending_skip: u16,
    pending_skip_everyone: bool,
    pending_skip_source: Option<PlayerId>,
    skip_turns: Vec<u16>,
    uno_exposed: Vec<bool>,
    uno_declared: Vec<bool>,
    jump_in_open: bool,
    pending_swap: Option<PendingSwapState>,
    pending_finisher: Option<PlayerId>,
    set_aside_cards: Vec<Card>,
    elimination_order: Vec<PlayerId>,
    flip_side: Option<FlipSide>,
    phase: Phase,
}

impl GameState {
    /// `deck[0]` 是第一张发出的牌。
    pub fn new_with_deck(
        rules: RuleSet,
        player_count: u8,
        deck: Vec<Card>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let player_count = RuleSet::validate_player_count(player_count)?;
        validate_deck(&deck, rules)?;
        let mut draw_pile = VecDeque::from(deck);
        let mut players = (0..player_count)
            .map(|index| PlayerState {
                id: PlayerId(index),
                hand: Vec::with_capacity(usize::from(RuleSet::HAND_SIZE)),
                eliminated: false,
            })
            .collect::<Vec<_>>();
        for _ in 0..RuleSet::HAND_SIZE {
            for player in &mut players {
                player.hand.push(
                    draw_pile
                        .pop_front()
                        .expect("a valid UNO deck is large enough"),
                );
            }
        }
        for player in &mut players {
            player.hand.sort_by(Card::display_cmp);
        }

        // 经典模式轮换万能摸四，FLIP 轮换万能摸二；No Mercy 不执行起始功能牌。
        let top_card = loop {
            let card = draw_pile
                .pop_front()
                .expect("a valid UNO deck has a starting card");
            if (rules.is_classic() && card.face() == Face::WildDrawFour)
                || (rules.is_flip() && card.face() == Face::WildDrawTwo)
                || (rules.is_no_mercy() && !matches!(card.face(), Face::Number(_)))
            {
                draw_pile.push_back(card);
            } else {
                break card;
            }
        };
        let mut state = Self {
            rules,
            players,
            draw_pile,
            discard_pile: vec![top_card],
            current_player: PlayerId(0),
            direction: Direction::Clockwise,
            current_color: top_card.color(),
            pending_draw: 0,
            pending_kind: None,
            pending_draw_source: None,
            pending_draw_colors: Vec::new(),
            challenge: None,
            drawn_card: None,
            pending_skip: 0,
            pending_skip_everyone: false,
            pending_skip_source: None,
            skip_turns: vec![0; player_count],
            uno_exposed: vec![false; player_count],
            uno_declared: vec![false; player_count],
            jump_in_open: false,
            pending_swap: None,
            pending_finisher: None,
            set_aside_cards: Vec::new(),
            elimination_order: Vec::new(),
            flip_side: rules.is_flip().then_some(FlipSide::Light),
            phase: Phase::Playing,
        };
        if rules.is_classic() || rules.is_flip() {
            state.apply_starting_card()?;
        }
        Ok(state)
    }

    pub const fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub fn player(&self, player: PlayerId) -> Option<&PlayerState> {
        self.players.get(player.0)
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn turn(&self) -> Option<TurnState> {
        matches!(self.phase, Phase::Playing).then(|| TurnState {
            current_player: self.current_player,
            direction: self.direction,
            current_color: self.current_color,
            top_card: *self.discard_pile.last().expect("a game has a discard"),
            pending_draw: self.pending_draw,
            pending_kind: self.pending_kind,
            pending_draw_source: self.pending_draw_source,
            challenge_offender: self.challenge.map(|challenge| challenge.offender),
            drawn_card: self.drawn_card,
            pending_skip: self.pending_skip,
            skipped_turns_remaining: self.skip_turns[self.current_player.0],
            pending_swap: self.pending_swap.map(PendingSwapState::public),
        })
    }

    pub fn pending_swap(&self) -> Option<PendingSwap> {
        self.pending_swap.map(PendingSwapState::public)
    }

    pub fn discard_pile(&self) -> &[Card] {
        &self.discard_pile
    }

    pub fn draw_pile_len(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn top_card(&self) -> Card {
        *self.discard_pile.last().expect("a game has a discard")
    }

    pub const fn current_color(&self) -> Option<Color> {
        self.current_color
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }

    pub const fn flip_side(&self) -> Option<FlipSide> {
        self.flip_side
    }

    pub fn draw_pile(&self) -> impl Iterator<Item = Card> + '_ {
        self.draw_pile.iter().copied()
    }

    pub fn skipped_turns(&self, player: PlayerId) -> Option<u16> {
        self.skip_turns.get(player.0).copied()
    }

    pub fn uno_exposed_players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.uno_exposed
            .iter()
            .enumerate()
            .filter_map(|(index, exposed)| exposed.then_some(PlayerId(index)))
    }

    pub fn uno_declared_players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.uno_declared
            .iter()
            .enumerate()
            .filter_map(|(index, declared)| declared.then_some(PlayerId(index)))
    }

    /// 返回该玩家当前唯一可抢出的物理牌。经典 108 张牌中每种彩色牌面至多
    /// 两张，桌面已经有一张，因此至多只会找到一张候选牌。
    pub fn jump_in_card(&self, player: PlayerId) -> Option<Card> {
        if !matches!(self.phase, Phase::Playing)
            || !self.jump_in_enabled()
            || !self.jump_in_open
            || self.pending_swap.is_some()
            || player == self.current_player
            || self.skip_turns.get(player.0).copied().unwrap_or(0) > 0
        {
            return None;
        }
        let top = self.top_card();
        top.color()?;
        if top.face().is_extension()
            || (!self.action_stacking_enabled() && !matches!(top.face(), Face::Number(_)))
        {
            return None;
        }
        self.players
            .get(player.0)?
            .hand
            .iter()
            .copied()
            .find(|card| *card != top && card.color() == top.color() && card.face() == top.face())
    }

    pub fn choose_initial_color(
        &mut self,
        player: PlayerId,
        color: Color,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if !self.color_allowed(color) {
            return Err(GameError::UnexpectedColor);
        }
        if let Some(PendingSwapState::ChooseColor { player: expected }) = self.pending_swap {
            if player != expected {
                return Err(GameError::NotPlayersTurn {
                    expected,
                    actual: player,
                });
            }
            self.current_color = Some(color);
            self.pending_swap = None;
            self.current_player = self.next_player(player);
            self.finish_pending_game();
            return Ok(ActionOutcome::ColorChosen { player, color });
        }
        if let Some(PendingSwapState::ColorRoulette { player: expected }) = self.pending_swap {
            if player != expected {
                return Err(GameError::NotPlayersTurn {
                    expected,
                    actual: player,
                });
            }
            let cards = self.draw_until_color(player, color)?;
            self.current_color = Some(color);
            self.pending_swap = None;
            self.check_mercy_elimination(player);
            let next_player = self.next_player(player);
            self.current_player = next_player;
            self.finish_pending_game();
            return Ok(ActionOutcome::ColorRouletteResolved {
                player,
                color,
                cards,
                next_player,
            });
        }
        if self.pending_swap.is_some() {
            return Err(GameError::MustResolveSwapEffect);
        }
        if self.current_color.is_some()
            || !self
                .discard_pile
                .last()
                .is_some_and(|card| card.face().is_wild())
        {
            return Err(GameError::InitialColorAlreadyChosen);
        }
        self.ensure_turn(player)?;
        self.current_color = Some(color);
        Ok(ActionOutcome::ColorChosen { player, color })
    }

    pub fn can_play(&self, player: PlayerId, card: Card) -> bool {
        self.ensure_turn(player).is_ok()
            && self.pending_swap.is_none()
            && self.current_color.is_some()
            && self.players[player.0].hand.contains(&card)
            && self.card_allowed_in_current_state(card)
            && self.drawn_card.is_none_or(|drawn_card| drawn_card == card)
    }

    pub fn play_card(
        &mut self,
        player: PlayerId,
        card: Card,
        chosen_color: Option<Color>,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        if self.current_color.is_none() {
            return Err(GameError::InitialColorChoiceRequired);
        }
        if !self.players[player.0].hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }
        let deferred_color = matches!(
            card.face(),
            Face::WildForceTrade | Face::WildPassHands | Face::WildColorRoulette
        );
        if card.face().is_wild() {
            if deferred_color && chosen_color.is_some() {
                return Err(GameError::UnexpectedColor);
            }
            if !deferred_color && chosen_color.is_none() {
                return Err(GameError::ColorRequired);
            }
        } else if chosen_color.is_some() {
            return Err(GameError::UnexpectedColor);
        }
        if chosen_color.is_some_and(|color| !self.color_allowed(color)) {
            return Err(GameError::UnexpectedColor);
        }
        if let Some(drawn_card) = self.drawn_card
            && card != drawn_card
        {
            return Err(GameError::MustPlayDrawnCard(drawn_card));
        }
        if self.skip_turns[player.0] > 0 {
            return Err(GameError::MustResolveSkip);
        }
        if self.pending_skip > 0 {
            if !self.skip_stack_allowed(card) {
                return Err(GameError::MustResolveSkip);
            }
        } else if self.pending_draw_active() {
            if !self.stack_allowed(card) {
                return Err(GameError::CannotStack(card));
            }
        } else if !self.matches_top(card) {
            return Err(GameError::CardDoesNotMatch);
        }

        let previous_color = self.current_color.expect("checked above");
        let wild_draw_was_legal = if (self.rules.is_classic() && card.face() == Face::WildDrawFour)
            || (self.rules.is_flip()
                && matches!(card.face(), Face::WildDrawTwo | Face::WildDrawColor))
        {
            match self.pending_kind {
                // A +4 stacked on a +2 can only be answered by the matching
                // +2 under the configured stacking rule. Other cards of the
                // same color are not playable while the draw penalty is open.
                Some(PendingDrawKind::DrawTwo) => {
                    !self.players[player.0].hand.iter().any(|other| {
                        *other != card
                            && (other.face() == Face::ReverseDrawTwo
                                || (other.face() == Face::DrawTwo
                                    && other.color() == Some(previous_color)))
                    })
                }
                _ => !self.players[player.0]
                    .hand
                    .iter()
                    .any(|other| *other != card && other.color() == Some(previous_color)),
            }
        } else {
            true
        };
        let declared_uno = self.uno_declared[player.0];
        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        self.drawn_card = None;
        self.jump_in_open = false;
        remove_card(&mut self.players[player.0].hand, card);
        self.discard_pile.push(card);
        self.current_color = if deferred_color {
            None
        } else {
            chosen_color.or(card.color())
        };

        if self.players[player.0].hand.is_empty() {
            self.pending_finisher.get_or_insert(player);
        }

        let mut effect = None;
        let next_player = match card.face() {
            Face::DrawOne => {
                self.pending_draw = self.pending_draw.saturating_add(1);
                self.pending_kind = Some(PendingDrawKind::FlipDrawOne);
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::DrawTwo => {
                self.pending_draw += 2;
                self.pending_kind = Some(if self.rules.is_no_mercy() {
                    PendingDrawKind::NoMercy(2)
                } else {
                    PendingDrawKind::DrawTwo
                });
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::DrawFour => {
                self.pending_draw += 4;
                self.pending_kind = Some(PendingDrawKind::NoMercy(4));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::DrawFive => {
                self.pending_draw = self.pending_draw.saturating_add(5);
                self.pending_kind = Some(PendingDrawKind::FlipDrawFive);
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::WildDrawTwo => {
                self.pending_draw = self.pending_draw.saturating_add(2);
                self.pending_kind = Some(PendingDrawKind::FlipWildDrawTwo);
                self.pending_draw_source = Some(player);
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: wild_draw_was_legal,
                    });
                }
                self.next_player(player)
            }
            Face::WildDrawFour => {
                self.pending_draw += 4;
                self.pending_kind = Some(PendingDrawKind::WildDrawFour);
                self.pending_draw_source = Some(player);
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: wild_draw_was_legal,
                    });
                }
                self.next_player(player)
            }
            Face::WildDrawColor => {
                self.pending_draw = self.pending_draw.saturating_add(1);
                self.pending_kind = Some(PendingDrawKind::FlipWildDrawColor);
                self.pending_draw_source = Some(player);
                self.pending_draw_colors
                    .push(chosen_color.expect("wild draw color requires a color"));
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: wild_draw_was_legal,
                    });
                }
                self.next_player(player)
            }
            Face::Reverse if self.players.len() == 2 => player,
            Face::Reverse => {
                self.direction = self.direction.reversed();
                self.next_player(player)
            }
            Face::Skip => {
                self.pending_skip = if self.action_stacking_enabled() {
                    self.pending_skip_everyone = false;
                    self.pending_skip_source = Some(player);
                    self.pending_skip.saturating_add(1)
                } else {
                    let target = self.next_player(player);
                    self.skip_turns[target.0] = self.skip_turns[target.0].max(1);
                    0
                };
                self.next_player(player)
            }
            Face::SkipEveryone if self.rules.is_flip() && self.action_stacking_enabled() => {
                self.pending_skip = self.pending_skip.saturating_add(1);
                self.pending_skip_everyone = true;
                self.pending_skip_source = Some(player);
                self.next_player(player)
            }
            Face::SkipEveryone => player,
            Face::Flip => {
                let side = self.flip_everything();
                effect = Some(PlayedEffect::Flipped { side });
                self.next_player(player)
            }
            Face::DiscardAll => {
                let color = card.color().expect("discard-all is colored");
                let mut discarded = Vec::new();
                self.players[player.0].hand.retain(|candidate| {
                    if candidate.color() == Some(color) {
                        discarded.push(*candidate);
                        false
                    } else {
                        true
                    }
                });
                if !discarded.is_empty() {
                    let top = self.discard_pile.pop().expect("played card is on top");
                    self.discard_pile.extend(discarded.iter().copied());
                    self.discard_pile.push(top);
                    effect = Some(PlayedEffect::CardsDiscarded { cards: discarded });
                }
                if self.players[player.0].hand.is_empty() {
                    self.pending_finisher.get_or_insert(player);
                }
                self.next_player(player)
            }
            Face::ReverseDrawTwo => {
                self.direction = self.direction.reversed();
                self.pending_draw += 2;
                self.pending_kind = Some(PendingDrawKind::DrawTwo);
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::ReverseSkip => {
                self.direction = self.direction.reversed();
                self.pending_skip = if self.rules.action_stacking {
                    self.pending_skip_everyone = false;
                    self.pending_skip_source = Some(player);
                    self.pending_skip.saturating_add(1)
                } else {
                    let target = self.next_player(player);
                    self.skip_turns[target.0] = self.skip_turns[target.0].max(1);
                    0
                };
                self.next_player(player)
            }
            Face::StackOne => {
                self.pending_draw = self.pending_draw.saturating_add(1);
                self.pending_kind = Some(PendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                self.next_player(player)
            }
            Face::StackTwo => {
                self.pending_draw = self.pending_draw.saturating_add(2);
                self.pending_kind = Some(PendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                self.next_player(player)
            }
            Face::SwapOne => {
                self.pending_swap = Some(PendingSwapState::SwapOneTarget {
                    player,
                    declared_uno,
                });
                player
            }
            Face::RefreshHand => {
                let count = self.refresh_hand(player)?;
                effect = Some(PlayedEffect::HandRefreshed { count });
                self.update_uno_after_play(player, declared_uno);
                self.next_player(player)
            }
            Face::WildForceTrade => {
                self.pending_swap = Some(PendingSwapState::ForceTrade { player });
                player
            }
            Face::WildPassHands => {
                self.pass_hands();
                self.uno_exposed.fill(false);
                self.uno_declared.fill(false);
                self.pending_swap = Some(PendingSwapState::ChooseColor { player });
                effect = Some(PlayedEffect::HandsPassed {
                    direction: self.direction,
                });
                player
            }
            Face::WildPowerReverse => {
                self.direction = self.direction.reversed();
                player
            }
            Face::WildNoU if self.pending_draw_active() => {
                self.direction = self.direction.reversed();
                let target = self
                    .pending_draw_source
                    .expect("a draw penalty records its latest source");
                let cards = self.draw_cards_for(target, self.pending_draw)?;
                self.clear_pending_draw();
                effect = Some(PlayedEffect::DrawReflected {
                    player: target,
                    cards,
                });
                self.next_player(target)
            }
            Face::WildNoU => {
                self.direction = self.direction.reversed();
                self.next_player(player)
            }
            Face::WildStackThree => {
                self.pending_draw = self.pending_draw.saturating_add(3);
                self.pending_kind = Some(PendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                self.next_player(player)
            }
            Face::WildStackNumber => {
                let (cards, value) = self.reveal_stack_number()?;
                self.pending_draw = self.pending_draw.saturating_add(u16::from(value));
                self.pending_kind = Some(PendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                effect = Some(PlayedEffect::StackNumberRevealed { cards, value });
                self.next_player(player)
            }
            Face::WildReverseDrawFour => {
                self.direction = self.direction.reversed();
                self.pending_draw = self.pending_draw.saturating_add(4);
                self.pending_kind = Some(PendingDrawKind::NoMercy(4));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::WildDrawSix => {
                self.pending_draw = self.pending_draw.saturating_add(6);
                self.pending_kind = Some(PendingDrawKind::NoMercy(6));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::WildDrawTen => {
                self.pending_draw = self.pending_draw.saturating_add(10);
                self.pending_kind = Some(PendingDrawKind::NoMercy(10));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            Face::WildColorRoulette => {
                let target = self.next_player(player);
                self.pending_swap = Some(PendingSwapState::ColorRoulette { player: target });
                target
            }
            Face::Number(0) if self.rules.is_no_mercy() && self.rules.no_mercy.zero_pass => {
                self.pass_hands();
                self.uno_exposed.fill(false);
                self.uno_declared.fill(false);
                effect = Some(PlayedEffect::HandsPassed {
                    direction: self.direction,
                });
                self.next_player(player)
            }
            Face::Number(7) if self.rules.is_no_mercy() && self.rules.no_mercy.seven_swap => {
                self.pending_swap = Some(PendingSwapState::SevenSwap { player });
                player
            }
            Face::Number(_) | Face::Wild | Face::DarkWild => self.next_player(player),
        };
        let no_mercy_hand_effect = self.rules.is_no_mercy()
            && (matches!(card.face(), Face::Number(0)) && self.rules.no_mercy.zero_pass
                || matches!(card.face(), Face::Number(7)) && self.rules.no_mercy.seven_swap);
        if !matches!(
            card.face(),
            Face::SwapOne | Face::RefreshHand | Face::WildForceTrade | Face::WildPassHands
        ) && !no_mercy_hand_effect
        {
            self.update_uno_after_play(player, declared_uno);
        }
        self.current_player = next_player;
        if !matches!(
            card.face(),
            Face::SwapOne
                | Face::RefreshHand
                | Face::WildForceTrade
                | Face::WildPassHands
                | Face::WildColorRoulette
        ) && !no_mercy_hand_effect
        {
            self.jump_in_open = self.jump_in_enabled() && card.color().is_some();
        }
        if self.pending_swap.is_none()
            && !self.pending_draw_active()
            && self.pending_skip == 0
            && self.skip_turns[self.current_player.0] == 0
        {
            self.finish_pending_game();
        }
        Ok(ActionOutcome::Played {
            player,
            card,
            next_player,
            effect,
        })
    }

    pub fn choose_swap_one_target(
        &mut self,
        player: PlayerId,
        target: PlayerId,
        taken_index: usize,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_player(target)?;
        let Some(PendingSwapState::SwapOneTarget {
            player: expected,
            declared_uno,
        }) = self.pending_swap
        else {
            return Err(GameError::NoSwapEffect);
        };
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if target == player || self.players[target.0].eliminated {
            return Err(GameError::InvalidSwapTargets);
        }
        let Some(card) = self.players[target.0].hand.get(taken_index).copied() else {
            return Err(GameError::InvalidSwapTargets);
        };
        self.players[target.0].hand.remove(taken_index);
        self.players[player.0].hand.push(card);
        self.players[player.0].hand.sort_by(Card::display_cmp);
        self.pending_swap = Some(PendingSwapState::SwapOneGive {
            player,
            target,
            declared_uno,
        });
        Ok(ActionOutcome::SwapOneCardTaken { player, target })
    }

    pub fn choose_seven_swap_target(
        &mut self,
        player: PlayerId,
        target: PlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_player(target)?;
        let Some(PendingSwapState::SevenSwap { player: expected }) = self.pending_swap else {
            return Err(GameError::NoSwapEffect);
        };
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if target == player || self.players[target.0].eliminated {
            return Err(GameError::InvalidSwapTargets);
        }
        swap_player_hands(&mut self.players, player, target);
        for selected in [player, target] {
            self.uno_exposed[selected.0] = false;
            self.uno_declared[selected.0] = false;
        }
        self.pending_swap = None;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::HandsTraded {
            player,
            first: player,
            second: target,
        })
    }

    pub fn give_swap_one_card(
        &mut self,
        player: PlayerId,
        card: Card,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        let Some(PendingSwapState::SwapOneGive {
            player: expected,
            target,
            declared_uno,
        }) = self.pending_swap
        else {
            return Err(GameError::NoSwapEffect);
        };
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if !self.players[player.0].hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }
        remove_card(&mut self.players[player.0].hand, card);
        self.players[target.0].hand.push(card);
        self.players[target.0].hand.sort_by(Card::display_cmp);
        self.pending_swap = None;
        self.update_uno_after_play(player, declared_uno);
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.jump_in_open = false;
        self.finish_pending_game();
        Ok(ActionOutcome::SwapOneCompleted {
            player,
            target,
            next_player,
        })
    }

    pub fn force_trade_hands(
        &mut self,
        player: PlayerId,
        first: PlayerId,
        second: PlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_player(first)?;
        self.ensure_player(second)?;
        let Some(PendingSwapState::ForceTrade { player: expected }) = self.pending_swap else {
            return Err(GameError::NoSwapEffect);
        };
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if first == second || self.players[first.0].eliminated || self.players[second.0].eliminated
        {
            return Err(GameError::InvalidSwapTargets);
        }
        swap_player_hands(&mut self.players, first, second);
        for target in [first, second] {
            self.uno_exposed[target.0] = false;
            self.uno_declared[target.0] = false;
        }
        self.pending_swap = Some(PendingSwapState::ChooseColor { player });
        Ok(ActionOutcome::HandsTraded {
            player,
            first,
            second,
        })
    }

    /// 一次提交一张普通牌，或在抢出规则下提交两张完全相同的彩色牌。
    /// 双牌通过克隆状态原子结算，第二张失败时不会留下只打出第一张的半成品。
    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[Card],
        chosen_color: Option<Color>,
    ) -> Result<ActionOutcome, GameError> {
        let [first, second] = cards else {
            return match cards {
                [card] => self.play_card(player, *card, chosen_color),
                _ => Err(GameError::CannotPlayTogether),
            };
        };
        if !self.jump_in_enabled()
            || (!self.action_stacking_enabled() && !matches!(first.face(), Face::Number(_)))
            || chosen_color.is_some()
            || first == second
            || first.color().is_none()
            || first.face().is_extension()
            || first.color() != second.color()
            || first.face() != second.face()
            || self.uno_declared.get(player.0).copied().unwrap_or(false)
            || !self
                .players
                .get(player.0)
                .is_some_and(|state| state.hand.contains(first) && state.hand.contains(second))
        {
            return Err(GameError::CannotPlayTogether);
        }

        if first.face() == Face::Flip {
            return self.play_flip_pair(player, *first, *second);
        }

        let mut staged = self.clone();
        staged.play_card(player, *first, None)?;
        let outcome = if staged.current_player == player {
            staged.play_card(player, *second, None)?
        } else {
            staged.jump_in(player, *second)?
        };
        *self = staged;
        Ok(outcome)
    }

    fn play_flip_pair(
        &mut self,
        player: PlayerId,
        first: Card,
        second: Card,
    ) -> Result<ActionOutcome, GameError> {
        if !self.can_play(player, first)
            || self.drawn_card.is_some()
            || self.pending_draw_active()
            || self.pending_skip > 0
            || self.skip_turns[player.0] > 0
        {
            return Err(GameError::CannotPlayTogether);
        }
        remove_card(&mut self.players[player.0].hand, first);
        remove_card(&mut self.players[player.0].hand, second);
        self.discard_pile.extend([first, second]);
        self.current_color = second.color();
        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        if self.players[player.0].hand.is_empty() {
            self.pending_finisher.get_or_insert(player);
        } else {
            self.update_uno_after_play(player, false);
        }
        self.jump_in_open = false;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::Played {
            player,
            card: second,
            next_player,
            effect: None,
        })
    }

    /// 非下家在窗口关闭前抢出与桌面牌颜色、牌面完全一致的另一张物理牌。
    pub fn jump_in(&mut self, player: PlayerId, card: Card) -> Result<ActionOutcome, GameError> {
        if self.jump_in_card(player) != Some(card) {
            return Err(GameError::CannotJumpIn);
        }
        self.current_player = player;
        self.play_card(player, card, None)
    }

    pub fn draw_card(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.current_color.is_none() {
            return Err(GameError::InitialColorChoiceRequired);
        }
        if self.pending_draw_active() {
            return Err(GameError::MustResolveDrawPenalty);
        }
        if self.drawn_card.is_some() {
            return Err(GameError::MustDrawBeforePassing);
        }
        let mut cards = self.draw_cards_for(player, 1)?;
        self.jump_in_open = false;
        loop {
            if self.check_mercy_elimination(player)
                || !self.rules.is_no_mercy()
                || !self.rules.no_mercy.draw_until_playable
                || cards
                    .last()
                    .is_some_and(|card| self.card_allowed_in_current_state(*card))
            {
                break;
            }
            cards.extend(self.draw_cards_for(player, 1)?);
        }
        let playable = if self.players[player.0].eliminated {
            None
        } else {
            cards
                .last()
                .copied()
                .filter(|card| self.card_allowed_in_current_state(*card))
        };
        let next_player = if let Some(card) = playable {
            self.drawn_card = Some(card);
            player
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        Ok(ActionOutcome::DrewCards {
            player,
            cards,
            playable,
            next_player,
        })
    }

    pub fn pass_after_draw(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.rules.is_no_mercy()
            && self.rules.no_mercy.draw_until_playable
            && let Some(card) = self.drawn_card
        {
            return Err(GameError::MustPlayDrawnCard(card));
        }
        if self.drawn_card.is_none() {
            return Err(GameError::MustDrawBeforePassing);
        }
        self.drawn_card = None;
        self.jump_in_open = false;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        Ok(ActionOutcome::PassedAfterDraw {
            player,
            next_player,
        })
    }

    pub fn accept_draw_penalty(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        if !self.pending_draw_active() {
            return Err(GameError::NoDrawPenalty);
        }
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.0] > 0;
        let cards = self.draw_pending_penalty(player, 0)?;
        self.check_mercy_elimination(player);
        self.jump_in_open = false;
        self.clear_pending_draw();
        // 罚牌始终属于功能牌出牌者的直接下家。若该玩家同时仍被禁手，先由其
        // 收下罚牌并留在当前回合，随后再单独消耗禁手，不能把罚牌传给下下家。
        let next_player = if must_resolve_skip {
            player
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::PenaltyDrawn {
            player,
            cards,
            next_player,
        })
    }

    pub fn challenge_draw_four(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        let challenge = self.challenge.ok_or(GameError::CannotChallenge)?;
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.0] > 0;
        let (result, penalized, extra, next_player) = if challenge.was_legal {
            (
                ChallengeResult::Failed,
                player,
                2,
                if must_resolve_skip {
                    player
                } else {
                    self.next_player(player)
                },
            )
        } else {
            (ChallengeResult::Successful, challenge.offender, 0, player)
        };
        let cards = self.draw_pending_penalty(penalized, extra)?;
        self.check_mercy_elimination(penalized);
        self.jump_in_open = false;
        self.clear_pending_draw();
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::ChallengeResolved {
            challenger: player,
            offender: challenge.offender,
            result,
            penalized,
            cards,
            next_player,
        })
    }

    pub fn call_uno(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if self.players[player.0].eliminated {
            return Err(GameError::PlayerEliminated(player));
        }
        if !self.rules.uno_callout() {
            return Err(GameError::UnoCalloutDisabled);
        }
        if self.uno_declared[player.0] {
            return Err(GameError::CannotCallUno(player));
        }
        let can_recover_after_play = self.can_recover_uno(player);
        let can_declare_before_play = self.can_declare_uno(player);
        if !can_declare_before_play && !can_recover_after_play {
            return Err(GameError::CannotCallUno(player));
        }

        if can_recover_after_play {
            self.uno_exposed[player.0] = false;
        } else {
            self.uno_declared[player.0] = true;
        }
        if player == self.current_player {
            self.jump_in_open = false;
        }
        Ok(ActionOutcome::UnoCalled { player })
    }

    pub fn can_call_uno(&self, player: PlayerId) -> bool {
        matches!(self.phase, Phase::Playing)
            && self.rules.uno_callout()
            && self
                .players
                .get(player.0)
                .is_some_and(|state| !state.eliminated)
            && !self.uno_declared.get(player.0).copied().unwrap_or(false)
            && (self.can_declare_uno(player) || self.can_recover_uno(player))
    }

    fn can_declare_uno(&self, player: PlayerId) -> bool {
        player == self.current_player
            && self.pending_swap.is_none()
            && self.players[player.0].hand.len() == 2
            && !self.pending_draw_active()
            && self.pending_skip == 0
            && self.skip_turns[player.0] == 0
            && self.players[player.0]
                .hand
                .iter()
                .copied()
                .any(|card| self.can_play(player, card))
    }

    fn can_recover_uno(&self, player: PlayerId) -> bool {
        self.players[player.0].hand.len() == 1 && self.uno_exposed[player.0]
    }

    pub fn report_uno(
        &mut self,
        reporter: PlayerId,
        target: PlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(reporter)?;
        self.ensure_player(target)?;
        if self.players[reporter.0].eliminated {
            return Err(GameError::PlayerEliminated(reporter));
        }
        if !self.rules.uno_callout() {
            return Err(GameError::UnoCalloutDisabled);
        }
        if reporter == target {
            return Err(GameError::CannotReportSelf);
        }
        if !self.uno_exposed[target.0] {
            return Err(GameError::PlayerNotReportable(target));
        }
        let cards = self.draw_cards_for(target, 2)?;
        self.check_mercy_elimination(target);
        if reporter == self.current_player {
            self.jump_in_open = false;
        }
        self.uno_exposed[target.0] = false;
        Ok(ActionOutcome::UnoReported {
            reporter,
            target,
            cards,
        })
    }

    pub fn resolve_skip(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        if self.pending_draw_active() {
            return Err(GameError::MustResolveDrawPenalty);
        }
        let pending = self.pending_skip;
        if self.pending_skip_everyone {
            let source = self
                .pending_skip_source
                .expect("stacked skip-everyone records its latest source");
            for state in &self.players {
                if !state.eliminated && state.id != source {
                    self.skip_turns[state.id.0] =
                        self.skip_turns[state.id.0].saturating_add(pending);
                }
            }
        }
        let existing = self.skip_turns[player.0];
        if pending == 0 && existing == 0 {
            return Err(GameError::NoSkipToResolve);
        }
        let total = if self.pending_skip_everyone {
            existing
        } else if self.action_stacking_enabled() {
            existing.saturating_add(pending)
        } else {
            existing.max(pending.min(1))
        };
        let cards = if self.skip_draw_penalty_enabled() {
            self.draw_cards_for(player, 1)?
        } else {
            Vec::new()
        };
        self.check_mercy_elimination(player);
        self.jump_in_open = false;
        self.pending_skip = 0;
        self.pending_skip_everyone = false;
        self.pending_skip_source = None;
        let remaining = total.saturating_sub(1);
        self.skip_turns[player.0] = remaining;
        self.drawn_card = None;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            next_player,
        })
    }

    fn apply_starting_card(&mut self) -> Result<(), GameError> {
        let card = *self.discard_pile.last().expect("starting card exists");
        match card.face() {
            Face::DrawOne => {
                let player = self.current_player;
                self.draw_cards_for(player, 1)?;
                self.current_player = self.next_player(player);
            }
            Face::DrawTwo => {
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            Face::Reverse => {
                self.direction = Direction::CounterClockwise;
                self.current_player = self.next_player(self.current_player);
            }
            Face::Skip => {
                if self.action_stacking_enabled() {
                    self.pending_skip = 1;
                } else {
                    self.skip_turns[self.current_player.0] = 1;
                }
            }
            Face::ReverseDrawTwo => {
                self.direction = self.direction.reversed();
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            Face::ReverseSkip => {
                self.direction = self.direction.reversed();
                if self.rules.action_stacking {
                    self.pending_skip = 1;
                } else {
                    self.skip_turns[self.current_player.0] = 1;
                }
            }
            Face::Flip => {
                self.flip_everything();
            }
            Face::StackOne => {
                let player = self.current_player;
                self.draw_cards_for(player, 1)?;
                self.current_player = self.next_player(player);
            }
            Face::StackTwo => {
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            Face::Wild
            | Face::DarkWild
            | Face::WildForceTrade
            | Face::WildPassHands
            | Face::WildPowerReverse
            | Face::WildNoU
            | Face::WildStackThree
            | Face::WildStackNumber
            | Face::WildReverseDrawFour
            | Face::WildDrawSix
            | Face::WildDrawTen
            | Face::WildColorRoulette => self.current_color = None,
            Face::Number(_)
            | Face::DrawFour
            | Face::SkipEveryone
            | Face::DiscardAll
            | Face::SwapOne
            | Face::RefreshHand => {}
            Face::DrawFive | Face::WildDrawColor => {}
            Face::WildDrawFour | Face::WildDrawTwo => {
                unreachable!("initial wild draw card was rotated away")
            }
        }
        Ok(())
    }

    fn ensure_playing(&self) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            Err(GameError::GameAlreadyFinished)
        } else {
            Ok(())
        }
    }

    fn ensure_player(&self, player: PlayerId) -> Result<(), GameError> {
        if player.0 < self.players.len() {
            Ok(())
        } else {
            Err(GameError::InvalidPlayer(player))
        }
    }

    fn ensure_turn(&self, player: PlayerId) -> Result<(), GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if player == self.current_player && !self.players[player.0].eliminated {
            Ok(())
        } else {
            Err(GameError::NotPlayersTurn {
                expected: self.current_player,
                actual: player,
            })
        }
    }

    fn ensure_skip_resolved(&self, player: PlayerId) -> Result<(), GameError> {
        if self.pending_skip > 0 || self.skip_turns[player.0] > 0 {
            Err(GameError::MustResolveSkip)
        } else {
            Ok(())
        }
    }

    fn ensure_swap_resolved(&self) -> Result<(), GameError> {
        if self.pending_swap.is_some() {
            Err(GameError::MustResolveSwapEffect)
        } else {
            Ok(())
        }
    }

    fn ensure_uno_followup_resolved(&self, player: PlayerId) -> Result<(), GameError> {
        if self.uno_declared[player.0] {
            Err(GameError::MustPlayAfterUno)
        } else {
            Ok(())
        }
    }

    const fn color_allowed(&self, color: Color) -> bool {
        match self.flip_side {
            Some(FlipSide::Dark) => color.is_dark(),
            Some(FlipSide::Light) | None => color.is_light(),
        }
    }

    const fn action_stacking_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.action_stacking
        } else {
            self.rules.is_classic() && self.rules.action_stacking
        }
    }

    const fn skip_draw_penalty_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.skip_draw_penalty
        } else {
            self.rules.is_classic() && self.rules.skip_draw_penalty
        }
    }

    const fn jump_in_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.jump_in
        } else {
            self.rules.is_classic() && self.rules.jump_in
        }
    }

    fn skip_stack_allowed(&self, card: Card) -> bool {
        if !self.action_stacking_enabled() {
            return false;
        }
        if self.rules.is_flip() {
            return self
                .discard_pile
                .last()
                .is_some_and(|top| top.face() == card.face())
                && matches!(card.face(), Face::Skip | Face::SkipEveryone);
        }
        matches!(card.face(), Face::Skip | Face::ReverseSkip)
    }

    fn flip_everything(&mut self) -> FlipSide {
        for player in &mut self.players {
            for card in &mut player.hand {
                *card = card.flipped();
            }
            player.hand.sort_by(Card::display_cmp);
        }
        for card in &mut self.draw_pile {
            *card = card.flipped();
        }
        self.draw_pile.make_contiguous().reverse();
        for card in &mut self.discard_pile {
            *card = card.flipped();
        }
        self.discard_pile.reverse();
        for card in &mut self.set_aside_cards {
            *card = card.flipped();
        }
        self.drawn_card = self.drawn_card.map(Card::flipped);
        let side = match self.flip_side.unwrap_or(FlipSide::Light) {
            FlipSide::Light => FlipSide::Dark,
            FlipSide::Dark => FlipSide::Light,
        };
        self.flip_side = Some(side);
        self.current_color = self.top_card().color();
        side
    }

    fn matches_top(&self, card: Card) -> bool {
        card.face().is_wild()
            || card.color() == self.current_color
            || self
                .discard_pile
                .last()
                .is_some_and(|top| faces_match(top.face(), card.face()))
    }

    fn stack_allowed(&self, card: Card) -> bool {
        if self.rules.is_no_mercy() {
            return match self.pending_kind {
                Some(PendingDrawKind::NoMercy(minimum)) => card
                    .face()
                    .draw_value()
                    .is_some_and(|value| value >= minimum),
                _ => false,
            };
        }
        if self.rules.is_flip() {
            if !self.rules.flip.action_stacking {
                return false;
            }
            return matches!(
                (self.pending_kind, card.face()),
                (
                    Some(PendingDrawKind::FlipDrawOne),
                    Face::DrawOne | Face::WildDrawTwo
                ) | (Some(PendingDrawKind::FlipWildDrawTwo), Face::WildDrawTwo)
                    | (Some(PendingDrawKind::FlipDrawFive), Face::DrawFive)
                    | (
                        Some(PendingDrawKind::FlipWildDrawColor),
                        Face::WildDrawColor
                    )
            );
        }
        if !self.rules.action_stacking {
            return false;
        }
        match (self.pending_kind, card.face()) {
            (Some(PendingDrawKind::DrawTwo), Face::DrawTwo | Face::ReverseDrawTwo)
            | (Some(PendingDrawKind::WildDrawFour), Face::WildDrawFour) => true,
            (_, Face::WildNoU) => true,
            (_, Face::StackOne | Face::StackTwo) => card.color() == self.current_color,
            (_, Face::WildStackThree | Face::WildStackNumber) => true,
            (Some(PendingDrawKind::DrawTwo), Face::WildDrawFour) => true,
            _ => false,
        }
    }

    fn card_allowed_in_current_state(&self, card: Card) -> bool {
        if self.pending_swap.is_some() || self.skip_turns[self.current_player.0] > 0 {
            false
        } else if self.pending_skip > 0 {
            self.skip_stack_allowed(card)
        } else if self.pending_draw_active() {
            self.stack_allowed(card)
        } else {
            self.matches_top(card)
        }
    }

    fn next_player(&self, player: PlayerId) -> PlayerId {
        let count = self.players.len();
        let mut next = player;
        for _ in 0..count {
            next = match self.direction {
                Direction::Clockwise => PlayerId((next.0 + 1) % count),
                Direction::CounterClockwise => PlayerId((next.0 + count - 1) % count),
            };
            if !self.players[next.0].eliminated {
                return next;
            }
        }
        player
    }

    fn draw_cards_for(&mut self, player: PlayerId, count: u16) -> Result<Vec<Card>, GameError> {
        self.ensure_player(player)?;
        let mut cards = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            self.players[player.0].hand.push(card);
            cards.push(card);
        }
        self.players[player.0].hand.sort_by(Card::display_cmp);
        Ok(cards)
    }

    fn draw_pending_penalty(
        &mut self,
        player: PlayerId,
        extra: u16,
    ) -> Result<Vec<Card>, GameError> {
        if self.pending_kind == Some(PendingDrawKind::FlipWildDrawColor) {
            let colors = self.pending_draw_colors.clone();
            let mut cards = Vec::new();
            for color in colors {
                cards.extend(self.draw_until_color(player, color)?);
            }
            cards.extend(self.draw_cards_for(player, extra)?);
            Ok(cards)
        } else {
            self.draw_cards_for(player, self.pending_draw.saturating_add(extra))
        }
    }

    fn replenish_draw_pile(&mut self) {
        if !self.draw_pile.is_empty() {
            return;
        }
        if self.discard_pile.len() <= 1 && self.set_aside_cards.is_empty() {
            return;
        }
        let top = self.discard_pile.pop().expect("checked above");
        let mut recycled = std::mem::take(&mut self.discard_pile);
        recycled.append(&mut self.set_aside_cards);
        recycled.reverse();
        self.draw_pile = recycled.into();
        self.discard_pile.push(top);
    }

    fn clear_pending_draw(&mut self) {
        self.pending_draw = 0;
        self.pending_kind = None;
        self.pending_draw_source = None;
        self.pending_draw_colors.clear();
        self.challenge = None;
    }

    const fn pending_draw_active(&self) -> bool {
        self.pending_kind.is_some()
    }

    fn reveal_stack_number(&mut self) -> Result<(Vec<Card>, u8), GameError> {
        let mut cards = Vec::new();
        let value = loop {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            cards.push(card);
            if let Face::Number(value) = card.face() {
                break value;
            }
        };
        let mut discard = cards.clone();
        discard.append(&mut self.discard_pile);
        self.discard_pile = discard;
        Ok((cards, value))
    }

    fn draw_until_color(&mut self, player: PlayerId, color: Color) -> Result<Vec<Card>, GameError> {
        let mut cards = Vec::new();
        loop {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            self.players[player.0].hand.push(card);
            cards.push(card);
            if card.color() == Some(color) || self.check_mercy_elimination(player) {
                break;
            }
        }
        self.players[player.0].hand.sort_by(Card::display_cmp);
        Ok(cards)
    }

    fn check_mercy_elimination(&mut self, player: PlayerId) -> bool {
        if !self.rules.is_no_mercy()
            || !self.rules.no_mercy.mercy_elimination
            || self.players[player.0].eliminated
            || self.players[player.0].hand.len() < 25
        {
            return false;
        }
        self.players[player.0].eliminated = true;
        self.elimination_order.push(player);
        self.set_aside_cards
            .append(&mut self.players[player.0].hand);
        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        self.skip_turns[player.0] = 0;
        if self
            .players
            .iter()
            .filter(|state| !state.eliminated)
            .count()
            == 1
        {
            let winner = self
                .players
                .iter()
                .find(|state| !state.eliminated)
                .expect("one active player remains")
                .id;
            self.finish(winner);
        }
        true
    }

    fn update_uno_after_play(&mut self, player: PlayerId, declared_uno: bool) {
        if self.rules.uno_callout() && self.players[player.0].hand.len() == 1 && !declared_uno {
            self.uno_exposed[player.0] = true;
        }
    }

    fn refresh_hand(&mut self, player: PlayerId) -> Result<u16, GameError> {
        let old_hand = std::mem::take(&mut self.players[player.0].hand);
        let count = u16::try_from(old_hand.len()).unwrap_or(u16::MAX);
        let mut discard = old_hand;
        discard.append(&mut self.discard_pile);
        self.discard_pile = discard;
        self.draw_cards_for(player, count)?;
        Ok(count)
    }

    fn pass_hands(&mut self) {
        let active = self
            .players
            .iter()
            .filter(|player| !player.eliminated)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        let mut old_hands = active
            .iter()
            .map(|player| std::mem::take(&mut self.players[player.0].hand))
            .collect::<Vec<_>>();
        let count = active.len();
        for (source, hand) in old_hands.iter_mut().enumerate() {
            let target_index = match self.direction {
                Direction::Clockwise => (source + 1) % count,
                Direction::CounterClockwise => (source + count - 1) % count,
            };
            self.players[active[target_index].0].hand = std::mem::take(hand);
        }
    }

    fn finish(&mut self, winner: PlayerId) -> GameResult {
        self.clear_pending_draw();
        self.pending_skip = 0;
        self.pending_skip_everyone = false;
        self.pending_skip_source = None;
        self.skip_turns.fill(0);
        self.drawn_card = None;
        self.uno_exposed.fill(false);
        self.uno_declared.fill(false);
        self.jump_in_open = false;
        self.pending_swap = None;
        self.pending_finisher = None;
        let hand_scores = self
            .players
            .iter()
            .map(PlayerState::hand_score)
            .collect::<Vec<_>>();
        let placements =
            placements_with_eliminations(winner, &hand_scores, &self.elimination_order)
                .expect("the recorded elimination order contains valid players");
        let result = GameResult {
            winner,
            reference_deltas: reference_point_deltas_for_placements(&placements)
                .expect("validated placements"),
            placements,
            hand_scores,
        };
        self.phase = Phase::Finished(result.clone());
        result
    }

    fn finish_pending_game(&mut self) -> Option<GameResult> {
        if !matches!(self.phase, Phase::Playing) {
            return None;
        }
        if self.pending_swap.is_some()
            || self.pending_draw_active()
            || self.pending_skip > 0
            || self.skip_turns.iter().any(|turns| *turns > 0)
        {
            return None;
        }
        let original = self.pending_finisher?;
        let winner = self
            .players
            .get(original.0)
            .filter(|player| !player.eliminated && player.hand.is_empty())
            .map(|player| player.id)
            .or_else(|| {
                self.players
                    .iter()
                    .find(|player| !player.eliminated && player.hand.is_empty())
                    .map(|player| player.id)
            })?;
        Some(self.finish(winner))
    }
}

fn remove_card(hand: &mut Vec<Card>, card: Card) {
    let index = hand
        .iter()
        .position(|candidate| *candidate == card)
        .expect("caller checked that the card is in hand");
    hand.remove(index);
}

fn faces_match(left: Face, right: Face) -> bool {
    if left == right {
        return !matches!(left, Face::StackOne | Face::StackTwo);
    }
    matches!(
        (left, right),
        (Face::ReverseDrawTwo, Face::Reverse | Face::DrawTwo)
            | (Face::Reverse | Face::DrawTwo, Face::ReverseDrawTwo)
            | (Face::ReverseSkip, Face::Reverse | Face::Skip)
            | (Face::Reverse | Face::Skip, Face::ReverseSkip)
    )
}

fn validate_deck(deck: &[Card], rules: RuleSet) -> Result<(), GameError> {
    let expected = build_deck_for_rules(rules);
    if deck.len() != expected.len() {
        return Err(GameError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    if rules.is_flip() {
        let mut actual_light = deck
            .iter()
            .map(|card| (card.color(), card.face()))
            .collect::<Vec<_>>();
        let mut expected_light = expected
            .iter()
            .map(|card| (card.color(), card.face()))
            .collect::<Vec<_>>();
        let mut actual_dark = deck
            .iter()
            .filter_map(|card| card.opposite().map(|side| (side.color(), side.face())))
            .collect::<Vec<_>>();
        let mut expected_dark = expected
            .iter()
            .filter_map(|card| card.opposite().map(|side| (side.color(), side.face())))
            .collect::<Vec<_>>();
        actual_light.sort_unstable();
        expected_light.sort_unstable();
        actual_dark.sort_unstable();
        expected_dark.sort_unstable();
        if actual_light != expected_light
            || actual_dark != expected_dark
            || deck.iter().copied().collect::<HashSet<_>>().len() != deck.len()
        {
            return Err(GameError::InvalidDeckContents);
        }
        return Ok(());
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}

fn swap_player_hands(players: &mut [PlayerState], first: PlayerId, second: PlayerId) {
    let (low, high) = if first.0 < second.0 {
        (first.0, second.0)
    } else {
        (second.0, first.0)
    };
    let (left, right) = players.split_at_mut(high);
    std::mem::swap(&mut left[low].hand, &mut right[0].hand);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardSide, build_deck, build_flip_deck, build_no_mercy_deck};

    fn no_mercy_game(player_count: u8) -> GameState {
        let rules = RuleSet {
            mode: crate::Mode::NoMercy,
            ..RuleSet::default()
        };
        GameState::new_with_deck(rules, player_count, build_no_mercy_deck()).unwrap()
    }

    fn flip_rules() -> RuleSet {
        RuleSet {
            mode: crate::Mode::Flip,
            ..RuleSet::default()
        }
    }

    fn flip_card(color: Color, face: Face) -> Card {
        Card::paired(
            CardSide::colored(color, face),
            CardSide::colored(Color::Pink, Face::Number(1)),
            0,
        )
    }

    fn deck_with_prefix(prefix: &[Card]) -> Vec<Card> {
        let mut deck = build_deck();
        for card in prefix.iter().rev() {
            let index = deck.iter().position(|candidate| candidate == card).unwrap();
            deck.remove(index);
            deck.insert(0, *card);
        }
        deck
    }

    fn card(color: Color, face: Face, copy: u8) -> Card {
        match face {
            Face::Number(value) => Card::number(color, value, copy),
            Face::DrawTwo
            | Face::DrawOne
            | Face::DrawFour
            | Face::DrawFive
            | Face::Reverse
            | Face::Skip
            | Face::SkipEveryone
            | Face::Flip
            | Face::DiscardAll
            | Face::SwapOne
            | Face::RefreshHand
            | Face::ReverseDrawTwo
            | Face::ReverseSkip
            | Face::StackOne
            | Face::StackTwo => Card::action(color, face, copy),
            Face::Wild
            | Face::DarkWild
            | Face::WildDrawTwo
            | Face::WildDrawFour
            | Face::WildDrawColor
            | Face::WildForceTrade
            | Face::WildPassHands
            | Face::WildPowerReverse
            | Face::WildNoU
            | Face::WildStackThree
            | Face::WildStackNumber
            | Face::WildReverseDrawFour
            | Face::WildDrawSix
            | Face::WildDrawTen
            | Face::WildColorRoulette => Card::wild(face, copy),
        }
    }

    #[test]
    fn deals_seven_cards_and_applies_number_start() {
        let game = GameState::new_with_deck(RuleSet::default(), 6, build_deck()).unwrap();
        assert!(game.players().iter().all(|player| player.hand().len() == 7));
        assert_eq!(game.players().len(), 6);
        assert_eq!(game.draw_pile_len(), 65);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(0));
    }

    #[test]
    fn flip_starting_card_turns_every_physical_card_to_the_dark_side() {
        let mut deck = build_flip_deck();
        let start = usize::from(RuleSet::HAND_SIZE) * 2;
        let flip = deck
            .iter()
            .position(|card| card.face() == Face::Flip)
            .unwrap();
        deck.swap(start, flip);
        let game = GameState::new_with_deck(flip_rules(), 2, deck).unwrap();
        assert_eq!(game.flip_side(), Some(FlipSide::Dark));
        assert!(
            game.players()
                .iter()
                .flat_map(PlayerState::hand)
                .all(|card| {
                    card.color().is_none_or(Color::is_dark)
                        && card
                            .opposite()
                            .is_some_and(|side| side.color().is_none_or(Color::is_light))
                })
        );
        assert!(game.top_card().color().is_none_or(Color::is_dark));
    }

    #[test]
    fn dark_side_keeps_uno_calls_and_reports_enabled() {
        fn dark_card(color: Color, face: Face, copy: u8) -> Card {
            Card::paired(
                CardSide::colored(Color::Red, Face::Number(1)),
                CardSide::colored(color, face),
                copy,
            )
            .flipped()
        }

        let playable = dark_card(Color::Pink, Face::Number(3), 0);
        let remaining = dark_card(Color::Teal, Face::Number(4), 0);
        let top = dark_card(Color::Pink, Face::Number(8), 0);

        let mut declared = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
        declared.flip_side = Some(FlipSide::Dark);
        declared.players[0].hand = vec![playable, remaining];
        declared.discard_pile = vec![top];
        declared.current_color = Some(Color::Pink);
        declared.current_player = PlayerId(0);
        assert!(matches!(
            declared.call_uno(PlayerId(0)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(0)
            })
        ));
        declared.play_card(PlayerId(0), playable, None).unwrap();
        assert!(declared.uno_exposed_players().next().is_none());

        let mut exposed = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
        exposed.flip_side = Some(FlipSide::Dark);
        exposed.players[0].hand = vec![playable, remaining];
        exposed.discard_pile = vec![top];
        exposed.current_color = Some(Color::Pink);
        exposed.current_player = PlayerId(0);
        exposed.play_card(PlayerId(0), playable, None).unwrap();
        assert_eq!(
            exposed.uno_exposed_players().collect::<Vec<_>>(),
            vec![PlayerId(0)]
        );
        assert!(matches!(
            exposed.report_uno(PlayerId(1), PlayerId(0)),
            Ok(ActionOutcome::UnoReported {
                reporter: PlayerId(1),
                target: PlayerId(0),
                ref cards,
            }) if cards.len() == 2
        ));
    }

    #[test]
    fn flipping_reverses_draw_and_discard_order_while_swapping_every_face() {
        let mut game = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
        let extra = [
            game.draw_pile.pop_front().unwrap(),
            game.draw_pile.pop_front().unwrap(),
        ];
        game.discard_pile.extend(extra);
        let old_draw = game.draw_pile.iter().copied().collect::<Vec<_>>();
        let old_discard = game.discard_pile.clone();
        game.flip_everything();
        assert_eq!(
            game.discard_pile,
            old_discard
                .into_iter()
                .rev()
                .map(Card::flipped)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            game.draw_pile.back().copied(),
            old_draw.first().copied().map(Card::flipped)
        );
    }

    #[test]
    fn flip_penalty_stacks_use_four_separate_chains() {
        let mut game = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
        game.rules.flip.action_stacking = true;
        let draw_one = flip_card(Color::Red, Face::DrawOne);
        let wild_two = Card::paired(
            CardSide::wild(Face::WildDrawTwo),
            CardSide::wild(Face::WildDrawColor),
            0,
        );
        let draw_five = flip_card(Color::Pink, Face::DrawFive);
        let wild_color = Card::paired(
            CardSide::wild(Face::WildDrawColor),
            CardSide::wild(Face::WildDrawTwo),
            0,
        );
        game.pending_kind = Some(PendingDrawKind::FlipDrawOne);
        assert!(game.stack_allowed(draw_one));
        assert!(game.stack_allowed(wild_two));
        assert!(!game.stack_allowed(draw_five));
        game.pending_kind = Some(PendingDrawKind::FlipWildDrawTwo);
        assert!(game.stack_allowed(wild_two));
        assert!(!game.stack_allowed(draw_one));
        game.pending_kind = Some(PendingDrawKind::FlipDrawFive);
        assert!(game.stack_allowed(draw_five));
        assert!(!game.stack_allowed(wild_color));
        game.pending_kind = Some(PendingDrawKind::FlipWildDrawColor);
        assert!(game.stack_allowed(wild_color));
        assert!(!game.stack_allowed(draw_five));
    }

    #[test]
    fn stacked_wild_draw_colors_resolve_each_selected_color_in_order() {
        let mut game = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
        game.pending_kind = Some(PendingDrawKind::FlipWildDrawColor);
        game.pending_draw = 2;
        game.pending_draw_colors = vec![Color::Red, Color::Blue];
        game.draw_pile = VecDeque::from(vec![
            flip_card(Color::Green, Face::Number(1)),
            flip_card(Color::Red, Face::Number(2)),
            flip_card(Color::Yellow, Face::Number(3)),
            flip_card(Color::Blue, Face::Number(4)),
            flip_card(Color::Red, Face::Number(5)),
        ]);
        let cards = game.draw_pending_penalty(PlayerId(0), 0).unwrap();
        assert_eq!(cards.len(), 4);
        assert_eq!(cards[1].color(), Some(Color::Red));
        assert_eq!(cards[3].color(), Some(Color::Blue));
    }

    #[test]
    fn identical_flip_pair_cancels_without_turning_the_table_over() {
        let mut rules = flip_rules();
        rules.flip.action_stacking = true;
        rules.flip.jump_in = true;
        let mut game = GameState::new_with_deck(rules, 2, build_flip_deck()).unwrap();
        let pair = build_flip_deck()
            .into_iter()
            .filter(|card| card.color() == Some(Color::Red) && card.face() == Face::Flip)
            .collect::<Vec<_>>();
        game.players[0].hand = pair.clone();
        game.current_player = PlayerId(0);
        game.current_color = Some(Color::Red);
        game.discard_pile = vec![flip_card(Color::Red, Face::Number(4))];
        game.play_cards(PlayerId(0), &pair, None).unwrap();
        assert_eq!(game.flip_side(), Some(FlipSide::Light));
        assert!(matches!(game.phase(), Phase::Finished(_)));
        assert!(
            game.discard_pile()
                .iter()
                .rev()
                .take(2)
                .all(|card| card.face() == Face::Flip)
        );
    }

    #[test]
    fn stacked_dark_skip_everyone_is_consumed_by_every_other_player() {
        let mut rules = flip_rules();
        rules.flip.action_stacking = true;
        let mut game = GameState::new_with_deck(rules, 3, build_flip_deck()).unwrap();
        game.flip_everything();
        let skip = build_flip_deck()
            .into_iter()
            .map(Card::flipped)
            .find(|card| card.color() == Some(Color::Pink) && card.face() == Face::SkipEveryone)
            .unwrap();
        game.players[0].hand = vec![skip];
        game.current_player = PlayerId(0);
        game.direction = Direction::Clockwise;
        game.current_color = Some(Color::Pink);
        game.discard_pile = vec![Card::paired(
            CardSide::colored(Color::Pink, Face::Number(4)),
            CardSide::colored(Color::Red, Face::Number(4)),
            120,
        )];
        game.play_card(PlayerId(0), skip, None).unwrap();
        assert!(matches!(game.phase(), Phase::Playing));
        game.resolve_skip(PlayerId(1)).unwrap();
        assert!(matches!(game.phase(), Phase::Playing));
        game.resolve_skip(PlayerId(2)).unwrap();
        assert!(matches!(game.phase(), Phase::Finished(_)));
    }

    #[test]
    fn two_to_six_players_are_supported_but_seven_are_rejected() {
        for player_count in RuleSet::MIN_PLAYERS..=RuleSet::MAX_PLAYERS {
            let game =
                GameState::new_with_deck(RuleSet::default(), player_count, build_deck()).unwrap();
            assert_eq!(game.players().len(), usize::from(player_count));
            assert!(game.players().iter().all(|player| player.hand().len() == 7));
        }
        assert!(matches!(
            GameState::new_with_deck(RuleSet::default(), 7, build_deck()),
            Err(GameError::InvalidRules(_))
        ));
    }

    #[test]
    fn action_stacking_controls_draw_two_and_draw_four_chains() {
        let p0_draw_two = card(Color::Red, Face::DrawTwo, 0);
        let p1_draw_two = card(Color::Yellow, Face::DrawTwo, 0);
        let p2_draw_four = Card::wild(Face::WildDrawFour, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        // Round-robin dealing positions are 0..=5; put the required cards explicitly.
        let mut deck = build_deck();
        for (position, required) in [
            (0, p0_draw_two),
            (1, p1_draw_two),
            (2, p2_draw_four),
            (42, start),
        ] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck.clone()).unwrap();
        game.play_card(PlayerId(0), p0_draw_two, None).unwrap();
        assert_eq!(
            game.play_card(PlayerId(1), p1_draw_two, None),
            Err(GameError::CannotStack(p1_draw_two))
        );

        let mut game = GameState::new_with_deck(
            RuleSet {
                action_stacking: true,
                ..RuleSet::default()
            },
            6,
            deck,
        )
        .unwrap();
        game.play_card(PlayerId(0), p0_draw_two, None).unwrap();
        game.play_card(PlayerId(1), p1_draw_two, None).unwrap();
        game.play_card(PlayerId(2), p2_draw_four, Some(Color::Blue))
            .unwrap();
        assert_eq!(game.turn().unwrap().pending_draw, 8);
    }

    #[test]
    fn draw_four_after_draw_two_only_considers_matching_draw_two_for_challenge() {
        let draw_two = card(Color::Red, Face::DrawTwo, 0);
        let draw_four = Card::wild(Face::WildDrawFour, 0);
        let same_color_number = card(Color::Red, Face::Number(7), 0);
        let same_color_draw_two = card(Color::Red, Face::DrawTwo, 1);
        let start = card(Color::Red, Face::Number(5), 0);

        for alternative in [same_color_number, same_color_draw_two] {
            let mut deck = build_deck();
            for (position, required) in
                [(0, draw_two), (1, draw_four), (7, alternative), (42, start)]
            {
                let current = deck
                    .iter()
                    .position(|candidate| *candidate == required)
                    .unwrap();
                deck.swap(position, current);
            }
            if alternative == same_color_number {
                for position in [1, 13, 19, 25, 31, 37] {
                    if deck[position].face() == Face::DrawTwo
                        && deck[position].color() == Some(Color::Red)
                    {
                        let replacement = (43..deck.len())
                            .find(|index| {
                                deck[*index].face() != Face::DrawTwo
                                    || deck[*index].color() != Some(Color::Red)
                            })
                            .unwrap();
                        deck.swap(position, replacement);
                    }
                }
            }

            let mut game = GameState::new_with_deck(
                RuleSet {
                    action_stacking: true,
                    ..RuleSet::default()
                },
                6,
                deck,
            )
            .unwrap();
            game.play_card(PlayerId(0), draw_two, None).unwrap();
            game.play_card(PlayerId(1), draw_four, Some(Color::Blue))
                .unwrap();
            let outcome = game.challenge_draw_four(PlayerId(2)).unwrap();
            let expected = if alternative == same_color_number {
                ChallengeResult::Failed
            } else {
                ChallengeResult::Successful
            };
            assert!(
                matches!(outcome, ActionOutcome::ChallengeResolved { result, .. } if result == expected)
            );
        }
    }

    #[test]
    fn successful_challenge_returns_whole_stack_to_first_offender() {
        let draw_four = Card::wild(Face::WildDrawFour, 0);
        let matching = card(Color::Red, Face::Number(7), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, draw_four), (6, matching), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), draw_four, Some(Color::Blue))
            .unwrap();
        let before = game.player(PlayerId(0)).unwrap().hand().len();
        let outcome = game.challenge_draw_four(PlayerId(1)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                result: ChallengeResult::Successful,
                penalized: PlayerId(0),
                ref cards,
                next_player: PlayerId(1),
                ..
            } if cards.len() == 4
        ));
        assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), before + 4);
    }

    #[test]
    fn failed_challenge_adds_two_to_the_whole_stack() {
        let draw_four = Card::wild(Face::WildDrawFour, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, draw_four), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        // Ensure player zero has no red card besides the wild.
        for position in [6, 12, 18, 24, 30, 36] {
            if deck[position].color() == Some(Color::Red) {
                let replacement = (43..deck.len())
                    .find(|index| deck[*index].color() != Some(Color::Red))
                    .unwrap();
                deck.swap(position, replacement);
            }
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), draw_four, Some(Color::Blue))
            .unwrap();
        let before = game.player(PlayerId(1)).unwrap().hand().len();
        let outcome = game.challenge_draw_four(PlayerId(1)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                result: ChallengeResult::Failed,
                penalized: PlayerId(1),
                ref cards,
                next_player: PlayerId(2),
                ..
            } if cards.len() == 6
        ));
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before + 6);
    }

    #[test]
    fn uno_may_be_called_before_or_immediately_after_the_penultimate_card() {
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, build_deck()).unwrap();
        game.players[0].hand.truncate(2);
        let playable = game.players[0]
            .hand
            .iter()
            .copied()
            .find(|card| game.matches_top(*card))
            .unwrap_or_else(|| {
                let card = *game.discard_pile.last().unwrap();
                game.players[0].hand[0] = card;
                card
            });
        assert!(game.can_call_uno(PlayerId(0)));
        game.call_uno(PlayerId(0)).unwrap();
        assert!(!game.can_call_uno(PlayerId(0)));
        assert_eq!(
            game.uno_declared_players().collect::<Vec<_>>(),
            vec![PlayerId(0)]
        );
        assert_eq!(
            game.draw_card(PlayerId(0)),
            Err(GameError::MustPlayAfterUno)
        );
        game.play_card(PlayerId(0), playable, None).unwrap();
        assert!(game.uno_exposed_players().next().is_none());

        game.current_player = PlayerId(1);
        game.players[1].hand.truncate(2);
        let playable = game.players[1]
            .hand
            .iter()
            .copied()
            .find(|card| game.matches_top(*card))
            .unwrap_or_else(|| {
                let card = *game.discard_pile.last().unwrap();
                game.players[1].hand[0] = card;
                card
            });
        game.play_card(PlayerId(1), playable, None).unwrap();
        assert_eq!(
            game.uno_exposed_players().collect::<Vec<_>>(),
            vec![PlayerId(1)]
        );
        assert!(matches!(
            game.call_uno(PlayerId(1)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(1)
            })
        ));
        assert!(game.uno_exposed_players().next().is_none());
        assert!(game.uno_declared_players().next().is_none());

        // 补喊后不能被重复检举，也不会留下“必须立刻出牌”的预喊状态。
        let before = game.player(PlayerId(1)).unwrap().hand().len();
        assert_eq!(
            game.report_uno(PlayerId(2), PlayerId(1)),
            Err(GameError::PlayerNotReportable(PlayerId(1)))
        );
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before);
    }

    #[test]
    fn color_roulette_keeps_the_uno_reaction_window_open() {
        fn prepared_game() -> GameState {
            let mut game = no_mercy_game(3);
            game.players[0].hand = vec![
                Card::wild(Face::WildColorRoulette, 0),
                card(Color::Blue, Face::Number(3), 0),
            ];
            game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
            game.current_color = Some(Color::Red);
            game.current_player = PlayerId(0);
            game
        }

        let mut recover = prepared_game();
        recover
            .play_card(PlayerId(0), Card::wild(Face::WildColorRoulette, 0), None)
            .unwrap();
        assert_eq!(
            recover.uno_exposed_players().collect::<Vec<_>>(),
            vec![PlayerId(0)]
        );
        assert!(recover.can_call_uno(PlayerId(0)));
        assert!(matches!(
            recover.call_uno(PlayerId(0)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(0)
            })
        ));

        let mut report = prepared_game();
        report
            .play_card(PlayerId(0), Card::wild(Face::WildColorRoulette, 0), None)
            .unwrap();
        assert!(matches!(
            report.report_uno(PlayerId(2), PlayerId(0)),
            Ok(ActionOutcome::UnoReported {
                reporter: PlayerId(2),
                target: PlayerId(0),
                ref cards,
            }) if cards.len() == 2
        ));
        assert_eq!(
            report.pending_swap(),
            Some(PendingSwap::ColorRoulette {
                player: PlayerId(1)
            })
        );
    }

    #[test]
    fn eliminated_players_cannot_call_or_report_uno() {
        let mut game = no_mercy_game(3);
        game.players[2].eliminated = true;
        game.uno_exposed[1] = true;

        assert_eq!(
            game.call_uno(PlayerId(2)),
            Err(GameError::PlayerEliminated(PlayerId(2)))
        );
        assert_eq!(
            game.report_uno(PlayerId(2), PlayerId(1)),
            Err(GameError::PlayerEliminated(PlayerId(2)))
        );
    }

    #[test]
    fn last_draw_four_finishes_after_the_penalty_is_resolved() {
        let wild_draw_four = Card::wild(Face::WildDrawFour, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, wild_draw_four), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.players[0].hand = vec![wild_draw_four];
        let target_hand = game.players[1].hand.len();
        let outcome = game
            .play_card(PlayerId(0), wild_draw_four, Some(Color::Blue))
            .unwrap();
        assert!(matches!(outcome, ActionOutcome::Played { .. }));
        let turn = game
            .turn()
            .expect("the final draw penalty still needs resolving");
        assert_eq!(turn.current_color, Some(Color::Blue));
        assert_eq!(turn.current_player, PlayerId(1));
        assert_eq!(turn.pending_draw, 4);
        assert_eq!(turn.challenge_offender, Some(PlayerId(0)));

        assert!(matches!(
            game.accept_draw_penalty(PlayerId(1)),
            Ok(ActionOutcome::PenaltyDrawn { ref cards, .. }) if cards.len() == 4
        ));
        assert_eq!(game.players[1].hand.len(), target_hand + 4);
        assert!(matches!(
            game.phase(),
            Phase::Finished(GameResult {
                winner: PlayerId(0),
                ..
            })
        ));
        assert_eq!(
            game.challenge_draw_four(PlayerId(1)),
            Err(GameError::GameAlreadyFinished)
        );
    }

    #[test]
    fn last_skip_finishes_after_the_target_loses_their_turn() {
        let skip = card(Color::Red, Face::Skip, 0);
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, build_deck()).unwrap();
        game.players[0].hand = vec![skip];
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        assert!(matches!(
            game.play_card(PlayerId(0), skip, None),
            Ok(ActionOutcome::Played { .. })
        ));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(1));
        assert!(matches!(game.phase(), Phase::Playing));

        assert!(matches!(
            game.resolve_skip(PlayerId(1)),
            Ok(ActionOutcome::SkipResolved {
                player: PlayerId(1),
                remaining: 0,
                ..
            })
        ));
        assert!(matches!(
            game.phase(),
            Phase::Finished(GameResult {
                winner: PlayerId(0),
                ..
            })
        ));
    }

    #[test]
    fn playable_drawn_card_may_be_played_or_passed_but_no_other_card_can() {
        let drawn = card(Color::Red, Face::Number(9), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = deck_with_prefix(&[]);
        for (position, required) in [(42, start), (43, drawn)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        let other = game.players[0].hand[0];
        let outcome = game.draw_card(PlayerId(0)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::DrewCards {
                playable: Some(card), ..
            } if card == drawn
        ));
        assert_eq!(
            game.play_card(
                PlayerId(0),
                other,
                other.face().is_wild().then_some(Color::Blue)
            ),
            Err(GameError::MustPlayDrawnCard(drawn))
        );
        game.pass_after_draw(PlayerId(0)).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    }

    #[test]
    fn uno_may_be_called_after_drawing_a_playable_card_from_one_to_two() {
        let drawn = card(Color::Red, Face::Number(9), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = deck_with_prefix(&[]);
        for (position, required) in [(42, start), (43, drawn)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.players[0].hand.truncate(1);

        assert!(matches!(
            game.draw_card(PlayerId(0)),
            Ok(ActionOutcome::DrewCards {
                cards,
                playable: Some(card),
                ..
            }) if cards == vec![drawn] && card == drawn
        ));
        assert!(matches!(
            game.call_uno(PlayerId(0)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(0)
            })
        ));
        game.play_card(PlayerId(0), drawn, None).unwrap();
        assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), 1);
        assert!(game.uno_exposed_players().next().is_none());
        assert!(game.uno_declared_players().next().is_none());
    }

    #[test]
    fn stacked_draw_fours_keep_the_first_player_responsible() {
        let first = Card::wild(Face::WildDrawFour, 0);
        let second = Card::wild(Face::WildDrawFour, 1);
        let matching = card(Color::Red, Face::Number(7), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, first), (1, second), (6, matching), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(
            RuleSet {
                action_stacking: true,
                ..RuleSet::default()
            },
            6,
            deck,
        )
        .unwrap();
        game.play_card(PlayerId(0), first, Some(Color::Blue))
            .unwrap();
        game.play_card(PlayerId(1), second, Some(Color::Green))
            .unwrap();
        let outcome = game.challenge_draw_four(PlayerId(2)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                offender: PlayerId(0),
                result: ChallengeResult::Successful,
                penalized: PlayerId(0),
                ref cards,
                ..
            } if cards.len() == 8
        ));
    }

    #[test]
    fn skip_stacking_transfers_multiple_blocked_turns() {
        let first = card(Color::Red, Face::Skip, 0);
        let second = card(Color::Yellow, Face::Skip, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, first), (1, second), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(
            RuleSet {
                action_stacking: true,
                skip_draw_penalty: true,
                ..RuleSet::default()
            },
            6,
            deck,
        )
        .unwrap();
        game.play_card(PlayerId(0), first, None).unwrap();
        assert_eq!(game.turn().unwrap().pending_skip, 1);
        game.play_card(PlayerId(1), second, None).unwrap();
        assert_eq!(game.turn().unwrap().pending_skip, 2);
        let before = game.player(PlayerId(2)).unwrap().hand().len();
        let outcome = game.resolve_skip(PlayerId(2)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::SkipResolved {
                player: PlayerId(2),
                remaining: 1,
                ref cards,
                next_player: PlayerId(3),
            } if cards.len() == 1
        ));
        assert_eq!(game.skipped_turns(PlayerId(2)), Some(1));
        assert_eq!(game.player(PlayerId(2)).unwrap().hand().len(), before + 1);

        game.current_player = PlayerId(2);
        game.resolve_skip(PlayerId(2)).unwrap();
        assert_eq!(game.skipped_turns(PlayerId(2)), Some(0));
        assert_eq!(game.player(PlayerId(2)).unwrap().hand().len(), before + 2);
    }

    #[test]
    fn ordinary_skip_blocks_exactly_one_turn() {
        let skip = card(Color::Red, Face::Skip, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, skip), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), skip, None).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(1));
        assert_eq!(game.draw_card(PlayerId(1)), Err(GameError::MustResolveSkip));
        game.resolve_skip(PlayerId(1)).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(2));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(0));
    }

    #[test]
    fn draw_two_is_paid_by_the_skipped_direct_next_player_before_skip_resolves() {
        let draw_two = card(Color::Red, Face::DrawTwo, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, draw_two), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.skip_turns[1] = 1;
        let before = game.player(PlayerId(1)).unwrap().hand().len();

        game.play_card(PlayerId(0), draw_two, None).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(
            game.resolve_skip(PlayerId(1)),
            Err(GameError::MustResolveDrawPenalty)
        );

        let outcome = game.accept_draw_penalty(PlayerId(1)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::PenaltyDrawn {
                player: PlayerId(1),
                ref cards,
                next_player: PlayerId(1),
            } if cards.len() == 2
        ));
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before + 2);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(game.turn().unwrap().pending_draw, 0);
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(1));

        game.resolve_skip(PlayerId(1)).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(2));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(0));
    }

    fn jump_in_game() -> GameState {
        let mut game = GameState::new_with_deck(
            RuleSet {
                action_stacking: true,
                jump_in: true,
                ..RuleSet::default()
            },
            3,
            build_deck(),
        )
        .unwrap();
        game.direction = Direction::Clockwise;
        game
    }

    #[test]
    fn non_next_player_can_jump_in_and_then_recover_uno_at_one_card() {
        let top = card(Color::Red, Face::Number(7), 0);
        let matching = card(Color::Red, Face::Number(7), 1);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = jump_in_game();
        game.discard_pile = vec![top];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(1);
        game.jump_in_open = true;
        game.players[2].hand = vec![matching, filler];

        assert_eq!(game.jump_in_card(PlayerId(1)), None);
        assert_eq!(game.jump_in_card(PlayerId(2)), Some(matching));
        assert!(matches!(
            game.jump_in(PlayerId(2), matching),
            Ok(ActionOutcome::Played {
                player: PlayerId(2),
                next_player: PlayerId(0),
                ..
            })
        ));
        assert_eq!(game.player(PlayerId(2)).unwrap().hand(), &[filler]);
        assert_eq!(
            game.uno_exposed_players().collect::<Vec<_>>(),
            vec![PlayerId(2)]
        );
        assert!(matches!(
            game.call_uno(PlayerId(2)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(2)
            })
        ));
    }

    #[test]
    fn jump_in_without_action_stacking_only_offers_number_cards() {
        let number_top = card(Color::Red, Face::Number(7), 0);
        let number_match = card(Color::Red, Face::Number(7), 1);
        let reverse_top = card(Color::Red, Face::Reverse, 0);
        let reverse_match = card(Color::Red, Face::Reverse, 1);
        let mut game = GameState::new_with_deck(
            RuleSet {
                jump_in: true,
                ..RuleSet::default()
            },
            3,
            build_deck(),
        )
        .unwrap();
        game.current_player = PlayerId(1);
        game.jump_in_open = true;
        game.discard_pile = vec![number_top];
        game.current_color = Some(Color::Red);
        game.players[2].hand = vec![number_match, reverse_match];

        assert_eq!(game.jump_in_card(PlayerId(2)), Some(number_match));

        game.discard_pile = vec![reverse_top];
        assert_eq!(game.jump_in_card(PlayerId(2)), None);
    }

    #[test]
    fn current_players_first_successful_action_closes_the_jump_window() {
        let top = card(Color::Red, Face::Number(7), 0);
        let matching = card(Color::Red, Face::Number(7), 1);
        let mut game = jump_in_game();
        game.discard_pile = vec![top];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(1);
        game.jump_in_open = true;
        game.players[2].hand = vec![matching, card(Color::Blue, Face::Number(3), 0)];

        assert_eq!(game.jump_in_card(PlayerId(2)), Some(matching));
        game.draw_card(PlayerId(1)).unwrap();
        assert_eq!(game.jump_in_card(PlayerId(2)), None);
        assert_eq!(
            game.jump_in(PlayerId(2), matching),
            Err(GameError::CannotJumpIn)
        );
    }

    #[test]
    fn jumped_draw_two_and_skip_extend_the_existing_stack() {
        let filler = card(Color::Blue, Face::Number(3), 0);

        let draw_top = card(Color::Red, Face::DrawTwo, 0);
        let draw_match = card(Color::Red, Face::DrawTwo, 1);
        let mut draw_game = jump_in_game();
        draw_game.discard_pile = vec![draw_top];
        draw_game.current_color = Some(Color::Red);
        draw_game.current_player = PlayerId(1);
        draw_game.pending_draw = 2;
        draw_game.pending_kind = Some(PendingDrawKind::DrawTwo);
        draw_game.jump_in_open = true;
        draw_game.players[2].hand = vec![draw_match, filler];
        draw_game.jump_in(PlayerId(2), draw_match).unwrap();
        assert_eq!(draw_game.turn().unwrap().pending_draw, 4);
        assert_eq!(draw_game.turn().unwrap().current_player, PlayerId(0));

        let skip_top = card(Color::Yellow, Face::Skip, 0);
        let skip_match = card(Color::Yellow, Face::Skip, 1);
        let mut skip_game = jump_in_game();
        skip_game.discard_pile = vec![skip_top];
        skip_game.current_color = Some(Color::Yellow);
        skip_game.current_player = PlayerId(1);
        skip_game.pending_skip = 1;
        skip_game.jump_in_open = true;
        skip_game.players[2].hand = vec![skip_match, filler];
        skip_game.jump_in(PlayerId(2), skip_match).unwrap();
        assert_eq!(skip_game.turn().unwrap().pending_skip, 2);
        assert_eq!(skip_game.turn().unwrap().current_player, PlayerId(0));
    }

    #[test]
    fn identical_pair_is_atomic_and_two_reverses_restore_direction() {
        let first = card(Color::Red, Face::Reverse, 0);
        let second = card(Color::Red, Face::Reverse, 1);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = jump_in_game();
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);
        game.players[0].hand = vec![first, second, filler];

        assert!(matches!(
            game.play_cards(PlayerId(0), &[first, second], None),
            Ok(ActionOutcome::Played {
                player: PlayerId(0),
                next_player: PlayerId(1),
                ..
            })
        ));
        assert_eq!(game.direction(), Direction::Clockwise);
        assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[filler]);
        assert_eq!(
            &game.discard_pile()[game.discard_pile().len() - 2..],
            &[first, second]
        );
    }

    #[test]
    fn identical_last_pair_finishes_together_but_declared_uno_forces_one_card() {
        let first = card(Color::Green, Face::Number(8), 0);
        let second = card(Color::Green, Face::Number(8), 1);
        let mut game = jump_in_game();
        game.discard_pile = vec![card(Color::Green, Face::Number(4), 0)];
        game.current_color = Some(Color::Green);
        game.current_player = PlayerId(0);
        game.players[0].hand = vec![first, second];
        assert!(matches!(
            game.play_cards(PlayerId(0), &[first, second], None),
            Ok(ActionOutcome::Played {
                player: PlayerId(0),
                ..
            })
        ));
        assert!(matches!(game.phase(), Phase::Finished(_)));

        let mut declared = jump_in_game();
        declared.discard_pile = vec![card(Color::Green, Face::Number(4), 0)];
        declared.current_color = Some(Color::Green);
        declared.current_player = PlayerId(0);
        declared.players[0].hand = vec![first, second];
        declared.uno_declared[0] = true;
        assert_eq!(
            declared.play_cards(PlayerId(0), &[first, second], None),
            Err(GameError::CannotPlayTogether)
        );
        assert_eq!(
            declared.player(PlayerId(0)).unwrap().hand(),
            &[first, second]
        );
    }

    fn swap_pack_game() -> GameState {
        let rules = RuleSet {
            swap_pack: true,
            ..RuleSet::default()
        };
        GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
    }

    fn reverse_pack_game() -> GameState {
        let rules = RuleSet {
            reverse_pack: true,
            ..RuleSet::default()
        };
        GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
    }

    fn stack_pack_game() -> GameState {
        let rules = RuleSet {
            stack_pack: true,
            ..RuleSet::default()
        };
        GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
    }

    #[test]
    fn colored_stack_cards_require_the_current_color_and_accumulate() {
        let stack_one = card(Color::Red, Face::StackOne, 0);
        let matching_stack_two = card(Color::Red, Face::StackTwo, 0);
        let wrong_stack_two = card(Color::Blue, Face::StackTwo, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = stack_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![stack_one, filler];
        game.players[1].hand = vec![matching_stack_two, wrong_stack_two, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), stack_one, None).unwrap();
        assert!(!game.can_play(PlayerId(1), wrong_stack_two));
        assert!(game.can_play(PlayerId(1), matching_stack_two));
        game.play_card(PlayerId(1), matching_stack_two, None)
            .unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.pending_draw, 3);
        assert_eq!(turn.pending_kind, Some(PendingDrawKind::Stack));
        assert_eq!(turn.pending_draw_source, Some(PlayerId(1)));
        assert_eq!(turn.current_player, PlayerId(2));
    }

    #[test]
    fn same_stack_face_does_not_bypass_its_color_requirement() {
        let blue_stack = card(Color::Blue, Face::StackOne, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = stack_pack_game();
        game.players[0].hand = vec![blue_stack, filler];
        game.discard_pile = vec![card(Color::Red, Face::StackOne, 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        assert!(!game.can_play(PlayerId(0), blue_stack));
        assert_eq!(
            game.play_card(PlayerId(0), blue_stack, None),
            Err(GameError::CardDoesNotMatch)
        );
    }

    #[test]
    fn wild_stack_number_reveals_to_a_number_below_the_discard_top() {
        let stack_number = Card::wild(Face::WildStackNumber, 0);
        let revealed_action = card(Color::Blue, Face::Skip, 0);
        let revealed_number = card(Color::Yellow, Face::Number(6), 0);
        let old_top = card(Color::Red, Face::Number(7), 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = stack_pack_game();
        game.players[0].hand = vec![stack_number, filler];
        game.discard_pile = vec![old_top];
        game.draw_pile = VecDeque::from([revealed_action, revealed_number, filler]);
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        let outcome = game
            .play_card(PlayerId(0), stack_number, Some(Color::Blue))
            .unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::Played {
                effect: Some(PlayedEffect::StackNumberRevealed { ref cards, value: 6 }),
                ..
            } if cards == &[revealed_action, revealed_number]
        ));
        assert_eq!(
            game.discard_pile(),
            &[revealed_action, revealed_number, old_top, stack_number]
        );
        let turn = game.turn().unwrap();
        assert_eq!(turn.pending_draw, 6);
        assert_eq!(turn.pending_kind, Some(PendingDrawKind::Stack));
        assert_eq!(turn.current_color, Some(Color::Blue));
    }

    #[test]
    fn stack_number_zero_is_still_an_active_penalty_turn() {
        let stack_number = Card::wild(Face::WildStackNumber, 0);
        let zero = card(Color::Yellow, Face::Number(0), 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = stack_pack_game();
        game.players[0].hand = vec![stack_number, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.draw_pile = VecDeque::from([zero, filler]);
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), stack_number, Some(Color::Yellow))
            .unwrap();
        assert_eq!(game.turn().unwrap().pending_draw, 0);
        assert_eq!(
            game.turn().unwrap().pending_kind,
            Some(PendingDrawKind::Stack)
        );
        assert_eq!(
            game.draw_card(PlayerId(1)),
            Err(GameError::MustResolveDrawPenalty)
        );
        assert!(matches!(
            game.accept_draw_penalty(PlayerId(1)),
            Ok(ActionOutcome::PenaltyDrawn {
                ref cards,
                next_player: PlayerId(2),
                ..
            }) if cards.is_empty()
        ));
    }

    #[test]
    fn stack_cards_preserve_the_first_draw_four_challenge_offender() {
        let draw_four = Card::wild(Face::WildDrawFour, 0);
        let matching = card(Color::Blue, Face::Number(7), 0);
        let stack_one = card(Color::Red, Face::StackOne, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = stack_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![draw_four, matching];
        game.players[1].hand = vec![stack_one, filler];
        game.discard_pile = vec![card(Color::Blue, Face::Number(5), 0)];
        game.current_color = Some(Color::Blue);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), draw_four, Some(Color::Red))
            .unwrap();
        game.play_card(PlayerId(1), stack_one, None).unwrap();
        let outcome = game.challenge_draw_four(PlayerId(2)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                offender: PlayerId(0),
                result: ChallengeResult::Successful,
                penalized: PlayerId(0),
                ref cards,
                ..
            } if cards.len() == 5
        ));
    }

    #[test]
    fn last_stack_card_finishes_after_applying_its_penalty() {
        let stack_two = card(Color::Red, Face::StackTwo, 0);
        let mut game = stack_pack_game();
        game.players[0].hand = vec![stack_two];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        assert!(matches!(
            game.play_card(PlayerId(0), stack_two, None),
            Ok(ActionOutcome::Played { .. })
        ));
        assert_eq!(game.turn().unwrap().pending_draw, 2);
        let before = game.players[1].hand.len();
        assert!(matches!(
            game.accept_draw_penalty(PlayerId(1)),
            Ok(ActionOutcome::PenaltyDrawn { ref cards, .. }) if cards.len() == 2
        ));
        assert_eq!(game.players[1].hand.len(), before + 2);
        assert!(matches!(
            game.phase(),
            Phase::Finished(GameResult {
                winner: PlayerId(0),
                ..
            })
        ));
    }

    #[test]
    fn reverse_draw_two_matches_reverse_and_redirects_the_penalty() {
        let reverse_draw = card(Color::Red, Face::ReverseDrawTwo, 0);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.players[0].hand = vec![reverse_draw, filler];
        game.discard_pile = vec![card(Color::Blue, Face::Reverse, 0)];
        game.current_color = Some(Color::Blue);
        game.current_player = PlayerId(0);

        assert!(game.can_play(PlayerId(0), reverse_draw));
        game.play_card(PlayerId(0), reverse_draw, None).unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.pending_draw, 2);
        assert_eq!(turn.pending_kind, Some(PendingDrawKind::DrawTwo));
        assert_eq!(turn.pending_draw_source, Some(PlayerId(0)));
        assert_eq!(turn.current_player, PlayerId(5));
    }

    #[test]
    fn reverse_skip_changes_direction_before_choosing_the_skipped_player() {
        let reverse_skip = card(Color::Red, Face::ReverseSkip, 0);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.players[0].hand = vec![reverse_skip, filler];
        game.discard_pile = vec![card(Color::Blue, Face::Skip, 0)];
        game.current_color = Some(Color::Blue);
        game.current_player = PlayerId(0);

        assert!(game.can_play(PlayerId(0), reverse_skip));
        game.play_card(PlayerId(0), reverse_skip, None).unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.current_player, PlayerId(5));
        assert_eq!(game.skipped_turns(PlayerId(5)), Some(1));
    }

    #[test]
    fn power_reverse_changes_color_and_gives_the_actor_another_turn() {
        let power = Card::wild(Face::WildPowerReverse, 0);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.players[0].hand = vec![power, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), power, Some(Color::Green))
            .unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.current_color, Some(Color::Green));
        assert_eq!(turn.current_player, PlayerId(0));
    }

    #[test]
    fn no_u_reflects_the_whole_penalty_to_the_latest_stacker() {
        let draw_a = card(Color::Red, Face::DrawTwo, 0);
        let draw_b = card(Color::Blue, Face::DrawTwo, 0);
        let no_u = Card::wild(Face::WildNoU, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![draw_a, filler];
        game.players[1].hand = vec![draw_b, filler];
        game.players[2].hand = vec![no_u, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), draw_a, None).unwrap();
        game.play_card(PlayerId(1), draw_b, None).unwrap();
        let before = game.player(PlayerId(1)).unwrap().hand().len();
        let outcome = game
            .play_card(PlayerId(2), no_u, Some(Color::Yellow))
            .unwrap();

        assert!(matches!(
            outcome,
            ActionOutcome::Played {
                effect: Some(PlayedEffect::DrawReflected {
                    player: PlayerId(1),
                    ref cards,
                }),
                ..
            } if cards.len() == 4
        ));
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before + 4);
        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.current_color, Some(Color::Yellow));
        assert_eq!(turn.pending_draw, 0);
        assert_eq!(turn.pending_draw_source, None);
        assert_eq!(turn.challenge_offender, None);
        assert_eq!(turn.current_player, PlayerId(0));
    }

    #[test]
    fn reverse_skip_can_redirect_an_accumulated_skip() {
        let skip = card(Color::Red, Face::Skip, 0);
        let reverse_skip = card(Color::Blue, Face::ReverseSkip, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![skip, filler];
        game.players[1].hand = vec![reverse_skip, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), skip, None).unwrap();
        assert!(game.can_play(PlayerId(1), reverse_skip));
        game.play_card(PlayerId(1), reverse_skip, None).unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.pending_skip, 2);
        assert_eq!(turn.current_player, PlayerId(0));
    }

    #[test]
    fn ordinary_skip_can_stack_on_reverse_skip() {
        let reverse_skip = card(Color::Red, Face::ReverseSkip, 0);
        let skip = card(Color::Blue, Face::Skip, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![reverse_skip, filler];
        game.players[5].hand = vec![skip, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), reverse_skip, None).unwrap();
        assert!(game.can_play(PlayerId(5), skip));
        game.play_card(PlayerId(5), skip, None).unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.pending_skip, 2);
        assert_eq!(turn.current_player, PlayerId(4));
    }

    #[test]
    fn ordinary_draw_two_can_stack_on_reverse_draw_two() {
        let reverse_draw = card(Color::Red, Face::ReverseDrawTwo, 0);
        let draw_two = card(Color::Blue, Face::DrawTwo, 0);
        let filler = card(Color::Green, Face::Number(3), 0);
        let mut game = reverse_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![reverse_draw, filler];
        game.players[5].hand = vec![draw_two, filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(7), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), reverse_draw, None).unwrap();
        assert!(game.can_play(PlayerId(5), draw_two));
        game.play_card(PlayerId(5), draw_two, None).unwrap();

        let turn = game.turn().unwrap();
        assert_eq!(turn.direction, Direction::CounterClockwise);
        assert_eq!(turn.pending_draw, 4);
        assert_eq!(turn.pending_draw_source, Some(PlayerId(5)));
        assert_eq!(turn.current_player, PlayerId(4));
    }

    #[test]
    fn last_no_u_reflects_the_pending_penalty_before_winning() {
        let no_u = Card::wild(Face::WildNoU, 0);
        let mut game = reverse_pack_game();
        game.rules.action_stacking = true;
        game.players[0].hand = vec![no_u];
        game.discard_pile = vec![card(Color::Red, Face::DrawTwo, 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);
        game.pending_draw = 2;
        game.pending_kind = Some(PendingDrawKind::DrawTwo);
        game.pending_draw_source = Some(PlayerId(5));
        let source_hand_len = game.player(PlayerId(5)).unwrap().hand().len();

        assert!(matches!(
            game.play_card(PlayerId(0), no_u, Some(Color::Blue)),
            Ok(ActionOutcome::Played {
                effect: Some(PlayedEffect::DrawReflected {
                    player: PlayerId(5),
                    ref cards,
                }),
                ..
            }) if cards.len() == 2
        ));
        assert_eq!(
            game.player(PlayerId(5)).unwrap().hand().len(),
            source_hand_len + 2
        );
        assert!(matches!(
            game.phase(),
            Phase::Finished(GameResult {
                winner: PlayerId(0),
                ..
            })
        ));
    }

    #[test]
    fn refresh_hand_places_old_cards_under_the_discard_and_draws_the_same_count() {
        let refresh = card(Color::Red, Face::RefreshHand, 0);
        let first = card(Color::Blue, Face::Number(1), 0);
        let second = card(Color::Green, Face::Number(2), 0);
        let top = card(Color::Red, Face::Number(5), 0);
        let mut game = swap_pack_game();
        game.players[0].hand = vec![refresh, first, second];
        game.discard_pile = vec![top];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);
        game.rules.action_stacking = true;
        game.rules.jump_in = true;

        let outcome = game.play_card(PlayerId(0), refresh, None).unwrap();

        assert!(matches!(
            outcome,
            ActionOutcome::Played {
                effect: Some(PlayedEffect::HandRefreshed { count: 2 }),
                ..
            }
        ));
        assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), 2);
        assert_eq!(game.discard_pile(), &[first, second, top, refresh]);
        assert_eq!(game.top_card(), refresh);
        assert!(!game.jump_in_open);
    }

    #[test]
    fn swap_one_allows_returning_the_taken_card_without_creating_uno_state() {
        let swap = card(Color::Red, Face::SwapOne, 0);
        let filler = card(Color::Blue, Face::Number(1), 0);
        let taken = card(Color::Yellow, Face::Number(4), 0);
        let target_filler = card(Color::Green, Face::Number(6), 0);
        let mut game = swap_pack_game();
        game.players[0].hand = vec![swap, filler];
        game.players[1].hand = vec![taken, target_filler];
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), swap, None).unwrap();
        game.choose_swap_one_target(PlayerId(0), PlayerId(1), 0)
            .unwrap();
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), 1);
        assert!(game.uno_exposed_players().next().is_none());
        assert!(game.player(PlayerId(0)).unwrap().hand().contains(&taken));

        game.give_swap_one_card(PlayerId(0), taken).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), 2);
        assert!(game.player(PlayerId(1)).unwrap().hand().contains(&taken));
        assert!(
            !game
                .uno_exposed_players()
                .any(|player| player == PlayerId(1))
        );
    }

    #[test]
    fn force_trade_may_include_actor_and_defers_color_until_after_confirmation() {
        let trade = Card::wild(Face::WildForceTrade, 0);
        let actor_card = card(Color::Blue, Face::Number(1), 0);
        let target_card = card(Color::Yellow, Face::Number(2), 0);
        let mut game = swap_pack_game();
        game.players[0].hand = vec![trade, actor_card];
        game.players[1].hand = vec![target_card];
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);
        game.uno_exposed[1] = true;

        game.play_card(PlayerId(0), trade, None).unwrap();
        assert_eq!(game.current_color(), None);
        assert_eq!(
            game.choose_initial_color(PlayerId(0), Color::Green),
            Err(GameError::MustResolveSwapEffect)
        );
        game.force_trade_hands(PlayerId(0), PlayerId(0), PlayerId(1))
            .unwrap();
        assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[target_card]);
        assert_eq!(game.player(PlayerId(1)).unwrap().hand(), &[actor_card]);
        assert!(game.uno_exposed_players().next().is_none());

        game.choose_initial_color(PlayerId(0), Color::Green)
            .unwrap();
        assert_eq!(game.current_color(), Some(Color::Green));
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    }

    #[test]
    fn pass_hands_follows_direction_and_clears_every_uno_state() {
        let pass = Card::wild(Face::WildPassHands, 0);
        let mut game = swap_pack_game();
        let markers = [
            card(Color::Red, Face::Number(1), 0),
            card(Color::Yellow, Face::Number(2), 0),
            card(Color::Green, Face::Number(3), 0),
            card(Color::Blue, Face::Number(4), 0),
            card(Color::Red, Face::Number(5), 0),
            card(Color::Yellow, Face::Number(6), 0),
        ];
        game.players[0].hand = vec![pass, markers[0]];
        for (index, marker) in markers.iter().copied().enumerate().skip(1) {
            game.players[index].hand = vec![marker];
            game.uno_declared[index] = true;
            game.uno_exposed[index] = true;
        }
        game.discard_pile = vec![card(Color::Red, Face::Number(9), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), pass, None).unwrap();

        for (source, marker) in markers.iter().copied().enumerate() {
            let target = (source + 1) % markers.len();
            assert_eq!(game.player(PlayerId(target)).unwrap().hand(), &[marker]);
        }
        assert!(game.uno_declared_players().next().is_none());
        assert!(game.uno_exposed_players().next().is_none());
        assert_eq!(
            game.pending_swap(),
            Some(PendingSwap::ChooseColor {
                player: PlayerId(0)
            })
        );
    }

    #[test]
    fn no_mercy_zero_pass_skips_eliminated_players() {
        let zero = card(Color::Red, Face::Number(0), 0);
        let markers = [
            card(Color::Blue, Face::Number(1), 0),
            card(Color::Green, Face::Number(2), 0),
            card(Color::Yellow, Face::Number(3), 0),
        ];
        let mut game = no_mercy_game(4);
        game.players[0].hand = vec![zero, markers[0]];
        game.players[1].hand = vec![markers[1]];
        game.players[2].hand.clear();
        game.players[2].eliminated = true;
        game.players[3].hand = vec![markers[2]];
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), zero, None).unwrap();

        assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[markers[2]]);
        assert_eq!(game.player(PlayerId(1)).unwrap().hand(), &[markers[0]]);
        assert!(game.player(PlayerId(2)).unwrap().hand().is_empty());
        assert_eq!(game.player(PlayerId(3)).unwrap().hand(), &[markers[1]]);
    }

    #[test]
    fn hand_trade_targets_cannot_be_eliminated() {
        let mut game = swap_pack_game();
        game.pending_swap = Some(PendingSwapState::ForceTrade {
            player: PlayerId(0),
        });
        game.current_player = PlayerId(0);
        game.players[2].hand.clear();
        game.players[2].eliminated = true;

        assert_eq!(
            game.force_trade_hands(PlayerId(0), PlayerId(1), PlayerId(2)),
            Err(GameError::InvalidSwapTargets)
        );
    }

    #[test]
    fn last_swap_pack_card_runs_its_effect_before_winning() {
        let refresh = card(Color::Red, Face::RefreshHand, 0);
        let top = card(Color::Red, Face::Number(5), 0);
        let mut game = swap_pack_game();
        game.players[0].hand = vec![refresh];
        game.discard_pile = vec![top];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        assert!(matches!(
            game.play_card(PlayerId(0), refresh, None),
            Ok(ActionOutcome::Played {
                effect: Some(PlayedEffect::HandRefreshed { count: 0 }),
                ..
            })
        ));
        assert_eq!(game.discard_pile(), &[top, refresh]);
        assert_eq!(game.pending_swap(), None);
        assert!(matches!(
            game.phase(),
            Phase::Finished(GameResult {
                winner: PlayerId(0),
                ..
            })
        ));
    }

    #[test]
    fn swap_pack_starting_cards_do_not_run_effects_and_wild_still_chooses_color() {
        let rules = RuleSet {
            swap_pack: true,
            ..RuleSet::default()
        };
        let colored = card(Color::Red, Face::SwapOne, 0);
        let mut deck = build_deck_for_rules(rules);
        let index = deck.iter().position(|card| *card == colored).unwrap();
        deck.swap(42, index);
        let game = GameState::new_with_deck(rules, 6, deck).unwrap();
        assert_eq!(game.top_card(), colored);
        assert_eq!(game.current_color(), Some(Color::Red));
        assert_eq!(game.pending_swap(), None);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(0));

        let wild = Card::wild(Face::WildForceTrade, 0);
        let mut deck = build_deck_for_rules(rules);
        let index = deck.iter().position(|card| *card == wild).unwrap();
        deck.swap(42, index);
        let mut game = GameState::new_with_deck(rules, 6, deck).unwrap();
        assert_eq!(game.top_card(), wild);
        assert_eq!(game.current_color(), None);
        assert_eq!(game.pending_swap(), None);
        game.choose_initial_color(PlayerId(0), Color::Blue).unwrap();
        assert_eq!(game.current_color(), Some(Color::Blue));
        assert_eq!(game.turn().unwrap().current_player, PlayerId(0));
    }

    #[test]
    fn no_mercy_draw_stack_only_accepts_an_equal_or_higher_value() {
        let draw_four = card(Color::Red, Face::DrawFour, 0);
        let draw_ten = Card::wild(Face::WildDrawTen, 0);
        let mut game = no_mercy_game(3);
        game.players[0].hand = vec![draw_four, draw_ten];
        game.discard_pile = vec![card(Color::Red, Face::DrawTwo, 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);
        game.pending_draw = 6;
        game.pending_kind = Some(PendingDrawKind::NoMercy(6));
        game.pending_draw_source = Some(PlayerId(2));

        assert!(!game.can_play(PlayerId(0), draw_four));
        assert!(game.can_play(PlayerId(0), draw_ten));
    }

    #[test]
    fn no_mercy_seven_swaps_the_actors_hand_with_one_target() {
        let seven = card(Color::Red, Face::Number(7), 0);
        let own = card(Color::Blue, Face::Number(1), 0);
        let target = card(Color::Green, Face::Number(2), 0);
        let mut game = no_mercy_game(3);
        game.players[0].hand = vec![seven, own];
        game.players[1].hand = vec![target];
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        game.play_card(PlayerId(0), seven, None).unwrap();
        assert_eq!(
            game.pending_swap(),
            Some(PendingSwap::SevenSwap {
                player: PlayerId(0)
            })
        );
        game.choose_seven_swap_target(PlayerId(0), PlayerId(1))
            .unwrap();
        assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[target]);
        assert_eq!(game.player(PlayerId(1)).unwrap().hand(), &[own]);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    }

    #[test]
    fn no_mercy_draws_until_a_playable_card() {
        let miss = card(Color::Blue, Face::Number(2), 0);
        let playable = card(Color::Red, Face::Number(3), 0);
        let mut game = no_mercy_game(3);
        game.players[0].hand = vec![card(Color::Green, Face::Number(1), 0)];
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.draw_pile = [miss, playable].into();
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);

        assert!(matches!(
            game.draw_card(PlayerId(0)),
            Ok(ActionOutcome::DrewCards {
                cards,
                playable: Some(card),
                next_player: PlayerId(0),
                ..
            }) if cards == vec![miss, playable] && card == playable
        ));
        assert_eq!(
            game.pass_after_draw(PlayerId(0)),
            Err(GameError::MustPlayDrawnCard(playable))
        );
    }

    #[test]
    fn no_mercy_eliminates_a_player_at_twenty_five_cards() {
        let mut game = no_mercy_game(3);
        game.players[0].hand = build_no_mercy_deck().into_iter().take(24).collect();
        game.current_player = PlayerId(0);
        game.pending_draw = 1;
        game.pending_kind = Some(PendingDrawKind::NoMercy(2));
        game.pending_draw_source = Some(PlayerId(2));

        game.accept_draw_penalty(PlayerId(0)).unwrap();
        assert!(game.player(PlayerId(0)).unwrap().eliminated());
        assert!(game.player(PlayerId(0)).unwrap().hand().is_empty());
        assert_eq!(game.elimination_order, vec![PlayerId(0)]);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    }

    #[test]
    fn no_mercy_eliminations_take_last_places_without_hand_scores() {
        let mut game = no_mercy_game(5);
        game.players[0].hand.clear();
        game.players[1].eliminated = true;
        game.players[1].hand.clear();
        game.elimination_order.push(PlayerId(1));
        game.players[3].eliminated = true;
        game.players[3].hand.clear();
        game.elimination_order.push(PlayerId(3));
        game.players[2].hand = vec![card(Color::Red, Face::Number(5), 0)];
        game.players[4].hand = vec![card(Color::Red, Face::DrawTwo, 0)];

        let result = game.finish(PlayerId(0));

        assert_eq!(result.hand_scores, vec![0, 0, 5, 0, 20]);
        assert_eq!(result.placements, vec![1, 5, 2, 4, 3]);
        assert_eq!(result.reference_deltas, vec![4, -3, 1, -2, 0]);
    }
}
