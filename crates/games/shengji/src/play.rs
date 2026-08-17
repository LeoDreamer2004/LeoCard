use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::{Card, Rank, RuleSet, Suit, Trump};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Category {
    Trump,
    Suit(Suit),
    /// 仅用于缺门后的混合垫牌；首家不能领出混合门类，且这种牌永远不能赢墩。
    Mixed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Component {
    Single {
        card: Card,
        strength: u8,
    },
    Pair {
        cards: [Card; 2],
        strength: u8,
    },
    Triple {
        cards: [Card; 3],
        strength: u8,
    },
    /// 四副牌中四张完全相同牌面的“炸弹”。相同牌力但不同花色不能合并。
    Quad {
        cards: [Card; 4],
        strength: u8,
    },
    Tractor {
        cards: Vec<Card>,
        pair_count: u8,
        top_strength: u8,
    },
    /// 三副牌中由相邻三同张组成的“泰坦尼克”。
    Titanic {
        cards: Vec<Card>,
        triple_count: u8,
        top_strength: u8,
    },
    /// 四副牌中由相邻四同张组成的“宇宙飞船”。
    Spaceship {
        cards: Vec<Card>,
        quad_count: u8,
        top_strength: u8,
    },
}

impl Component {
    pub fn card_count(&self) -> usize {
        match self {
            Self::Single { .. } => 1,
            Self::Pair { .. } => 2,
            Self::Triple { .. } => 3,
            Self::Quad { .. } => 4,
            Self::Tractor { cards, .. }
            | Self::Titanic { cards, .. }
            | Self::Spaceship { cards, .. } => cards.len(),
        }
    }

    pub const fn comparison_strength(&self) -> u8 {
        match self {
            Self::Single { strength, .. }
            | Self::Pair { strength, .. }
            | Self::Triple { strength, .. }
            | Self::Quad { strength, .. } => *strength,
            Self::Tractor { top_strength, .. }
            | Self::Titanic { top_strength, .. }
            | Self::Spaceship { top_strength, .. } => *top_strength,
        }
    }

    pub fn cards(&self) -> Vec<Card> {
        match self {
            Self::Single { card, .. } => vec![*card],
            Self::Pair { cards, .. } => cards.to_vec(),
            Self::Triple { cards, .. } => cards.to_vec(),
            Self::Quad { cards, .. } => cards.to_vec(),
            Self::Tractor { cards, .. }
            | Self::Titanic { cards, .. }
            | Self::Spaceship { cards, .. } => cards.clone(),
        }
    }

    pub fn kitty_multiplier(&self) -> u32 {
        match self {
            Self::Single { .. } => 2,
            Self::Pair { .. } => 4,
            Self::Triple { .. } => 8,
            Self::Quad { .. } => 16,
            Self::Tractor { pair_count, .. } => 1_u32 << (u32::from(*pair_count) + 1),
            Self::Titanic { triple_count, .. } => 4_u32.saturating_pow(u32::from(*triple_count)),
            Self::Spaceship { quad_count, .. } => 8_u32.saturating_pow(u32::from(*quad_count)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassifiedPlay {
    pub cards: Vec<Card>,
    pub category: Category,
    pub components: Vec<Component>,
}

impl ClassifiedPlay {
    pub fn is_throw(&self) -> bool {
        self.components.len() > 1
            && !matches!(
                self.components.as_slice(),
                [Component::Quad { .. }, Component::Quad { .. }]
            )
    }

    pub fn strongest_component(&self) -> &Component {
        self.components
            .iter()
            .max_by_key(|component| component_priority(component))
            .expect("a play has at least one component")
    }

    pub fn kitty_multiplier(&self) -> u32 {
        self.components
            .iter()
            .map(Component::kitty_multiplier)
            .max()
            .unwrap_or(2)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThrowFailure {
    pub attempted: ClassifiedPlay,
    pub forced: ClassifiedPlay,
    pub penalty_points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrickPlay {
    Accepted(ClassifiedPlay),
    ThrowFailed(ThrowFailure),
}

impl TrickPlay {
    pub fn effective(&self) -> &ClassifiedPlay {
        match self {
            Self::Accepted(play) => play,
            Self::ThrowFailed(failure) => &failure.forced,
        }
    }
}

#[derive(Clone, Debug)]
struct PairUnit {
    cards: [Card; 2],
    strength: u8,
}

#[derive(Clone, Debug)]
struct TripleUnit {
    cards: [Card; 3],
    strength: u8,
}

#[derive(Clone, Debug)]
struct QuadUnit {
    cards: [Card; 4],
    strength: u8,
}

pub fn classify_lead(
    cards: &[Card],
    trump: Trump,
    rules: &RuleSet,
    opponent_hands: &[&[Card]],
) -> Result<TrickPlay, PlayError> {
    let attempted = classify_cards(cards, trump)?;
    if !attempted.is_throw() {
        return Ok(TrickPlay::Accepted(attempted));
    }
    if !rules.allow_throw {
        return Err(PlayError::ThrowDisabled);
    }

    let failed = attempted
        .components
        .iter()
        .filter(|component| {
            opponent_hands
                .iter()
                .any(|hand| component_can_be_beaten(component, attempted.category, hand, trump))
        })
        .min_by_key(|component| {
            (
                component.card_count(),
                component.comparison_strength(),
                component_kind_order(component),
            )
        });
    let Some(failed) = failed else {
        return Ok(TrickPlay::Accepted(attempted));
    };
    let forced = classify_cards(&failed.cards(), trump)?;
    Ok(TrickPlay::ThrowFailed(ThrowFailure {
        penalty_points: (cards.len() as u16).saturating_mul(rules.throw_penalty.points_per_card()),
        attempted,
        forced,
    }))
}

pub fn validate_follow(
    hand: &[Card],
    cards: &[Card],
    lead: &ClassifiedPlay,
    trump: Trump,
) -> Result<ClassifiedPlay, FollowError> {
    if cards.len() != lead.cards.len() {
        return Err(FollowError::WrongCardCount {
            expected: lead.cards.len(),
            actual: cards.len(),
        });
    }
    validate_owned(cards, hand).map_err(FollowError::Play)?;
    let required_in_hand = hand
        .iter()
        .filter(|card| category(**card, trump) == lead.category)
        .count();
    let required = required_in_hand.min(cards.len());
    let actual = cards
        .iter()
        .filter(|card| category(**card, trump) == lead.category)
        .count();
    if actual != required {
        return Err(FollowError::MustFollowCategory { required, actual });
    }
    if required_in_hand >= cards.len() {
        let hand_cards = hand
            .iter()
            .copied()
            .filter(|card| category(*card, trump) == lead.category)
            .collect::<Vec<_>>();
        let proposed = cards.to_vec();
        let follows_structure = if let Some(hierarchy) = special_follow_hierarchy(lead) {
            best_follow_tier(&proposed, trump, &hierarchy, true)
                == best_follow_tier(&hand_cards, trump, &hierarchy, false)
        } else {
            match lead.components.as_slice() {
                [Component::Triple { .. }] => {
                    triple_follow_rank(&proposed) == triple_follow_rank(&hand_cards)
                }
                [Component::Titanic { triple_count, .. }] => {
                    titanic_follow_rank(&proposed, usize::from(*triple_count), trump)
                        == titanic_follow_rank(&hand_cards, usize::from(*triple_count), trump)
                }
                _ => {
                    let demands = StructureDemands::from_play(lead);
                    satisfies_follow_structure(
                        &hand_cards,
                        &proposed,
                        &demands,
                        trump,
                        lead.category,
                    )
                }
            }
        };
        if !follows_structure {
            return Err(FollowError::MustFollowStructure);
        }
    }
    if cards
        .iter()
        .all(|card| category(*card, trump) == category(cards[0], trump))
    {
        classify_cards(cards, trump).map_err(FollowError::Play)
    } else {
        Ok(classify_mixed_discard(cards, trump))
    }
}

fn triple_follow_rank(cards: &[Card]) -> u8 {
    let counts = face_counts(cards);
    if counts.values().any(|count| *count >= 3) {
        2
    } else if counts.values().any(|count| *count >= 2) {
        1
    } else {
        0
    }
}

/// 泰坦尼克跟牌顺序：泰坦尼克 > 拖拉机 > 尽可能多三同张 > 尽可能多对子。
/// 对六张牌展开后正好对应规则列出的八个层级。
fn titanic_follow_rank(
    cards: &[Card],
    triple_count: usize,
    trump: Trump,
) -> (bool, bool, usize, usize) {
    let triples = triple_units(cards, trump);
    let pairs = pair_units(cards, trump);
    let full_titanic = !titanic_tops(&triples, triple_count).is_empty();
    let full_tractor = !tractor_tops(&pairs, triple_count).is_empty();
    if full_titanic {
        return (true, false, 0, 0);
    }
    if full_tractor {
        return (false, true, 0, 0);
    }
    let required_cards = triple_count * 3;
    let selected_triples = triples.len().min(triple_count);
    let remaining_slots = required_cards.saturating_sub(selected_triples * 3);
    let remaining_pair_faces = pairs.len().saturating_sub(selected_triples);
    let selected_pairs = remaining_pair_faces.min(remaining_slots / 2);
    (full_titanic, full_tractor, selected_triples, selected_pairs)
}

fn face_counts(cards: &[Card]) -> BTreeMap<(Option<Suit>, Rank), usize> {
    let mut counts = BTreeMap::new();
    for card in cards {
        *counts.entry((card.suit(), card.rank())).or_default() += 1;
    }
    counts
}

#[derive(Clone, Copy, Debug)]
struct FollowPattern {
    run_width: u8,
    run_len: u8,
    quads: u8,
    triples: u8,
    pairs: u8,
    singles: u8,
    wildcards: u8,
}

const fn pattern(
    run_width: u8,
    run_len: u8,
    quads: u8,
    triples: u8,
    pairs: u8,
    singles: u8,
) -> FollowPattern {
    FollowPattern {
        run_width,
        run_len,
        quads,
        triples,
        pairs,
        singles,
        wildcards: 0,
    }
}

const fn pattern_with_wildcards(run_width: u8, run_len: u8, wildcards: u8) -> FollowPattern {
    FollowPattern {
        run_width,
        run_len,
        quads: 0,
        triples: 0,
        pairs: 0,
        singles: 0,
        wildcards,
    }
}

fn special_follow_hierarchy(lead: &ClassifiedPlay) -> Option<Vec<Vec<FollowPattern>>> {
    let p = pattern;
    match lead.components.as_slice() {
        [Component::Quad { .. }] => Some(vec![
            vec![p(0, 0, 1, 0, 0, 0)],
            vec![p(2, 2, 0, 0, 0, 0)],
            vec![p(0, 0, 0, 0, 2, 0)],
            vec![p(0, 0, 0, 1, 0, 1)],
            vec![p(0, 0, 0, 0, 1, 2)],
            vec![p(0, 0, 0, 0, 0, 4)],
        ]),
        [Component::Tractor { pair_count: 2, .. }] => Some(vec![
            // 两连对与炸弹同属最高跟牌义务；实际比较时炸弹压过任意两连对。
            vec![p(2, 2, 0, 0, 0, 0), p(0, 0, 1, 0, 0, 0)],
            vec![p(0, 0, 0, 0, 2, 0)],
            vec![p(0, 0, 0, 0, 1, 2)],
            vec![p(0, 0, 0, 0, 0, 4)],
        ]),
        [Component::Tractor { pair_count: 3, .. }] => Some(vec![
            // 三连对与两连泰坦尼克均可跟；泰坦尼克只在牌力比较时更大。
            vec![p(2, 3, 0, 0, 0, 0), p(3, 2, 0, 0, 0, 0)],
            // 四副牌新增；三副牌中手里不可能出现这一层。
            vec![p(0, 0, 1, 0, 1, 0)],
            vec![p(2, 2, 0, 0, 1, 0)],
            vec![p(0, 0, 0, 0, 3, 0)],
            vec![p(0, 0, 0, 0, 2, 2)],
            vec![p(0, 0, 0, 0, 1, 4)],
            vec![p(0, 0, 0, 0, 0, 6)],
        ]),
        [
            Component::Titanic {
                triple_count: 2, ..
            },
        ] => Some(vec![
            vec![p(3, 2, 0, 0, 0, 0)],
            vec![pattern_with_wildcards(2, 2, 2)],
            vec![p(0, 0, 0, 2, 0, 0)],
            vec![p(0, 0, 0, 1, 1, 1)],
            vec![p(0, 0, 0, 1, 0, 3)],
            vec![p(0, 0, 0, 0, 2, 2)],
            vec![p(0, 0, 0, 0, 1, 4)],
            vec![p(0, 0, 0, 0, 0, 6)],
        ]),
        [Component::Tractor { pair_count: 4, .. }] => Some(eight_card_tractor_hierarchy()),
        [Component::Spaceship { quad_count: 2, .. }]
        | [Component::Quad { .. }, Component::Quad { .. }] => Some(eight_card_quad_hierarchy()),
        _ => None,
    }
}

fn eight_card_tractor_hierarchy() -> Vec<Vec<FollowPattern>> {
    let p = pattern;
    vec![
        vec![p(4, 2, 0, 0, 0, 0)],
        vec![p(3, 2, 0, 0, 1, 0)],
        vec![p(3, 2, 0, 0, 0, 2)],
        vec![p(2, 2, 0, 0, 2, 0)],
        vec![p(2, 2, 0, 0, 1, 2)],
        vec![p(2, 2, 0, 0, 0, 4)],
        vec![p(0, 0, 1, 0, 2, 0)],
        vec![p(0, 0, 1, 0, 1, 2)],
        vec![p(0, 0, 1, 0, 0, 4)],
        vec![p(0, 0, 0, 2, 0, 2)],
        vec![p(0, 0, 0, 1, 2, 1)],
        vec![p(0, 0, 0, 1, 1, 3)],
        vec![p(0, 0, 0, 1, 0, 5)],
        vec![p(0, 0, 0, 0, 4, 0)],
        vec![p(0, 0, 0, 0, 3, 2)],
        vec![p(0, 0, 0, 0, 2, 4)],
        vec![p(0, 0, 0, 0, 1, 6)],
        vec![p(0, 0, 0, 0, 0, 8)],
    ]
}

fn eight_card_quad_hierarchy() -> Vec<Vec<FollowPattern>> {
    let p = pattern;
    vec![
        vec![p(4, 2, 0, 0, 0, 0)],
        vec![p(0, 0, 2, 0, 0, 0)],
        vec![p(3, 2, 0, 0, 1, 0)],
        vec![p(3, 2, 0, 0, 0, 2)],
        vec![p(2, 3, 0, 0, 1, 0)],
        vec![p(2, 3, 0, 0, 0, 2)],
        vec![p(2, 2, 0, 0, 2, 0)],
        vec![p(2, 2, 0, 0, 1, 2)],
        vec![p(2, 2, 0, 0, 0, 4)],
        vec![p(0, 0, 1, 0, 2, 0)],
        vec![p(0, 0, 1, 0, 1, 2)],
        vec![p(0, 0, 1, 0, 0, 4)],
        vec![p(0, 0, 0, 2, 0, 2)],
        vec![p(0, 0, 0, 1, 2, 1)],
        vec![p(0, 0, 0, 1, 1, 3)],
        vec![p(0, 0, 0, 1, 0, 5)],
        vec![p(0, 0, 0, 0, 4, 0)],
        vec![p(0, 0, 0, 0, 3, 2)],
        vec![p(0, 0, 0, 0, 2, 4)],
        vec![p(0, 0, 0, 0, 1, 6)],
        vec![p(0, 0, 0, 0, 0, 8)],
    ]
}

#[derive(Clone)]
struct FaceStock {
    cards: Vec<Card>,
    strength: u8,
}

fn face_stocks(cards: &[Card], trump: Trump) -> Vec<FaceStock> {
    let mut faces: BTreeMap<(Option<Suit>, Rank), Vec<Card>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    let mut stocks = faces
        .into_values()
        .map(|mut cards| {
            cards.sort_by_key(|card| card.deck());
            FaceStock {
                strength: strength(cards[0], trump),
                cards,
            }
        })
        .collect::<Vec<_>>();
    stocks.sort_by_key(|stock| {
        (
            stock.strength,
            stock.cards[0].suit().map_or(4, Suit::bid_strength),
            stock.cards[0].rank(),
        )
    });
    stocks
}

fn pattern_assignments(
    cards: &[Card],
    trump: Trump,
    pattern: FollowPattern,
    exact: bool,
    limit: usize,
) -> (Vec<Vec<usize>>, bool) {
    let stocks = face_stocks(cards, trump);
    let mut group_widths = Vec::new();
    group_widths.extend(std::iter::repeat_n(4, usize::from(pattern.quads)));
    group_widths.extend(std::iter::repeat_n(3, usize::from(pattern.triples)));
    group_widths.extend(std::iter::repeat_n(2, usize::from(pattern.pairs)));
    group_widths.extend(std::iter::repeat_n(1, usize::from(pattern.singles)));
    let starts = if pattern.run_len == 0 {
        vec![None]
    } else {
        let mut starts = stocks
            .iter()
            .map(|stock| stock.strength)
            .collect::<HashSet<_>>()
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        starts.sort_unstable();
        starts
    };
    let mut output = Vec::new();
    let mut complete = true;
    for start in starts {
        let mut selected = vec![0; stocks.len()];
        collect_run_assignments(
            &stocks,
            pattern,
            start,
            0,
            &mut selected,
            &group_widths,
            0,
            exact,
            limit,
            &mut output,
            &mut complete,
        );
        if !complete {
            break;
        }
    }
    output.sort();
    output.dedup();
    (output, complete)
}

#[allow(clippy::too_many_arguments)]
fn collect_run_assignments(
    stocks: &[FaceStock],
    pattern: FollowPattern,
    start: Option<u8>,
    offset: usize,
    selected: &mut [usize],
    group_widths: &[usize],
    group_index: usize,
    exact: bool,
    limit: usize,
    output: &mut Vec<Vec<usize>>,
    complete: &mut bool,
) {
    if output.len() >= limit {
        *complete = false;
        return;
    }
    if offset < usize::from(pattern.run_len) {
        let wanted = start.unwrap() + offset as u8;
        for index in 0..stocks.len() {
            if selected[index] == 0
                && stocks[index].strength == wanted
                && stocks[index].cards.len() >= usize::from(pattern.run_width)
            {
                selected[index] = usize::from(pattern.run_width);
                collect_run_assignments(
                    stocks,
                    pattern,
                    start,
                    offset + 1,
                    selected,
                    group_widths,
                    group_index,
                    exact,
                    limit,
                    output,
                    complete,
                );
                selected[index] = 0;
            }
        }
        return;
    }
    if group_index < group_widths.len() {
        let width = group_widths[group_index];
        for index in 0..stocks.len() {
            if selected[index] == 0 && stocks[index].cards.len() >= width {
                selected[index] = width;
                collect_run_assignments(
                    stocks,
                    pattern,
                    start,
                    offset,
                    selected,
                    group_widths,
                    group_index + 1,
                    exact,
                    limit,
                    output,
                    complete,
                );
                selected[index] = 0;
            }
        }
        return;
    }

    let used = selected.iter().sum::<usize>();
    let remaining = stocks
        .iter()
        .zip(selected.iter())
        .map(|(stock, used)| stock.cards.len() - *used)
        .sum::<usize>();
    let wildcards = usize::from(pattern.wildcards);
    if remaining < wildcards || (exact && remaining != wildcards) {
        return;
    }
    let mut completed = selected.to_vec();
    let mut needed = wildcards;
    for (index, stock) in stocks.iter().enumerate() {
        let available = stock.cards.len() - completed[index];
        let take = available.min(needed);
        completed[index] += take;
        needed -= take;
    }
    debug_assert_eq!(needed, 0);
    debug_assert_eq!(completed.iter().sum::<usize>(), used + wildcards);
    output.push(completed);
}

fn best_follow_tier(
    cards: &[Card],
    trump: Trump,
    hierarchy: &[Vec<FollowPattern>],
    exact: bool,
) -> Option<usize> {
    hierarchy.iter().position(|tier| {
        tier.iter().any(|pattern| {
            let (assignments, _) = pattern_assignments(cards, trump, *pattern, exact, 1);
            !assignments.is_empty()
        })
    })
}

fn special_follow_card_sets(
    cards: &[Card],
    trump: Trump,
    hierarchy: &[Vec<FollowPattern>],
    limit: usize,
) -> Vec<Vec<Card>> {
    let Some(tier) = best_follow_tier(cards, trump, hierarchy, false) else {
        return Vec::new();
    };
    let stocks = face_stocks(cards, trump);
    let mut output = Vec::new();
    // 先从同一义务层级的每一种牌型各取一个，确保“提示”可轮换展示不同牌型。
    for pattern in &hierarchy[tier] {
        let (assignments, _) = pattern_assignments(cards, trump, *pattern, false, 1);
        for assignment in assignments {
            output.push(cards_for_assignment(&stocks, &assignment, trump));
        }
        if output.len() >= limit {
            break;
        }
    }
    if output.len() < limit {
        for pattern in &hierarchy[tier] {
            let (assignments, _) =
                pattern_assignments(cards, trump, *pattern, false, limit - output.len() + 1);
            for assignment in assignments {
                output.push(cards_for_assignment(&stocks, &assignment, trump));
                if output.len() >= limit {
                    break;
                }
            }
            if output.len() >= limit {
                break;
            }
        }
    }
    let mut seen = HashSet::new();
    output.retain(|candidate| seen.insert(candidate.clone()));
    output.truncate(limit);
    output
}

fn cards_for_assignment(stocks: &[FaceStock], assignment: &[usize], trump: Trump) -> Vec<Card> {
    let mut selected = Vec::new();
    for (stock, count) in stocks.iter().zip(assignment) {
        selected.extend(stock.cards.iter().take(*count).copied());
    }
    selected.sort_by_key(|card| (strength(*card, trump), card.deck()));
    selected
}

fn forced_cards_for_hierarchy(
    hand: &[Card],
    cards: &[Card],
    trump: Trump,
    hierarchy: &[Vec<FollowPattern>],
) -> Vec<Card> {
    const ASSIGNMENT_LIMIT: usize = 4096;
    let Some(tier) = best_follow_tier(cards, trump, hierarchy, false) else {
        return Vec::new();
    };
    // “任意牌”存在大量等价选择；不抢替玩家作主观选择。
    if hierarchy[tier].iter().any(|pattern| pattern.wildcards > 0) {
        return Vec::new();
    }
    let stocks = face_stocks(cards, trump);
    let mut assignments = Vec::new();
    for pattern in &hierarchy[tier] {
        let remaining = ASSIGNMENT_LIMIT.saturating_sub(assignments.len());
        if remaining == 0 {
            return Vec::new();
        }
        let (mut found, complete) = pattern_assignments(cards, trump, *pattern, false, remaining);
        if !complete {
            return Vec::new();
        }
        assignments.append(&mut found);
    }
    let Some(first) = assignments.first() else {
        return Vec::new();
    };
    let mut minimum = first.clone();
    for assignment in &assignments[1..] {
        for (minimum, selected) in minimum.iter_mut().zip(assignment) {
            *minimum = (*minimum).min(*selected);
        }
    }
    let forced = stocks
        .iter()
        .zip(minimum)
        .flat_map(|(stock, minimum)| {
            // 同牌面有未被选中的实体副本时，不应擅自指定其中某一张。
            (minimum == stock.cards.len())
                .then_some(stock.cards.as_slice())
                .unwrap_or(&[])
                .iter()
                .copied()
        })
        .collect::<HashSet<_>>();
    hand.iter()
        .copied()
        .filter(|card| forced.contains(card))
        .collect()
}

/// 返回当前跟牌中无论如何选择都必须包含的实体牌。
///
/// 这和 [`validate_follow`] 使用完全相同的跟门、拖拉机及对子义务：同门牌不够时
/// 所有同门牌都是必选；同门牌足够时，只返回所有合法结构选择的交集。客户端可
/// 直接高亮这些牌，但仍由服务端在实际出牌时作最终校验。
pub fn forced_follow_cards(hand: &[Card], lead: &ClassifiedPlay, trump: Trump) -> Vec<Card> {
    let required = lead.cards.len();
    if required == 0 || hand.is_empty() {
        return Vec::new();
    }
    if hand.len() <= required {
        return hand.to_vec();
    }

    let same_category = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) == lead.category)
        .collect::<Vec<_>>();
    if same_category.len() <= required {
        return same_category;
    }
    if let Some(hierarchy) = special_follow_hierarchy(lead) {
        return forced_cards_for_hierarchy(hand, &same_category, trump, &hierarchy);
    }

    if matches!(lead.components.as_slice(), [Component::Triple { .. }]) {
        match triple_follow_rank(&same_category) {
            2 => {
                let triples = triple_units(&same_category, trump);
                if triples.len() == 1 {
                    return triples[0].cards.to_vec();
                }
            }
            1 => {
                let pairs = pair_units(&same_category, trump);
                if pairs.len() == 1 {
                    return pairs[0].cards.to_vec();
                }
            }
            _ => {}
        }
    }
    if let [Component::Titanic { triple_count, .. }] = lead.components.as_slice() {
        let titanics = titanic_card_sets(&same_category, trump, usize::from(*triple_count));
        if let Some(first) = titanics.first() {
            let mut intersection = first.iter().copied().collect::<HashSet<_>>();
            for candidate in &titanics[1..] {
                let candidate = candidate.iter().copied().collect::<HashSet<_>>();
                intersection.retain(|card| candidate.contains(card));
            }
            return hand
                .iter()
                .copied()
                .filter(|card| intersection.contains(card))
                .collect();
        }
    }

    let demands = StructureDemands::from_play(lead);
    let all_pairs = pair_units(&same_category, trump);
    let mut obligation_pairs = all_pairs.clone();
    let mut required_runs = Vec::new();
    for demand in &demands.tractors {
        let best = best_run_at_most(&obligation_pairs, *demand);
        if best.len() >= 2 {
            required_runs.push(best.len());
            remove_pair_indices(&mut obligation_pairs, &best);
        }
    }
    let required_pair_count = demands.pair_slots().min(all_pairs.len());
    if required_pair_count == 0 {
        return Vec::new();
    }

    let mut forced = None::<HashSet<Card>>;
    for mask in 0_u32..(1_u32 << all_pairs.len()) {
        if mask.count_ones() as usize != required_pair_count {
            continue;
        }
        let mut selected_pairs = all_pairs
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .map(|(_, pair)| pair.clone())
            .collect::<Vec<_>>();
        if !pair_selection_satisfies_runs(&mut selected_pairs, &required_runs) {
            continue;
        }
        let selected = all_pairs
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .flat_map(|(_, pair)| pair.cards)
            .collect::<HashSet<_>>();
        match &mut forced {
            Some(forced) => forced.retain(|card| selected.contains(card)),
            None => forced = Some(selected),
        }
    }

    let forced = forced.unwrap_or_default();
    hand.iter()
        .copied()
        .filter(|card| forced.contains(card))
        .collect()
}

/// 按从小到大的确定性顺序枚举若干组合法跟牌，用于客户端连续提示。
///
/// 实体牌组合可能非常多，因此调用方提供候选上限；搜索本身也有固定预算，避免
/// 极端大甩牌令界面卡顿。服务端仍会通过 [`validate_follow`] 校验最终选择。
pub fn follow_suggestions(
    hand: &[Card],
    lead: &ClassifiedPlay,
    trump: Trump,
    limit: usize,
) -> Vec<ClassifiedPlay> {
    const SEARCH_BUDGET: usize = 200_000;

    let required = lead.cards.len();
    if limit == 0 || required == 0 || hand.len() < required {
        return Vec::new();
    }
    let mut same_category = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) == lead.category)
        .collect::<Vec<_>>();
    let mut other = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) != lead.category)
        .collect::<Vec<_>>();
    let card_key = |card: &Card| {
        (
            trump.is_trump(*card),
            strength(*card, trump),
            card.suit().map_or(4, Suit::bid_strength),
            card.deck(),
        )
    };
    same_category.sort_by_key(card_key);
    other.sort_by_key(card_key);

    if same_category.len() >= required
        && let Some(hierarchy) = special_follow_hierarchy(lead)
    {
        let candidates = special_follow_card_sets(&same_category, trump, &hierarchy, limit);
        let suggestions = candidates
            .into_iter()
            .filter_map(|cards| validate_follow(hand, &cards, lead, trump).ok())
            .take(limit)
            .collect::<Vec<_>>();
        if !suggestions.is_empty() {
            return suggestions;
        }
    }

    let (fixed, choices, choose_count) = if same_category.len() >= required {
        (Vec::new(), same_category, required)
    } else {
        let choose_count = required - same_category.len();
        (same_category, other, choose_count)
    };
    let mut suggestions = Vec::new();
    let mut combination = Vec::with_capacity(choose_count);
    let mut visited = 0;
    visit_card_combinations(
        &choices,
        0,
        choose_count,
        &mut combination,
        &mut visited,
        SEARCH_BUDGET,
        &mut |chosen| {
            let mut cards = fixed.clone();
            cards.extend_from_slice(chosen);
            if let Ok(play) = validate_follow(hand, &cards, lead, trump) {
                suggestions.push(play);
            }
            suggestions.len() >= limit
        },
    );
    suggestions
}

fn visit_card_combinations(
    choices: &[Card],
    start: usize,
    remaining: usize,
    current: &mut Vec<Card>,
    visited: &mut usize,
    budget: usize,
    accept: &mut impl FnMut(&[Card]) -> bool,
) -> bool {
    if *visited >= budget {
        return true;
    }
    if remaining == 0 {
        *visited += 1;
        return accept(current);
    }
    if choices.len().saturating_sub(start) < remaining {
        return false;
    }
    let last_start = choices.len() - remaining;
    for index in start..=last_start {
        current.push(choices[index]);
        if visit_card_combinations(
            choices,
            index + 1,
            remaining - 1,
            current,
            visited,
            budget,
            accept,
        ) {
            current.pop();
            return true;
        }
        current.pop();
    }
    false
}

fn pair_selection_satisfies_runs(
    selected_pairs: &mut Vec<PairUnit>,
    required_runs: &[usize],
) -> bool {
    for run_len in required_runs {
        let run = best_run_at_most(selected_pairs, *run_len);
        if run.len() < *run_len {
            return false;
        }
        remove_pair_indices(selected_pairs, &run);
    }
    true
}

fn classify_mixed_discard(cards: &[Card], trump: Trump) -> ClassifiedPlay {
    let mut cards = cards.to_vec();
    cards.sort_by_key(|card| {
        (
            trump.is_trump(*card),
            strength(*card, trump),
            card.suit(),
            card.deck(),
        )
    });
    let components = cards
        .iter()
        .map(|card| Component::Single {
            card: *card,
            strength: strength(*card, trump),
        })
        .collect();
    ClassifiedPlay {
        cards,
        category: Category::Mixed,
        components,
    }
}

/// 比较两手已合法跟出的牌。返回 `Greater` 表示挑战者成为当前赢家。
pub fn compare_for_trick(
    lead: &ClassifiedPlay,
    current: &ClassifiedPlay,
    challenger: &ClassifiedPlay,
    trump: Trump,
) -> Ordering {
    let current_category = category_precedence(lead.category, current.category);
    let challenger_category = category_precedence(lead.category, challenger.category);
    if challenger_category == 0 {
        return Ordering::Less;
    }
    if special_follow_hierarchy(lead).is_some() {
        let current_key = special_competition_key(lead, current);
        let challenger_key = special_competition_key(lead, challenger);
        return match (current_key, challenger_key) {
            (Some(current), Some(challenger)) => challenger_category
                .cmp(&current_category)
                .then_with(|| challenger.cmp(&current)),
            (None, Some(_)) => Ordering::Greater,
            _ => Ordering::Less,
        };
    }
    let demands = StructureDemands::from_play(lead);
    let current_key = competition_key(&current.cards, &demands, trump, current.category);
    let challenger_key = competition_key(&challenger.cards, &demands, trump, challenger.category);
    match (current_key, challenger_key) {
        (Some(current), Some(challenger)) => challenger_category
            .cmp(&current_category)
            .then_with(|| challenger.cmp(&current)),
        (None, Some(_)) => Ordering::Greater,
        _ => Ordering::Less, // 同牌力及同一实体副本均由先出者保持领先。
    }
}

fn special_competition_key(lead: &ClassifiedPlay, candidate: &ClassifiedPlay) -> Option<Vec<u16>> {
    let single = candidate.components.as_slice();
    match lead.components.as_slice() {
        [Component::Quad { .. }] => match single {
            [Component::Quad { strength, .. }] => Some(vec![1, u16::from(*strength)]),
            _ => None,
        },
        [Component::Tractor { pair_count: 2, .. }] => match single {
            [Component::Quad { strength, .. }] => Some(vec![2, u16::from(*strength)]),
            [
                Component::Tractor {
                    pair_count: 2,
                    top_strength,
                    ..
                },
            ] => Some(vec![1, u16::from(*top_strength)]),
            _ => None,
        },
        [Component::Tractor { pair_count: 3, .. }] => match single {
            [
                Component::Titanic {
                    triple_count: 2,
                    top_strength,
                    ..
                },
            ] => Some(vec![2, u16::from(*top_strength)]),
            [
                Component::Tractor {
                    pair_count: 3,
                    top_strength,
                    ..
                },
            ] => Some(vec![1, u16::from(*top_strength)]),
            _ => None,
        },
        [
            Component::Titanic {
                triple_count: 2, ..
            },
        ] => match single {
            [
                Component::Titanic {
                    triple_count: 2,
                    top_strength,
                    ..
                },
            ] => Some(vec![1, u16::from(*top_strength)]),
            _ => None,
        },
        [Component::Tractor { pair_count: 4, .. }] => match single {
            [
                Component::Spaceship {
                    quad_count: 2,
                    top_strength,
                    ..
                },
            ] => Some(vec![2, u16::from(*top_strength)]),
            [
                Component::Tractor {
                    pair_count: 4,
                    top_strength,
                    ..
                },
            ] => Some(vec![1, u16::from(*top_strength)]),
            _ => None,
        },
        [Component::Spaceship { quad_count: 2, .. }] => match single {
            [
                Component::Spaceship {
                    quad_count: 2,
                    top_strength,
                    ..
                },
            ] => Some(vec![1, u16::from(*top_strength)]),
            _ => None,
        },
        [Component::Quad { .. }, Component::Quad { .. }] => match single {
            [
                Component::Spaceship {
                    quad_count: 2,
                    top_strength,
                    ..
                },
            ] => Some(vec![2, u16::from(*top_strength)]),
            [
                Component::Quad { strength: left, .. },
                Component::Quad {
                    strength: right, ..
                },
            ] => {
                let mut strengths = [u16::from(*left), u16::from(*right)];
                strengths.sort_unstable();
                Some(vec![1, strengths[1], strengths[0]])
            }
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn classify_cards(cards: &[Card], trump: Trump) -> Result<ClassifiedPlay, PlayError> {
    if cards.is_empty() {
        return Err(PlayError::Empty);
    }
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() {
        return Err(PlayError::DuplicatePhysicalCard);
    }
    let play_category = category(cards[0], trump);
    if cards
        .iter()
        .any(|card| category(*card, trump) != play_category)
    {
        return Err(PlayError::MixedCategory);
    }
    let mut cards = cards.to_vec();
    cards.sort_by_key(|card| (strength(*card, trump), card.deck()));
    let components = decompose(&cards, trump, play_category);
    Ok(ClassifiedPlay {
        cards,
        category: play_category,
        components,
    })
}

pub fn category(card: Card, trump: Trump) -> Category {
    if trump.is_trump(card) {
        Category::Trump
    } else {
        Category::Suit(card.suit().expect("non-trump cards are suited"))
    }
}

pub fn strength(card: Card, trump: Trump) -> u8 {
    trump.strength(card)
}

fn decompose(cards: &[Card], trump: Trump, play_category: Category) -> Vec<Component> {
    let mut faces: BTreeMap<(Option<Suit>, Rank), Vec<Card>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    let mut quads = Vec::new();
    let mut triples = Vec::new();
    let mut pairs = Vec::new();
    let mut singles = Vec::new();
    for face_cards in faces.values_mut() {
        face_cards.sort_by_key(|card| card.deck());
        match face_cards.len() {
            4 => quads.push(QuadUnit {
                cards: [face_cards[0], face_cards[1], face_cards[2], face_cards[3]],
                strength: strength(face_cards[0], trump),
            }),
            3 => triples.push(TripleUnit {
                cards: [face_cards[0], face_cards[1], face_cards[2]],
                strength: strength(face_cards[0], trump),
            }),
            2 => pairs.push(PairUnit {
                cards: [face_cards[0], face_cards[1]],
                strength: strength(face_cards[0], trump),
            }),
            1 => singles.push(face_cards[0]),
            _ => unreachable!("一副牌中每个牌面只有一张实体牌"),
        }
    }

    let mut components = Vec::new();
    while let Some(indices) = longest_quad_run(&quads) {
        if indices.len() < 2 {
            break;
        }
        let top_strength = indices
            .iter()
            .map(|index| quads[*index].strength)
            .max()
            .unwrap();
        let mut spaceship_cards = indices
            .iter()
            .flat_map(|index| quads[*index].cards)
            .collect::<Vec<_>>();
        spaceship_cards.sort_by_key(|card| (strength(*card, trump), card.deck()));
        components.push(Component::Spaceship {
            cards: spaceship_cards,
            quad_count: indices.len() as u8,
            top_strength,
        });
        let mut indices = indices;
        indices.sort_unstable();
        for index in indices.into_iter().rev() {
            quads.remove(index);
        }
    }
    while let Some(indices) = longest_triple_run(&triples) {
        if indices.len() < 2 {
            break;
        }
        let top_strength = indices
            .iter()
            .map(|index| triples[*index].strength)
            .max()
            .unwrap();
        let mut titanic_cards = indices
            .iter()
            .flat_map(|index| triples[*index].cards)
            .collect::<Vec<_>>();
        titanic_cards.sort_by_key(|card| (strength(*card, trump), card.deck()));
        components.push(Component::Titanic {
            cards: titanic_cards,
            triple_count: indices.len() as u8,
            top_strength,
        });
        let mut indices = indices;
        indices.sort_unstable();
        for index in indices.into_iter().rev() {
            triples.remove(index);
        }
    }
    while let Some(indices) = longest_pair_run(&pairs) {
        if indices.len() < 2 {
            break;
        }
        let top_strength = indices
            .iter()
            .map(|index| pairs[*index].strength)
            .max()
            .unwrap();
        let mut tractor_cards = indices
            .iter()
            .flat_map(|index| pairs[*index].cards)
            .collect::<Vec<_>>();
        tractor_cards.sort_by_key(|card| (strength(*card, trump), card.deck()));
        components.push(Component::Tractor {
            cards: tractor_cards,
            pair_count: indices.len() as u8,
            top_strength,
        });
        let mut indices = indices;
        indices.sort_unstable();
        for index in indices.into_iter().rev() {
            pairs.remove(index);
        }
    }
    components.extend(pairs.into_iter().map(|pair| Component::Pair {
        cards: pair.cards,
        strength: pair.strength,
    }));
    components.extend(triples.into_iter().map(|triple| Component::Triple {
        cards: triple.cards,
        strength: triple.strength,
    }));
    components.extend(quads.into_iter().map(|quad| Component::Quad {
        cards: quad.cards,
        strength: quad.strength,
    }));
    components.extend(singles.into_iter().map(|card| Component::Single {
        card,
        strength: strength(card, trump),
    }));
    components.sort_by_key(|component| std::cmp::Reverse(component_priority(component)));
    debug_assert!(components.iter().all(|component| {
        component
            .cards()
            .iter()
            .all(|card| category(*card, trump) == play_category)
    }));
    components
}

fn longest_quad_run(quads: &[QuadUnit]) -> Option<Vec<usize>> {
    longest_strength_run(
        &quads.iter().map(|unit| unit.strength).collect::<Vec<_>>(),
        usize::MAX,
    )
}

fn longest_triple_run(triples: &[TripleUnit]) -> Option<Vec<usize>> {
    longest_strength_run(
        &triples.iter().map(|unit| unit.strength).collect::<Vec<_>>(),
        usize::MAX,
    )
}

fn longest_pair_run(pairs: &[PairUnit]) -> Option<Vec<usize>> {
    longest_strength_run(
        &pairs.iter().map(|unit| unit.strength).collect::<Vec<_>>(),
        usize::MAX,
    )
}

fn longest_strength_run(strengths: &[u8], maximum: usize) -> Option<Vec<usize>> {
    let mut best: Vec<usize> = Vec::new();
    for start in 0..strengths.len() {
        let mut run = vec![start];
        let mut next_strength = strengths[start] + 1;
        loop {
            if run.len() >= maximum {
                break;
            }
            let next = strengths
                .iter()
                .enumerate()
                .find(|(index, strength)| !run.contains(index) && **strength == next_strength)
                .map(|(index, _)| index);
            let Some(next) = next else { break };
            run.push(next);
            next_strength += 1;
        }
        let run_top = run.iter().map(|index| strengths[*index]).max();
        let best_top = best.iter().map(|index| strengths[*index]).max();
        if run.len() > best.len() || (run.len() == best.len() && run_top > best_top) {
            best = run;
        }
    }
    (!best.is_empty()).then_some(best)
}

fn component_can_be_beaten(
    component: &Component,
    play_category: Category,
    hand: &[Card],
    trump: Trump,
) -> bool {
    let same_category = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) == play_category)
        .collect::<Vec<_>>();
    match component {
        Component::Single {
            strength: current, ..
        } => same_category
            .iter()
            .any(|card| strength(*card, trump) > *current),
        Component::Pair {
            strength: current, ..
        } => pair_units(&same_category, trump)
            .iter()
            .any(|pair| pair.strength > *current),
        Component::Triple {
            strength: current, ..
        } => triple_units(&same_category, trump)
            .iter()
            .any(|triple| triple.strength > *current),
        Component::Quad {
            strength: current, ..
        } => quad_units(&same_category, trump)
            .iter()
            .any(|quad| quad.strength > *current),
        Component::Tractor {
            pair_count,
            top_strength,
            ..
        } => {
            tractor_tops(&pair_units(&same_category, trump), usize::from(*pair_count))
                .into_iter()
                .any(|top| top > *top_strength)
                || match *pair_count {
                    2 => !quad_units(&same_category, trump).is_empty(),
                    3 => !titanic_tops(&triple_units(&same_category, trump), 2).is_empty(),
                    4 => !spaceship_tops(&quad_units(&same_category, trump), 2).is_empty(),
                    _ => false,
                }
        }
        Component::Titanic {
            triple_count,
            top_strength,
            ..
        } => titanic_tops(
            &triple_units(&same_category, trump),
            usize::from(*triple_count),
        )
        .into_iter()
        .any(|top| top > *top_strength),
        Component::Spaceship {
            quad_count,
            top_strength,
            ..
        } => spaceship_tops(&quad_units(&same_category, trump), usize::from(*quad_count))
            .into_iter()
            .any(|top| top > *top_strength),
    }
}

#[derive(Clone, Debug)]
struct StructureDemands {
    spaceships: Vec<usize>,
    titanics: Vec<usize>,
    tractors: Vec<usize>,
    quads: usize,
    triples: usize,
    pairs: usize,
    singles: usize,
}

impl StructureDemands {
    fn from_play(play: &ClassifiedPlay) -> Self {
        let mut spaceships = Vec::new();
        let mut titanics = Vec::new();
        let mut tractors = Vec::new();
        let mut quads = 0;
        let mut triples = 0;
        let mut pairs = 0;
        let mut singles = 0;
        for component in &play.components {
            match component {
                Component::Single { .. } => singles += 1,
                Component::Pair { .. } => pairs += 1,
                Component::Triple { .. } => triples += 1,
                Component::Quad { .. } => quads += 1,
                Component::Tractor { pair_count, .. } => {
                    tractors.push(usize::from(*pair_count));
                }
                Component::Titanic { triple_count, .. } => {
                    titanics.push(usize::from(*triple_count));
                }
                Component::Spaceship { quad_count, .. } => {
                    spaceships.push(usize::from(*quad_count));
                }
            }
        }
        spaceships.sort_unstable_by(|left, right| right.cmp(left));
        titanics.sort_unstable_by(|left, right| right.cmp(left));
        tractors.sort_unstable_by(|left, right| right.cmp(left));
        Self {
            spaceships,
            titanics,
            tractors,
            quads,
            triples,
            pairs,
            singles,
        }
    }

    fn pair_slots(&self) -> usize {
        self.spaceships.iter().sum::<usize>() * 2 + self.tractors.iter().sum::<usize>() + self.pairs
    }
}

fn satisfies_follow_structure(
    hand: &[Card],
    proposed: &[Card],
    demands: &StructureDemands,
    trump: Trump,
    play_category: Category,
) -> bool {
    let mut hand_quads = quad_units(hand, trump);
    let mut required_spaceships = Vec::new();
    for demand in &demands.spaceships {
        let best = best_quad_run_at_most(&hand_quads, *demand);
        if best.len() >= 2 {
            required_spaceships.push(best.len());
            remove_quad_indices(&mut hand_quads, &best);
        }
    }
    let available_quad_count = quad_units(hand, trump).len();
    let required_quad_count =
        (demands.spaceships.iter().sum::<usize>() + demands.quads).min(available_quad_count);
    let mut proposed_quads = quad_units(proposed, trump);
    if proposed_quads.len() < required_quad_count {
        return false;
    }
    for run_len in required_spaceships {
        let run = best_quad_run_at_most(&proposed_quads, run_len);
        if run.len() < run_len {
            return false;
        }
        remove_quad_indices(&mut proposed_quads, &run);
    }

    let mut hand_triples = triple_units(hand, trump);
    let mut required_titanics = Vec::new();
    for demand in &demands.titanics {
        let best = best_triple_run_at_most(&hand_triples, *demand);
        if best.len() >= 2 {
            required_titanics.push(best.len());
            remove_triple_indices(&mut hand_triples, &best);
        }
    }
    let available_triple_count = triple_units(hand, trump).len();
    let required_triple_count =
        (demands.titanics.iter().sum::<usize>() + demands.triples).min(available_triple_count);
    let mut proposed_triples = triple_units(proposed, trump);
    if proposed_triples.len() < required_triple_count {
        return false;
    }
    for run_len in required_titanics {
        let run = best_triple_run_at_most(&proposed_triples, run_len);
        if run.len() < run_len {
            return false;
        }
        remove_triple_indices(&mut proposed_triples, &run);
    }

    let mut hand_pairs = pair_units(hand, trump);
    let mut required_runs = Vec::new();
    for demand in &demands.tractors {
        let best = best_run_at_most(&hand_pairs, *demand);
        if best.len() >= 2 {
            required_runs.push(best.len());
            remove_pair_indices(&mut hand_pairs, &best);
        }
    }
    let available_pair_count = pair_units(hand, trump).len();
    let required_pair_count = demands.pair_slots().min(available_pair_count);
    let proposed_pairs = pair_units(proposed, trump);
    if proposed_pairs.len() < required_pair_count {
        return false;
    }
    let mut proposed_remaining = proposed_pairs;
    for run_len in required_runs {
        let run = best_run_at_most(&proposed_remaining, run_len);
        if run.len() < run_len {
            return false;
        }
        remove_pair_indices(&mut proposed_remaining, &run);
    }
    proposed
        .iter()
        .all(|card| category(*card, trump) == play_category)
}

fn competition_key(
    cards: &[Card],
    demands: &StructureDemands,
    trump: Trump,
    play_category: Category,
) -> Option<Vec<u8>> {
    let mut used = HashSet::new();
    let mut key = Vec::new();
    for demand in &demands.spaceships {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let quads = quad_units(&available, trump);
        let indices = highest_quad_run_exact(&quads, *demand)?;
        key.push(indices.iter().map(|index| quads[*index].strength).max()?);
        for index in indices {
            used.extend(quads[index].cards);
        }
    }
    for _ in 0..demands.quads {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let quad = quad_units(&available, trump)
            .into_iter()
            .max_by_key(|quad| quad.strength)?;
        key.push(quad.strength);
        used.extend(quad.cards);
    }
    for demand in &demands.titanics {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let triples = triple_units(&available, trump);
        let indices = highest_triple_run_exact(&triples, *demand)?;
        key.push(indices.iter().map(|index| triples[*index].strength).max()?);
        for index in indices {
            used.extend(triples[index].cards);
        }
    }
    for _ in 0..demands.triples {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let triple = triple_units(&available, trump)
            .into_iter()
            .max_by_key(|triple| triple.strength)?;
        key.push(triple.strength);
        used.extend(triple.cards);
    }
    let available = cards
        .iter()
        .copied()
        .filter(|card| !used.contains(card))
        .collect::<Vec<_>>();
    let mut pairs = pair_units(&available, trump);
    for demand in &demands.tractors {
        let indices = highest_run_exact(&pairs, *demand)?;
        key.push(indices.iter().map(|index| pairs[*index].strength).max()?);
        for index in indices.iter().rev() {
            for card in pairs[*index].cards {
                used.insert(card);
            }
        }
        remove_pair_indices(&mut pairs, &indices);
    }
    for _ in 0..demands.pairs {
        let (index, pair) = pairs
            .iter()
            .enumerate()
            .max_by_key(|(_, pair)| pair.strength)?;
        key.push(pair.strength);
        for card in pair.cards {
            used.insert(card);
        }
        pairs.remove(index);
    }
    let mut singles = cards
        .iter()
        .copied()
        .filter(|card| !used.contains(card))
        .map(|card| strength(card, trump))
        .collect::<Vec<_>>();
    if singles.len() < demands.singles {
        return None;
    }
    singles.sort_unstable_by(|left, right| right.cmp(left));
    key.extend(singles.into_iter().take(demands.singles));
    cards
        .iter()
        .all(|card| category(*card, trump) == play_category)
        .then_some(key)
}

fn category_precedence(lead: Category, candidate: Category) -> u8 {
    match (lead, candidate) {
        (Category::Trump, Category::Trump) => 2,
        (Category::Suit(_), Category::Trump) => 3,
        (Category::Suit(lead), Category::Suit(candidate)) if lead == candidate => 2,
        _ => 0,
    }
}

fn pair_units(cards: &[Card], trump: Trump) -> Vec<PairUnit> {
    let mut faces: BTreeMap<(Option<Suit>, Rank), Vec<Card>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    faces
        .into_values()
        .filter_map(|mut cards| {
            if cards.len() < 2 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(PairUnit {
                cards: [cards[0], cards[1]],
                strength: strength(cards[0], trump),
            })
        })
        .collect()
}

fn triple_units(cards: &[Card], trump: Trump) -> Vec<TripleUnit> {
    let mut faces: BTreeMap<(Option<Suit>, Rank), Vec<Card>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    faces
        .into_values()
        .filter_map(|mut cards| {
            if cards.len() < 3 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(TripleUnit {
                cards: [cards[0], cards[1], cards[2]],
                strength: strength(cards[0], trump),
            })
        })
        .collect()
}

fn quad_units(cards: &[Card], trump: Trump) -> Vec<QuadUnit> {
    let mut faces: BTreeMap<(Option<Suit>, Rank), Vec<Card>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    faces
        .into_values()
        .filter_map(|mut cards| {
            if cards.len() < 4 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(QuadUnit {
                cards: [cards[0], cards[1], cards[2], cards[3]],
                strength: strength(cards[0], trump),
            })
        })
        .collect()
}

fn tractor_tops(pairs: &[PairUnit], length: usize) -> Vec<u8> {
    let mut tops = Vec::new();
    for pair in pairs {
        let start = pair.strength;
        if (0..length).all(|offset| {
            pairs
                .iter()
                .any(|candidate| candidate.strength == start + offset as u8)
        }) {
            tops.push(start + length as u8 - 1);
        }
    }
    tops
}

fn titanic_tops(triples: &[TripleUnit], length: usize) -> Vec<u8> {
    let mut tops = Vec::new();
    for triple in triples {
        let start = triple.strength;
        if (0..length).all(|offset| {
            triples
                .iter()
                .any(|candidate| candidate.strength == start + offset as u8)
        }) {
            tops.push(start + length as u8 - 1);
        }
    }
    tops
}

fn spaceship_tops(quads: &[QuadUnit], length: usize) -> Vec<u8> {
    let mut tops = Vec::new();
    for quad in quads {
        let start = quad.strength;
        if (0..length).all(|offset| {
            quads
                .iter()
                .any(|candidate| candidate.strength == start + offset as u8)
        }) {
            tops.push(start + length as u8 - 1);
        }
    }
    tops
}

fn titanic_card_sets(cards: &[Card], trump: Trump, length: usize) -> Vec<Vec<Card>> {
    let triples = triple_units(cards, trump);
    let mut candidates = Vec::new();
    for start in triples.iter().map(|triple| triple.strength) {
        collect_titanic_card_sets(
            &triples,
            start,
            length,
            0,
            &mut Vec::new(),
            &mut candidates,
            trump,
        );
    }
    candidates.sort_by_key(|cards| {
        cards
            .iter()
            .map(|card| (card.rank(), card.suit(), card.deck()))
            .collect::<Vec<_>>()
    });
    candidates.dedup();
    candidates
}

fn collect_titanic_card_sets(
    triples: &[TripleUnit],
    start: u8,
    length: usize,
    offset: usize,
    current: &mut Vec<Card>,
    output: &mut Vec<Vec<Card>>,
    trump: Trump,
) {
    if offset == length {
        let mut cards = current.clone();
        cards.sort_by_key(|card| (strength(*card, trump), card.deck()));
        output.push(cards);
        return;
    }
    for triple in triples
        .iter()
        .filter(|triple| triple.strength == start + offset as u8)
    {
        current.extend(triple.cards);
        collect_titanic_card_sets(triples, start, length, offset + 1, current, output, trump);
        current.truncate(current.len() - 3);
    }
}

fn highest_run_exact(pairs: &[PairUnit], length: usize) -> Option<Vec<usize>> {
    let mut best: Option<Vec<usize>> = None;
    for start in pairs {
        let indices = (0..length)
            .map(|offset| {
                pairs
                    .iter()
                    .position(|pair| pair.strength == start.strength + offset as u8)
            })
            .collect::<Option<Vec<_>>>();
        if let Some(indices) = indices {
            let top = indices.iter().map(|index| pairs[*index].strength).max();
            let best_top = best
                .as_ref()
                .and_then(|best| best.iter().map(|index| pairs[*index].strength).max());
            if top > best_top {
                best = Some(indices);
            }
        }
    }
    best
}

fn highest_triple_run_exact(triples: &[TripleUnit], length: usize) -> Option<Vec<usize>> {
    let strengths = triples
        .iter()
        .map(|triple| triple.strength)
        .collect::<Vec<_>>();
    highest_strength_run_exact(&strengths, length)
}

fn highest_quad_run_exact(quads: &[QuadUnit], length: usize) -> Option<Vec<usize>> {
    let strengths = quads.iter().map(|quad| quad.strength).collect::<Vec<_>>();
    highest_strength_run_exact(&strengths, length)
}

fn highest_strength_run_exact(strengths: &[u8], length: usize) -> Option<Vec<usize>> {
    let mut best = None::<Vec<usize>>;
    for start in strengths {
        let indices = (0..length)
            .map(|offset| {
                strengths
                    .iter()
                    .position(|strength| *strength == *start + offset as u8)
            })
            .collect::<Option<Vec<_>>>();
        if let Some(indices) = indices {
            let top = indices.iter().map(|index| strengths[*index]).max();
            let best_top = best
                .as_ref()
                .and_then(|best| best.iter().map(|index| strengths[*index]).max());
            if top > best_top {
                best = Some(indices);
            }
        }
    }
    best
}

fn best_run_at_most(pairs: &[PairUnit], maximum: usize) -> Vec<usize> {
    for length in (2..=maximum).rev() {
        if let Some(run) = highest_run_exact(pairs, length) {
            return run;
        }
    }
    Vec::new()
}

fn best_triple_run_at_most(triples: &[TripleUnit], maximum: usize) -> Vec<usize> {
    for length in (2..=maximum).rev() {
        if let Some(run) = highest_triple_run_exact(triples, length) {
            return run;
        }
    }
    Vec::new()
}

fn best_quad_run_at_most(quads: &[QuadUnit], maximum: usize) -> Vec<usize> {
    for length in (2..=maximum).rev() {
        if let Some(run) = highest_quad_run_exact(quads, length) {
            return run;
        }
    }
    Vec::new()
}

fn remove_pair_indices(pairs: &mut Vec<PairUnit>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        pairs.remove(index);
    }
}

fn remove_triple_indices(triples: &mut Vec<TripleUnit>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        triples.remove(index);
    }
}

fn remove_quad_indices(quads: &mut Vec<QuadUnit>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        quads.remove(index);
    }
}

fn validate_owned(cards: &[Card], hand: &[Card]) -> Result<(), PlayError> {
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() {
        return Err(PlayError::DuplicatePhysicalCard);
    }
    if cards.iter().any(|card| !hand.contains(card)) {
        return Err(PlayError::CardsNotOwned);
    }
    Ok(())
}

fn component_priority(component: &Component) -> (u8, usize, u8) {
    match component {
        Component::Single { strength, .. } => (0, 1, *strength),
        Component::Pair { strength, .. } => (1, 1, *strength),
        Component::Triple { strength, .. } => (2, 1, *strength),
        Component::Quad { strength, .. } => (3, 1, *strength),
        Component::Tractor {
            pair_count,
            top_strength,
            ..
        } => (4, usize::from(*pair_count), *top_strength),
        Component::Titanic {
            triple_count,
            top_strength,
            ..
        } => (5, usize::from(*triple_count), *top_strength),
        Component::Spaceship {
            quad_count,
            top_strength,
            ..
        } => (6, usize::from(*quad_count), *top_strength),
    }
}

fn component_kind_order(component: &Component) -> u8 {
    match component {
        Component::Single { .. } => 0,
        Component::Pair { .. } => 1,
        Component::Triple { .. } => 2,
        Component::Quad { .. } => 3,
        Component::Tractor { .. } => 4,
        Component::Titanic { .. } => 5,
        Component::Spaceship { .. } => 6,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayError {
    Empty,
    DuplicatePhysicalCard,
    CardsNotOwned,
    MixedCategory,
    ThrowDisabled,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("出牌不能为空"),
            Self::DuplicatePhysicalCard => f.write_str("同一张实体牌不能重复提交"),
            Self::CardsNotOwned => f.write_str("提交的牌不全在玩家手中"),
            Self::MixedCategory => f.write_str("首家只能出同一门副牌或全部主牌"),
            Self::ThrowDisabled => f.write_str("当前规则不允许甩牌"),
        }
    }
}

impl std::error::Error for PlayError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FollowError {
    Play(PlayError),
    WrongCardCount { expected: usize, actual: usize },
    MustFollowCategory { required: usize, actual: usize },
    MustFollowStructure,
}

impl fmt::Display for FollowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Play(error) => error.fmt(f),
            Self::WrongCardCount { expected, actual } => {
                write!(f, "必须跟出 {expected} 张牌，实际为 {actual}")
            }
            Self::MustFollowCategory { required, actual } => {
                write!(f, "必须跟出本门牌 {required} 张，实际为 {actual}")
            }
            Self::MustFollowStructure => {
                f.write_str("必须优先跟泰坦尼克、拖拉机、三同张和对子结构")
            }
        }
    }
}

impl std::error::Error for FollowError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ThrowPenalty, Trump};

    fn card(deck: u8, suit: Suit, rank: Rank) -> Card {
        Card::suited(deck, suit, rank)
    }

    fn pair(suit: Suit, rank: Rank) -> [Card; 2] {
        [card(0, suit, rank), card(1, suit, rank)]
    }

    fn triple(suit: Suit, rank: Rank) -> [Card; 3] {
        [
            card(0, suit, rank),
            card(1, suit, rank),
            card(2, suit, rank),
        ]
    }

    fn quad(suit: Suit, rank: Rank) -> [Card; 4] {
        [
            card(0, suit, rank),
            card(1, suit, rank),
            card(2, suit, rank),
            card(3, suit, rank),
        ]
    }

    fn trump() -> Trump {
        Trump::new(Rank::Ten, Some(Suit::Heart)).unwrap()
    }

    #[test]
    fn three_deck_triples_form_titanics_across_skipped_and_special_levels() {
        let level_five = Trump::new(Rank::Five, Some(Suit::Heart)).unwrap();
        for cards in [
            [
                triple(Suit::Spade, Rank::Three).as_slice(),
                triple(Suit::Spade, Rank::Four).as_slice(),
            ]
            .concat(),
            [
                triple(Suit::Spade, Rank::Four).as_slice(),
                triple(Suit::Spade, Rank::Six).as_slice(),
            ]
            .concat(),
            [
                triple(Suit::Heart, Rank::Ace).as_slice(),
                triple(Suit::Spade, Rank::Five).as_slice(),
            ]
            .concat(),
            [
                triple(Suit::Spade, Rank::Five).as_slice(),
                triple(Suit::Heart, Rank::Five).as_slice(),
            ]
            .concat(),
            [
                triple(Suit::Heart, Rank::Five).as_slice(),
                &[
                    Card::small_joker(0),
                    Card::small_joker(1),
                    Card::small_joker(2),
                ],
            ]
            .concat(),
        ] {
            let play = classify_cards(&cards, level_five).unwrap();
            assert!(matches!(
                play.components.as_slice(),
                [Component::Titanic {
                    triple_count: 2,
                    ..
                }]
            ));
            assert_eq!(play.kitty_multiplier(), 16);
        }

        let split_category = [
            triple(Suit::Spade, Rank::Five).as_slice(),
            triple(Suit::Spade, Rank::Six).as_slice(),
        ]
        .concat();
        assert_eq!(
            classify_cards(&split_category, level_five),
            Err(PlayError::MixedCategory)
        );

        let no_trump = Trump::new(Rank::Five, None).unwrap();
        let same_strength = [
            triple(Suit::Diamond, Rank::Five).as_slice(),
            triple(Suit::Club, Rank::Five).as_slice(),
        ]
        .concat();
        let play = classify_cards(&same_strength, no_trump).unwrap();
        assert!(
            play.components
                .iter()
                .all(|component| matches!(component, Component::Triple { .. }))
        );
    }

    #[test]
    fn four_identical_faces_form_bombs_and_adjacent_bombs_form_spaceships() {
        let bombs = [
            quad(Suit::Spade, Rank::Three).as_slice(),
            quad(Suit::Spade, Rank::Four).as_slice(),
        ]
        .concat();
        let play = classify_cards(&bombs, trump()).unwrap();
        assert!(matches!(
            play.components.as_slice(),
            [Component::Spaceship { quad_count: 2, .. }]
        ));
        assert_eq!(play.kitty_multiplier(), 64);

        let single_bomb = classify_cards(&quad(Suit::Spade, Rank::Ace), trump()).unwrap();
        assert!(matches!(
            single_bomb.components.as_slice(),
            [Component::Quad { .. }]
        ));
        assert_eq!(single_bomb.kitty_multiplier(), 16);
    }

    #[test]
    fn equal_strength_off_suit_level_cards_never_merge_into_a_bomb() {
        let no_trump = Trump::new(Rank::Ten, None).unwrap();
        let mixed_faces = [
            pair(Suit::Spade, Rank::Ten).as_slice(),
            pair(Suit::Heart, Rank::Ten).as_slice(),
        ]
        .concat();
        let play = classify_cards(&mixed_faces, no_trump).unwrap();
        assert_eq!(play.components.len(), 2);
        assert!(
            play.components
                .iter()
                .all(|component| matches!(component, Component::Pair { .. }))
        );
    }

    #[test]
    fn bomb_and_two_pair_tractor_share_follow_tier_but_bomb_wins() {
        let lead_cards = [
            pair(Suit::Spade, Rank::Six).as_slice(),
            pair(Suit::Spade, Rank::Seven).as_slice(),
        ]
        .concat();
        let lead = classify_cards(&lead_cards, trump()).unwrap();
        let bomb = quad(Suit::Spade, Rank::Three);
        let tractor_cards = [
            pair(Suit::Spade, Rank::Eight).as_slice(),
            pair(Suit::Spade, Rank::Nine).as_slice(),
        ]
        .concat();
        let hand = [bomb.as_slice(), tractor_cards.as_slice()].concat();
        let bomb_play = validate_follow(&hand, &bomb, &lead, trump()).unwrap();
        let tractor_play = validate_follow(&hand, &tractor_cards, &lead, trump()).unwrap();
        assert_eq!(
            compare_for_trick(&lead, &tractor_play, &bomb_play, trump()),
            Ordering::Greater
        );
        assert_eq!(
            compare_for_trick(&lead, &bomb_play, &tractor_play, trump()),
            Ordering::Less
        );
    }

    #[test]
    fn a_led_bomb_requires_a_bomb_before_a_tractor() {
        let lead = classify_cards(&quad(Suit::Spade, Rank::Ace), trump()).unwrap();
        let bomb = quad(Suit::Spade, Rank::Three);
        let tractor_cards = [
            pair(Suit::Spade, Rank::Six).as_slice(),
            pair(Suit::Spade, Rank::Seven).as_slice(),
        ]
        .concat();
        let hand = [bomb.as_slice(), tractor_cards.as_slice()].concat();
        assert_eq!(
            validate_follow(&hand, &tractor_cards, &lead, trump()),
            Err(FollowError::MustFollowStructure)
        );
        assert!(validate_follow(&hand, &bomb, &lead, trump()).is_ok());
    }

    #[test]
    fn spaceship_cross_beats_a_four_pair_tractor_and_is_mandatory() {
        let lead_cards = [
            pair(Suit::Spade, Rank::Three).as_slice(),
            pair(Suit::Spade, Rank::Four).as_slice(),
            pair(Suit::Spade, Rank::Five).as_slice(),
            pair(Suit::Spade, Rank::Six).as_slice(),
        ]
        .concat();
        let lead = classify_cards(&lead_cards, trump()).unwrap();
        let spaceship = [
            quad(Suit::Spade, Rank::Seven).as_slice(),
            quad(Suit::Spade, Rank::Eight).as_slice(),
        ]
        .concat();
        let lower_shape = [
            pair(Suit::Spade, Rank::Jack).as_slice(),
            pair(Suit::Spade, Rank::Queen).as_slice(),
            pair(Suit::Spade, Rank::King).as_slice(),
            pair(Suit::Spade, Rank::Ace).as_slice(),
        ]
        .concat();
        let hand = [spaceship.as_slice(), lower_shape.as_slice()].concat();
        assert_eq!(
            validate_follow(&hand, &lower_shape, &lead, trump()),
            Err(FollowError::MustFollowStructure)
        );
        let spaceship_play = validate_follow(&hand, &spaceship, &lead, trump()).unwrap();
        assert_eq!(
            compare_for_trick(&lead, &lead, &spaceship_play, trump()),
            Ordering::Greater
        );

        let attempted = [lead_cards.as_slice(), &[card(0, Suit::Spade, Rank::Ace)]].concat();
        let TrickPlay::ThrowFailed(failure) =
            classify_lead(&attempted, trump(), &RuleSet::default(), &[&spaceship]).unwrap()
        else {
            panic!("甩出的四连对应被宇宙飞船击破");
        };
        assert!(matches!(
            failure.forced.components.as_slice(),
            [Component::Tractor { pair_count: 4, .. }]
        ));
    }

    #[test]
    fn four_pair_tractor_follow_hierarchy_keeps_the_declared_order() {
        let hierarchy = eight_card_tractor_hierarchy();
        let spaceship = [
            quad(Suit::Spade, Rank::Three).as_slice(),
            quad(Suit::Spade, Rank::Four).as_slice(),
        ]
        .concat();
        let titanic_pair = [
            triple(Suit::Spade, Rank::Three).as_slice(),
            triple(Suit::Spade, Rank::Four).as_slice(),
            pair(Suit::Spade, Rank::Eight).as_slice(),
        ]
        .concat();
        let bomb_pairs = [
            quad(Suit::Spade, Rank::Two).as_slice(),
            pair(Suit::Spade, Rank::Four).as_slice(),
            pair(Suit::Spade, Rank::Six).as_slice(),
        ]
        .concat();
        let triples = [
            triple(Suit::Spade, Rank::Two).as_slice(),
            triple(Suit::Spade, Rank::Four).as_slice(),
            &[
                card(0, Suit::Spade, Rank::Six),
                card(0, Suit::Spade, Rank::Eight),
            ],
        ]
        .concat();
        let four_pairs = [Rank::Two, Rank::Four, Rank::Six, Rank::Eight]
            .into_iter()
            .flat_map(|rank| pair(Suit::Spade, rank))
            .collect::<Vec<_>>();
        let singles = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
        ]
        .map(|rank| card(0, Suit::Spade, rank));

        assert_eq!(
            best_follow_tier(&spaceship, trump(), &hierarchy, true),
            Some(0)
        );
        assert_eq!(
            best_follow_tier(&titanic_pair, trump(), &hierarchy, true),
            Some(1)
        );
        assert_eq!(
            best_follow_tier(&bomb_pairs, trump(), &hierarchy, true),
            Some(6)
        );
        assert_eq!(
            best_follow_tier(&triples, trump(), &hierarchy, true),
            Some(9)
        );
        assert_eq!(
            best_follow_tier(&four_pairs, trump(), &hierarchy, true),
            Some(13)
        );
        assert_eq!(
            best_follow_tier(&singles, trump(), &hierarchy, true),
            Some(17)
        );
    }

    #[test]
    fn two_bombs_are_a_structured_lead_and_spaceship_has_first_follow_priority() {
        let lead_cards = [
            quad(Suit::Spade, Rank::Two).as_slice(),
            quad(Suit::Spade, Rank::Four).as_slice(),
        ]
        .concat();
        let lead = classify_cards(&lead_cards, trump()).unwrap();
        assert!(!lead.is_throw());
        let spaceship = [
            quad(Suit::Spade, Rank::Six).as_slice(),
            quad(Suit::Spade, Rank::Seven).as_slice(),
        ]
        .concat();
        let other_bombs = [
            quad(Suit::Spade, Rank::Eight).as_slice(),
            quad(Suit::Spade, Rank::Jack).as_slice(),
        ]
        .concat();
        let hand = [spaceship.as_slice(), other_bombs.as_slice()].concat();
        assert_eq!(
            validate_follow(&hand, &other_bombs, &lead, trump()),
            Err(FollowError::MustFollowStructure)
        );
        let spaceship_play = validate_follow(&hand, &spaceship, &lead, trump()).unwrap();
        assert_eq!(
            compare_for_trick(&lead, &lead, &spaceship_play, trump()),
            Ordering::Greater
        );
    }

    #[test]
    fn bomb_plus_pair_is_below_tractor_and_titanic_for_six_card_following() {
        let lead_cards = [
            pair(Suit::Spade, Rank::Six).as_slice(),
            pair(Suit::Spade, Rank::Seven).as_slice(),
            pair(Suit::Spade, Rank::Eight).as_slice(),
        ]
        .concat();
        let lead = classify_cards(&lead_cards, trump()).unwrap();
        let titanic = [
            triple(Suit::Spade, Rank::Two).as_slice(),
            triple(Suit::Spade, Rank::Three).as_slice(),
        ]
        .concat();
        let bomb_plus_pair = [
            quad(Suit::Spade, Rank::King).as_slice(),
            pair(Suit::Spade, Rank::Ace).as_slice(),
        ]
        .concat();
        let hand = [titanic.as_slice(), bomb_plus_pair.as_slice()].concat();
        assert_eq!(
            validate_follow(&hand, &bomb_plus_pair, &lead, trump()),
            Err(FollowError::MustFollowStructure)
        );
        assert!(validate_follow(&hand, &titanic, &lead, trump()).is_ok());
    }

    #[test]
    fn triple_and_titanic_follow_priorities_are_enforced() {
        let lead_triple = classify_cards(&triple(Suit::Spade, Rank::King), trump()).unwrap();
        let sixes = triple(Suit::Spade, Rank::Six);
        let twos = pair(Suit::Spade, Rank::Two);
        let triple_hand = [sixes.as_slice(), twos.as_slice()].concat();
        let pair_follow = [twos.as_slice(), &sixes[..1]].concat();
        assert_eq!(
            validate_follow(&triple_hand, &pair_follow, &lead_triple, trump()),
            Err(FollowError::MustFollowStructure)
        );
        assert!(validate_follow(&triple_hand, &sixes, &lead_triple, trump()).is_ok());

        let lead_titanic_cards = [
            triple(Suit::Spade, Rank::Three).as_slice(),
            triple(Suit::Spade, Rank::Four).as_slice(),
        ]
        .concat();
        let lead_titanic = classify_cards(&lead_titanic_cards, trump()).unwrap();
        let six_pair = pair(Suit::Spade, Rank::Six);
        let seven_pair = pair(Suit::Spade, Rank::Seven);
        let two_triple = triple(Suit::Spade, Rank::Two);
        let eight_triple = triple(Suit::Spade, Rank::Eight);
        let hand = [
            six_pair.as_slice(),
            seven_pair.as_slice(),
            two_triple.as_slice(),
            eight_triple.as_slice(),
        ]
        .concat();
        let two_triples = [two_triple.as_slice(), eight_triple.as_slice()].concat();
        assert_eq!(
            validate_follow(&hand, &two_triples, &lead_titanic, trump()),
            Err(FollowError::MustFollowStructure)
        );
        let tractor_and_singles = [
            six_pair.as_slice(),
            seven_pair.as_slice(),
            &two_triple[..1],
            &eight_triple[..1],
        ]
        .concat();
        assert!(validate_follow(&hand, &tractor_and_singles, &lead_titanic, trump()).is_ok());
    }

    #[test]
    fn two_link_titanic_and_three_pair_tractor_are_optional_but_titanic_wins() {
        let lead_cards = [
            pair(Suit::Spade, Rank::Six).as_slice(),
            pair(Suit::Spade, Rank::Seven).as_slice(),
            pair(Suit::Spade, Rank::Eight).as_slice(),
        ]
        .concat();
        let lead = classify_cards(&lead_cards, trump()).unwrap();
        let titanic = [
            triple(Suit::Spade, Rank::Two).as_slice(),
            triple(Suit::Spade, Rank::Three).as_slice(),
        ]
        .concat();
        let ordinary_tractor = [
            pair(Suit::Spade, Rank::Four).as_slice(),
            pair(Suit::Spade, Rank::Five).as_slice(),
            pair(Suit::Spade, Rank::Six).as_slice(),
        ]
        .concat();
        let hand = [titanic.as_slice(), ordinary_tractor.as_slice()].concat();

        assert!(validate_follow(&hand, &ordinary_tractor, &lead, trump()).is_ok());
        let titanic_play = validate_follow(&hand, &titanic, &lead, trump()).unwrap();
        assert!(forced_follow_cards(&hand, &lead, trump()).is_empty());
        let suggestions = follow_suggestions(&hand, &lead, trump(), 8);
        assert!(suggestions.iter().any(|play| matches!(
            play.components.as_slice(),
            [Component::Tractor { pair_count: 3, .. }]
        )));
        assert!(suggestions.iter().any(|play| matches!(
            play.components.as_slice(),
            [Component::Titanic {
                triple_count: 2,
                ..
            }]
        )));
        assert_eq!(
            compare_for_trick(&lead, &lead, &titanic_play, trump()),
            Ordering::Greater
        );

        let attempted = [lead_cards.as_slice(), &[card(0, Suit::Spade, Rank::Ace)]].concat();
        let TrickPlay::ThrowFailed(failure) =
            classify_lead(&attempted, trump(), &RuleSet::default(), &[&titanic]).unwrap()
        else {
            panic!("甩出的三连对拖拉机应被泰坦尼克击破");
        };
        assert!(matches!(
            failure.forced.components.as_slice(),
            [Component::Tractor { pair_count: 3, .. }]
        ));
    }

    #[test]
    fn level_is_skipped_and_special_trump_pairs_form_tractors() {
        let jj99 = [pair(Suit::Spade, Rank::Jack), pair(Suit::Spade, Rank::Nine)].concat();
        let play = classify_cards(&jj99, trump()).unwrap();
        assert!(matches!(
            play.components.as_slice(),
            [Component::Tractor { pair_count: 2, .. }]
        ));

        let special = [
            pair(Suit::Heart, Rank::Ace).as_slice(),
            pair(Suit::Spade, Rank::Ten).as_slice(),
            pair(Suit::Heart, Rank::Ten).as_slice(),
            &[Card::small_joker(0), Card::small_joker(1)],
            &[Card::big_joker(0), Card::big_joker(1)],
        ]
        .concat();
        let play = classify_cards(&special, trump()).unwrap();
        assert!(matches!(
            play.components.as_slice(),
            [Component::Tractor { pair_count: 5, .. }]
        ));
    }

    #[test]
    fn constant_twos_extend_the_special_trump_tractor_chain() {
        let trump = trump().with_constant_trump(true);
        let special = [
            pair(Suit::Heart, Rank::Ace).as_slice(),
            pair(Suit::Spade, Rank::Two).as_slice(),
            pair(Suit::Heart, Rank::Two).as_slice(),
            pair(Suit::Spade, Rank::Ten).as_slice(),
            pair(Suit::Heart, Rank::Ten).as_slice(),
            &[Card::small_joker(0), Card::small_joker(1)],
            &[Card::big_joker(0), Card::big_joker(1)],
        ]
        .concat();
        assert!(matches!(
            classify_cards(&special, trump)
                .unwrap()
                .components
                .as_slice(),
            [Component::Tractor { pair_count: 7, .. }]
        ));
    }

    #[test]
    fn no_trump_small_jokers_connect_level_pairs_and_big_jokers() {
        let no_trump = Trump::new(Rank::Ten, None).unwrap();
        let cards = [
            pair(Suit::Diamond, Rank::Ten).as_slice(),
            &[Card::small_joker(0), Card::small_joker(1)],
            &[Card::big_joker(0), Card::big_joker(1)],
        ]
        .concat();
        assert!(matches!(
            classify_cards(&cards, no_trump)
                .unwrap()
                .components
                .as_slice(),
            [Component::Tractor { pair_count: 3, .. }]
        ));
    }

    #[test]
    fn failed_throw_selects_the_shortest_beatable_then_weakest_component() {
        let attempted = [
            card(0, Suit::Spade, Rank::Nine),
            pair(Suit::Spade, Rank::Eight)[0],
            pair(Suit::Spade, Rank::Eight)[1],
            pair(Suit::Spade, Rank::Four)[0],
            pair(Suit::Spade, Rank::Four)[1],
        ];
        let opponent = [
            card(0, Suit::Spade, Rank::Jack),
            pair(Suit::Spade, Rank::Five)[0],
        ];
        let rules = RuleSet {
            throw_penalty: ThrowPenalty::TenPerCard,
            ..RuleSet::default()
        };
        let TrickPlay::ThrowFailed(failure) =
            classify_lead(&attempted, trump(), &rules, &[&opponent]).unwrap()
        else {
            panic!("throw should fail");
        };
        assert_eq!(failure.forced.cards, vec![attempted[0]]);
        assert_eq!(failure.penalty_points, 50);
    }

    #[test]
    fn failed_throw_uses_a_pair_when_its_single_is_unbeatable() {
        let attempted = [
            card(0, Suit::Spade, Rank::Ace),
            pair(Suit::Spade, Rank::Eight)[0],
            pair(Suit::Spade, Rank::Eight)[1],
            pair(Suit::Spade, Rank::Four)[0],
            pair(Suit::Spade, Rank::Four)[1],
        ];
        let opponent = pair(Suit::Spade, Rank::Five);
        let TrickPlay::ThrowFailed(failure) =
            classify_lead(&attempted, trump(), &RuleSet::default(), &[&opponent]).unwrap()
        else {
            panic!("the low pair should make the throw fail");
        };
        assert_eq!(failure.forced.cards, pair(Suit::Spade, Rank::Four));
    }

    #[test]
    fn pair_and_tractor_follow_obligations_are_enforced() {
        let lead_pair = classify_cards(&pair(Suit::Spade, Rank::Five), trump()).unwrap();
        let hand = [
            pair(Suit::Spade, Rank::Four).as_slice(),
            &[card(0, Suit::Spade, Rank::Ace)],
        ]
        .concat();
        assert_eq!(
            validate_follow(
                &hand,
                &[
                    card(0, Suit::Spade, Rank::Ace),
                    pair(Suit::Spade, Rank::Four)[0]
                ],
                &lead_pair,
                trump()
            ),
            Err(FollowError::MustFollowStructure)
        );

        let lead_tractor = classify_cards(
            &[
                pair(Suit::Spade, Rank::King),
                pair(Suit::Spade, Rank::Queen),
            ]
            .concat(),
            trump(),
        )
        .unwrap();
        let follow = [
            pair(Suit::Spade, Rank::Eight),
            pair(Suit::Spade, Rank::Seven),
        ]
        .concat();
        let hand = [follow.clone(), pair(Suit::Spade, Rank::Four).to_vec()].concat();
        assert!(validate_follow(&hand, &follow, &lead_tractor, trump()).is_ok());
    }

    #[test]
    fn forced_follow_cards_selects_the_only_pair_and_all_short_suit_cards() {
        let lead_tractor = classify_cards(
            &[
                pair(Suit::Spade, Rank::Jack),
                pair(Suit::Spade, Rank::Queen),
            ]
            .concat(),
            trump(),
        )
        .unwrap();
        let threes = pair(Suit::Spade, Rank::Three);
        let hand = [
            threes.as_slice(),
            &[
                card(0, Suit::Spade, Rank::Six),
                card(0, Suit::Spade, Rank::Seven),
                card(0, Suit::Spade, Rank::Nine),
            ],
        ]
        .concat();
        assert_eq!(forced_follow_cards(&hand, &lead_tractor, trump()), threes);
        let suggestions = follow_suggestions(&hand, &lead_tractor, trump(), 16);
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.iter().all(|play| {
            threes
                .iter()
                .all(|card| play.cards.iter().any(|candidate| candidate == card))
        }));
        assert!(
            suggestions
                .windows(2)
                .all(|window| window[0].cards != window[1].cards)
        );

        let lead_throw = classify_cards(
            &[
                card(0, Suit::Spade, Rank::Ace),
                pair(Suit::Spade, Rank::Jack)[0],
                pair(Suit::Spade, Rank::Jack)[1],
                pair(Suit::Spade, Rank::Queen)[0],
                pair(Suit::Spade, Rank::Queen)[1],
            ],
            trump(),
        )
        .unwrap();
        assert_eq!(forced_follow_cards(&hand, &lead_throw, trump()), hand);
    }

    #[test]
    fn forced_follow_cards_does_not_choose_between_two_valid_tractors() {
        let lead = classify_cards(
            &[
                pair(Suit::Spade, Rank::Jack),
                pair(Suit::Spade, Rank::Queen),
            ]
            .concat(),
            trump(),
        )
        .unwrap();
        let hand = [
            pair(Suit::Spade, Rank::Three).as_slice(),
            pair(Suit::Spade, Rank::Four).as_slice(),
            pair(Suit::Spade, Rank::Seven).as_slice(),
            pair(Suit::Spade, Rank::Eight).as_slice(),
        ]
        .concat();
        assert!(forced_follow_cards(&hand, &lead, trump()).is_empty());
        assert_eq!(follow_suggestions(&hand, &lead, trump(), 16).len(), 2);
    }

    #[test]
    fn a_void_player_may_mix_off_suits_and_the_discard_cannot_win() {
        let lead = classify_cards(
            &[
                card(0, Suit::Spade, Rank::Five),
                card(0, Suit::Spade, Rank::Seven),
            ],
            trump(),
        )
        .unwrap();
        let hand = [
            card(0, Suit::Diamond, Rank::Three),
            card(0, Suit::Club, Rank::Four),
            card(0, Suit::Heart, Rank::Ace),
        ];
        let discard = validate_follow(&hand, &hand[..2], &lead, trump()).unwrap();

        assert_eq!(discard.category, Category::Mixed);
        assert_eq!(
            compare_for_trick(&lead, &lead, &discard, trump()),
            Ordering::Less
        );
    }

    #[test]
    fn a_short_led_suit_may_be_completed_with_another_suit() {
        let lead = classify_cards(
            &[
                card(0, Suit::Spade, Rank::Five),
                card(0, Suit::Spade, Rank::Seven),
            ],
            trump(),
        )
        .unwrap();
        let spade = card(0, Suit::Spade, Rank::Three);
        let club = card(0, Suit::Club, Rank::Four);
        let hand = [spade, club, card(0, Suit::Heart, Rank::Ace)];
        let discard = validate_follow(&hand, &[spade, club], &lead, trump()).unwrap();

        assert_eq!(discard.category, Category::Mixed);
    }

    #[test]
    fn kill_and_overkill_compare_the_structure_required_by_the_lead() {
        let all_singles = classify_cards(
            &[
                card(0, Suit::Spade, Rank::Five),
                card(0, Suit::Spade, Rank::Six),
                card(0, Suit::Spade, Rank::Seven),
            ],
            trump(),
        )
        .unwrap();
        let first_kill = classify_cards(
            &[
                card(0, Suit::Heart, Rank::Nine),
                card(0, Suit::Heart, Rank::Seven),
                card(1, Suit::Heart, Rank::Seven),
            ],
            trump(),
        )
        .unwrap();
        let attempted_cover = classify_cards(
            &[
                card(0, Suit::Heart, Rank::Eight),
                card(1, Suit::Heart, Rank::Eight),
                card(0, Suit::Heart, Rank::Four),
            ],
            trump(),
        )
        .unwrap();
        assert_eq!(
            compare_for_trick(&all_singles, &first_kill, &attempted_cover, trump()),
            Ordering::Less
        );

        let pair_lead = classify_cards(
            &[
                card(0, Suit::Spade, Rank::Five),
                pair(Suit::Spade, Rank::Four)[0],
                pair(Suit::Spade, Rank::Four)[1],
            ],
            trump(),
        )
        .unwrap();
        assert_eq!(
            compare_for_trick(&pair_lead, &first_kill, &attempted_cover, trump()),
            Ordering::Greater
        );
    }

    #[test]
    fn trump_padding_without_the_led_structure_cannot_kill() {
        let lead_cards = [
            pair(Suit::Spade, Rank::Six).as_slice(),
            pair(Suit::Spade, Rank::Seven).as_slice(),
        ]
        .concat();
        let lead = classify_cards(&lead_cards, trump()).unwrap();
        let padding = classify_cards(
            &[
                card(0, Suit::Heart, Rank::Three),
                card(0, Suit::Heart, Rank::Five),
                card(0, Suit::Heart, Rank::Seven),
                card(0, Suit::Heart, Rank::Nine),
            ],
            trump(),
        )
        .unwrap();
        assert_eq!(
            compare_for_trick(&lead, &lead, &padding, trump()),
            Ordering::Less
        );
    }

    #[test]
    fn kitty_multiplier_uses_the_strongest_throw_component() {
        let cards = [
            pair(Suit::Spade, Rank::Ace).as_slice(),
            pair(Suit::Spade, Rank::King).as_slice(),
            &[card(0, Suit::Spade, Rank::Queen)],
        ]
        .concat();
        let play = classify_cards(&cards, trump()).unwrap();
        assert_eq!(play.kitty_multiplier(), 8);

        for (pair_count, multiplier) in [(2, 8), (3, 16), (4, 32)] {
            let ranks = [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack];
            let tractor = ranks[..pair_count]
                .iter()
                .flat_map(|rank| pair(Suit::Spade, *rank))
                .collect::<Vec<_>>();
            assert_eq!(
                classify_cards(&tractor, trump())
                    .unwrap()
                    .kitty_multiplier(),
                multiplier
            );
        }
    }
}
