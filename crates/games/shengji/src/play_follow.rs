use super::*;

pub fn classify_lead(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    rules: &ShengjiRuleSet,
    opponent_hands: &[&[ShengjiCard]],
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
    hand: &[ShengjiCard],
    cards: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
) -> Result<ShengjiClassifiedPlay, FollowError> {
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

fn triple_follow_rank(cards: &[ShengjiCard]) -> u8 {
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
    cards: &[ShengjiCard],
    triple_count: usize,
    trump: ShengjiTrump,
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

fn face_counts(cards: &[ShengjiCard]) -> BTreeMap<(Option<ShengjiSuit>, ShengjiRank), usize> {
    let mut counts = BTreeMap::new();
    for card in cards {
        *counts.entry((card.suit(), card.rank())).or_default() += 1;
    }
    counts
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FollowPattern {
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

pub(super) fn special_follow_hierarchy(
    lead: &ShengjiClassifiedPlay,
) -> Option<Vec<Vec<FollowPattern>>> {
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

pub(super) fn eight_card_tractor_hierarchy() -> Vec<Vec<FollowPattern>> {
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
    cards: Vec<ShengjiCard>,
    strength: u8,
}

fn face_stocks(cards: &[ShengjiCard], trump: ShengjiTrump) -> Vec<FaceStock> {
    let mut faces: BTreeMap<(Option<ShengjiSuit>, ShengjiRank), Vec<ShengjiCard>> = BTreeMap::new();
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
            stock.cards[0].suit().map_or(4, ShengjiSuit::bid_strength),
            stock.cards[0].rank(),
        )
    });
    stocks
}

fn pattern_assignments(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
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

pub(super) fn best_follow_tier(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
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
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    hierarchy: &[Vec<FollowPattern>],
    limit: usize,
) -> Vec<Vec<ShengjiCard>> {
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

fn cards_for_assignment(
    stocks: &[FaceStock],
    assignment: &[usize],
    trump: ShengjiTrump,
) -> Vec<ShengjiCard> {
    let mut selected = Vec::new();
    for (stock, count) in stocks.iter().zip(assignment) {
        selected.extend(stock.cards.iter().take(*count).copied());
    }
    selected.sort_by_key(|card| (strength(*card, trump), card.deck()));
    selected
}

fn forced_cards_for_hierarchy(
    hand: &[ShengjiCard],
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    hierarchy: &[Vec<FollowPattern>],
) -> Vec<ShengjiCard> {
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
            (if minimum == stock.cards.len() {
                stock.cards.as_slice()
            } else {
                &[]
            })
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
pub fn forced_follow_cards(
    hand: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
) -> Vec<ShengjiCard> {
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
    for demand in demands.tractors() {
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

    let mut forced = None::<HashSet<ShengjiCard>>;
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
    hand: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
    limit: usize,
) -> Vec<ShengjiClassifiedPlay> {
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
    let card_key = |card: &ShengjiCard| {
        (
            trump.is_trump(*card),
            strength(*card, trump),
            card.suit().map_or(4, ShengjiSuit::bid_strength),
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
    choices: &[ShengjiCard],
    start: usize,
    remaining: usize,
    current: &mut Vec<ShengjiCard>,
    visited: &mut usize,
    budget: usize,
    accept: &mut impl FnMut(&[ShengjiCard]) -> bool,
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
