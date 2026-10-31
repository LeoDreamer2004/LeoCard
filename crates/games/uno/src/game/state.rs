use crate::{
    UnoCard, UnoColor, UnoFace, UnoFlipSide, UnoRuleError, UnoRuleSet, build_deck_for_rules,
};
use leocard_game_common::{
    DeckError, PlayerId, has_unique_cards, validate_deck as validate_physical_deck,
};
use std::{collections::VecDeque, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnoPlayerState {
    pub(super) id: PlayerId,
    pub(super) hand: Vec<UnoCard>,
    pub(super) eliminated: bool,
}

impl UnoPlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[UnoCard] {
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
pub enum UnoDirection {
    Clockwise,
    CounterClockwise,
}

impl UnoDirection {
    pub(super) const fn reversed(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnoPendingDrawKind {
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
pub struct UnoTurnState {
    pub current_player: PlayerId,
    pub direction: UnoDirection,
    pub current_color: Option<UnoColor>,
    pub top_card: UnoCard,
    pub pending_draw: u16,
    pub pending_kind: Option<UnoPendingDrawKind>,
    pub pending_draw_source: Option<PlayerId>,
    pub challenge_offender: Option<PlayerId>,
    pub drawn_card: Option<UnoCard>,
    pub pending_skip: u16,
    pub skipped_turns_remaining: u16,
    pub pending_swap: Option<UnoPendingSwap>,
    pub sudden_death_free_play: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnoPendingSwap {
    SwapOneTarget { player: PlayerId },
    SwapOneGive { player: PlayerId, target: PlayerId },
    ForceTrade { player: PlayerId },
    ChooseColor { player: PlayerId },
    SevenSwap { player: PlayerId },
    ColorRoulette { player: PlayerId },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PendingSwapState {
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
    pub(super) const fn public(self) -> UnoPendingSwap {
        match self {
            Self::SwapOneTarget { player, .. } => UnoPendingSwap::SwapOneTarget { player },
            Self::SwapOneGive { player, target, .. } => {
                UnoPendingSwap::SwapOneGive { player, target }
            }
            Self::ForceTrade { player } => UnoPendingSwap::ForceTrade { player },
            Self::ChooseColor { player } => UnoPendingSwap::ChooseColor { player },
            Self::SevenSwap { player } => UnoPendingSwap::SevenSwap { player },
            Self::ColorRoulette { player } => UnoPendingSwap::ColorRoulette { player },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnoPlayedEffect {
    HandRefreshed {
        count: u16,
    },
    HandsPassed {
        direction: UnoDirection,
    },
    DrawReflected {
        player: PlayerId,
        cards: Vec<UnoCard>,
    },
    StackNumberRevealed {
        cards: Vec<UnoCard>,
        value: u8,
    },
    CardsDiscarded {
        cards: Vec<UnoCard>,
    },
    Flipped {
        side: UnoFlipSide,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnoGameResult {
    pub winner: PlayerId,
    pub hand_scores: Vec<u16>,
    pub placements: Vec<u8>,
    pub reference_deltas: Vec<i16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnoPhase {
    Playing,
    Finished(UnoGameResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnoChallengeResult {
    Successful,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnoActionOutcome {
    ColorChosen {
        player: PlayerId,
        color: UnoColor,
    },
    Played {
        player: PlayerId,
        card: UnoCard,
        next_player: PlayerId,
        effect: Option<UnoPlayedEffect>,
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
        cards: Vec<UnoCard>,
        playable: Option<UnoCard>,
        next_player: PlayerId,
    },
    PassedAfterDraw {
        player: PlayerId,
        next_player: PlayerId,
    },
    PenaltyDrawn {
        player: PlayerId,
        cards: Vec<UnoCard>,
        next_player: PlayerId,
    },
    ChallengeResolved {
        challenger: PlayerId,
        offender: PlayerId,
        result: UnoChallengeResult,
        penalized: PlayerId,
        cards: Vec<UnoCard>,
        next_player: PlayerId,
    },
    UnoCalled {
        player: PlayerId,
    },
    UnoReported {
        reporter: PlayerId,
        target: PlayerId,
        cards: Vec<UnoCard>,
    },
    SkipResolved {
        player: PlayerId,
        cards: Vec<UnoCard>,
        remaining: u16,
        next_player: PlayerId,
    },
    ColorRouletteResolved {
        player: PlayerId,
        color: UnoColor,
        cards: Vec<UnoCard>,
        next_player: PlayerId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnoGameError {
    InvalidRules(UnoRuleError),
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
    CardNotInHand(UnoCard),
    CardDoesNotMatch,
    ColorRequired,
    UnexpectedColor,
    MustPlayDrawnCard(UnoCard),
    MustResolveDrawPenalty,
    NoDrawPenalty,
    CannotStack(UnoCard),
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

impl fmt::Display for UnoGameError {
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

impl std::error::Error for UnoGameError {}

impl From<DeckError> for UnoGameError {
    fn from(value: DeckError) -> Self {
        match value {
            DeckError::InvalidSize { expected, actual } => {
                Self::InvalidDeckSize { expected, actual }
            }
            DeckError::InvalidContents => Self::InvalidDeckContents,
        }
    }
}

impl From<UnoRuleError> for UnoGameError {
    fn from(value: UnoRuleError) -> Self {
        Self::InvalidRules(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ChallengeState {
    pub(super) offender: PlayerId,
    pub(super) was_legal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnoGameState {
    pub(super) rules: UnoRuleSet,
    pub(super) players: Vec<UnoPlayerState>,
    pub(super) draw_pile: VecDeque<UnoCard>,
    pub(super) discard_pile: Vec<UnoCard>,
    pub(super) current_player: PlayerId,
    pub(super) direction: UnoDirection,
    pub(super) current_color: Option<UnoColor>,
    pub(super) pending_draw: u16,
    pub(super) pending_kind: Option<UnoPendingDrawKind>,
    pub(super) pending_draw_source: Option<PlayerId>,
    pub(super) pending_draw_colors: Vec<UnoColor>,
    pub(super) challenge: Option<ChallengeState>,
    pub(super) drawn_card: Option<UnoCard>,
    pub(super) pending_skip: u16,
    pub(super) pending_skip_everyone: bool,
    pub(super) pending_skip_source: Option<PlayerId>,
    /// Recipients that must still accept the current skip effect, before a pending win.
    pub(super) skip_effect_targets: Vec<PlayerId>,
    pub(super) skip_turns: Vec<u16>,
    pub(super) uno_exposed: Vec<bool>,
    pub(super) uno_declared: Vec<bool>,
    pub(super) jump_in_open: bool,
    pub(super) pending_swap: Option<PendingSwapState>,
    pub(super) pending_finisher: Option<PlayerId>,
    pub(super) set_aside_cards: Vec<UnoCard>,
    pub(super) elimination_order: Vec<PlayerId>,
    pub(super) sudden_death_misses: u8,
    pub(super) flip_side: Option<UnoFlipSide>,
    pub(super) phase: UnoPhase,
}

pub(super) fn remove_card(hand: &mut Vec<UnoCard>, card: UnoCard) {
    let index = hand
        .iter()
        .position(|candidate| *candidate == card)
        .expect("caller checked that the card is in hand");
    hand.remove(index);
}

pub(super) fn faces_match(left: UnoFace, right: UnoFace) -> bool {
    if left == right {
        return !matches!(left, UnoFace::StackOne | UnoFace::StackTwo);
    }
    matches!(
        (left, right),
        (UnoFace::ReverseDrawTwo, UnoFace::Reverse | UnoFace::DrawTwo)
            | (UnoFace::Reverse | UnoFace::DrawTwo, UnoFace::ReverseDrawTwo)
            | (UnoFace::ReverseSkip, UnoFace::Reverse | UnoFace::Skip)
            | (UnoFace::Reverse | UnoFace::Skip, UnoFace::ReverseSkip)
    )
}

pub(super) fn validate_deck(deck: &[UnoCard], rules: UnoRuleSet) -> Result<(), UnoGameError> {
    let expected = build_deck_for_rules(rules);
    if !rules.is_flip() {
        return validate_physical_deck(deck, &expected).map_err(UnoGameError::from);
    }
    if deck.len() != expected.len() {
        return Err(UnoGameError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
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
    if actual_light != expected_light || actual_dark != expected_dark || !has_unique_cards(deck) {
        return Err(UnoGameError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn swap_player_hands(players: &mut [UnoPlayerState], first: PlayerId, second: PlayerId) {
    let (low, high) = if first.index() < second.index() {
        (first.index(), second.index())
    } else {
        (second.index(), first.index())
    };
    let (left, right) = players.split_at_mut(high);
    std::mem::swap(&mut left[low].hand, &mut right[0].hand);
}
