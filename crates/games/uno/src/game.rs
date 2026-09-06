use std::collections::{HashSet, VecDeque};
use std::fmt;

use crate::rating::{placements_with_eliminations, reference_point_deltas_for_placements};
use crate::{RuleError, UnoCard, UnoColor, UnoFace, UnoFlipSide, UnoRuleSet, build_deck_for_rules};

mod mechanics;
mod play;
mod setup;
mod swap;
mod turn;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoPlayerId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    id: UnoPlayerId,
    hand: Vec<UnoCard>,
    eliminated: bool,
}

impl PlayerState {
    pub const fn id(&self) -> UnoPlayerId {
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
    const fn reversed(self) -> Self {
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
pub struct TurnState {
    pub current_player: UnoPlayerId,
    pub direction: UnoDirection,
    pub current_color: Option<UnoColor>,
    pub top_card: UnoCard,
    pub pending_draw: u16,
    pub pending_kind: Option<UnoPendingDrawKind>,
    pub pending_draw_source: Option<UnoPlayerId>,
    pub challenge_offender: Option<UnoPlayerId>,
    pub drawn_card: Option<UnoCard>,
    pub pending_skip: u16,
    pub skipped_turns_remaining: u16,
    pub pending_swap: Option<PendingSwap>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingSwap {
    SwapOneTarget {
        player: UnoPlayerId,
    },
    SwapOneGive {
        player: UnoPlayerId,
        target: UnoPlayerId,
    },
    ForceTrade {
        player: UnoPlayerId,
    },
    ChooseColor {
        player: UnoPlayerId,
    },
    SevenSwap {
        player: UnoPlayerId,
    },
    ColorRoulette {
        player: UnoPlayerId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingSwapState {
    SwapOneTarget {
        player: UnoPlayerId,
        declared_uno: bool,
    },
    SwapOneGive {
        player: UnoPlayerId,
        target: UnoPlayerId,
        declared_uno: bool,
    },
    ForceTrade {
        player: UnoPlayerId,
    },
    ChooseColor {
        player: UnoPlayerId,
    },
    SevenSwap {
        player: UnoPlayerId,
    },
    ColorRoulette {
        player: UnoPlayerId,
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
    HandRefreshed {
        count: u16,
    },
    HandsPassed {
        direction: UnoDirection,
    },
    DrawReflected {
        player: UnoPlayerId,
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
pub struct GameResult {
    pub winner: UnoPlayerId,
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
pub enum UnoChallengeResult {
    Successful,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    ColorChosen {
        player: UnoPlayerId,
        color: UnoColor,
    },
    Played {
        player: UnoPlayerId,
        card: UnoCard,
        next_player: UnoPlayerId,
        effect: Option<PlayedEffect>,
    },
    SwapOneCardTaken {
        player: UnoPlayerId,
        target: UnoPlayerId,
    },
    SwapOneCompleted {
        player: UnoPlayerId,
        target: UnoPlayerId,
        next_player: UnoPlayerId,
    },
    HandsTraded {
        player: UnoPlayerId,
        first: UnoPlayerId,
        second: UnoPlayerId,
    },
    DrewCards {
        player: UnoPlayerId,
        cards: Vec<UnoCard>,
        playable: Option<UnoCard>,
        next_player: UnoPlayerId,
    },
    PassedAfterDraw {
        player: UnoPlayerId,
        next_player: UnoPlayerId,
    },
    PenaltyDrawn {
        player: UnoPlayerId,
        cards: Vec<UnoCard>,
        next_player: UnoPlayerId,
    },
    ChallengeResolved {
        challenger: UnoPlayerId,
        offender: UnoPlayerId,
        result: UnoChallengeResult,
        penalized: UnoPlayerId,
        cards: Vec<UnoCard>,
        next_player: UnoPlayerId,
    },
    UnoCalled {
        player: UnoPlayerId,
    },
    UnoReported {
        reporter: UnoPlayerId,
        target: UnoPlayerId,
        cards: Vec<UnoCard>,
    },
    SkipResolved {
        player: UnoPlayerId,
        cards: Vec<UnoCard>,
        remaining: u16,
        next_player: UnoPlayerId,
    },
    ColorRouletteResolved {
        player: UnoPlayerId,
        color: UnoColor,
        cards: Vec<UnoCard>,
        next_player: UnoPlayerId,
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
    InvalidPlayer(UnoPlayerId),
    PlayerEliminated(UnoPlayerId),
    NotPlayersTurn {
        expected: UnoPlayerId,
        actual: UnoPlayerId,
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
    CannotCallUno(UnoPlayerId),
    MustPlayAfterUno,
    CannotReportSelf,
    PlayerNotReportable(UnoPlayerId),
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
    offender: UnoPlayerId,
    was_legal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: UnoRuleSet,
    players: Vec<PlayerState>,
    draw_pile: VecDeque<UnoCard>,
    discard_pile: Vec<UnoCard>,
    current_player: UnoPlayerId,
    direction: UnoDirection,
    current_color: Option<UnoColor>,
    pending_draw: u16,
    pending_kind: Option<UnoPendingDrawKind>,
    pending_draw_source: Option<UnoPlayerId>,
    pending_draw_colors: Vec<UnoColor>,
    challenge: Option<ChallengeState>,
    drawn_card: Option<UnoCard>,
    pending_skip: u16,
    pending_skip_everyone: bool,
    pending_skip_source: Option<UnoPlayerId>,
    skip_turns: Vec<u16>,
    uno_exposed: Vec<bool>,
    uno_declared: Vec<bool>,
    jump_in_open: bool,
    pending_swap: Option<PendingSwapState>,
    pending_finisher: Option<UnoPlayerId>,
    set_aside_cards: Vec<UnoCard>,
    elimination_order: Vec<UnoPlayerId>,
    flip_side: Option<UnoFlipSide>,
    phase: Phase,
}

fn remove_card(hand: &mut Vec<UnoCard>, card: UnoCard) {
    let index = hand
        .iter()
        .position(|candidate| *candidate == card)
        .expect("caller checked that the card is in hand");
    hand.remove(index);
}

fn faces_match(left: UnoFace, right: UnoFace) -> bool {
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

fn validate_deck(deck: &[UnoCard], rules: UnoRuleSet) -> Result<(), GameError> {
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

fn swap_player_hands(players: &mut [PlayerState], first: UnoPlayerId, second: UnoPlayerId) {
    let (low, high) = if first.0 < second.0 {
        (first.0, second.0)
    } else {
        (second.0, first.0)
    };
    let (left, right) = players.split_at_mut(high);
    std::mem::swap(&mut left[low].hand, &mut right[0].hand);
}
