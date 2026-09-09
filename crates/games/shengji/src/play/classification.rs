use super::{
    BTreeMap, Category, Component, HashSet, Ordering, PairUnit, PlayError, QuadUnit, ShengjiCard,
    ShengjiClassifiedPlay, ShengjiRank, ShengjiSuit, ShengjiTrump, StructureDemands, TripleUnit,
    category_precedence, competition_key, component_priority, pair_units, quad_units,
    spaceship_tops, special_follow_hierarchy, titanic_tops, tractor_tops, triple_units,
};

pub(super) fn classify_mixed_discard(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
) -> ShengjiClassifiedPlay {
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
    ShengjiClassifiedPlay {
        cards,
        category: Category::Mixed,
        components,
    }
}

/// 比较两手已合法跟出的牌。返回 `Greater` 表示挑战者成为当前赢家。
pub fn compare_for_trick(
    lead: &ShengjiClassifiedPlay,
    current: &ShengjiClassifiedPlay,
    challenger: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
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

fn special_competition_key(
    lead: &ShengjiClassifiedPlay,
    candidate: &ShengjiClassifiedPlay,
) -> Option<Vec<u16>> {
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

pub(crate) fn classify_cards(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
) -> Result<ShengjiClassifiedPlay, PlayError> {
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
    Ok(ShengjiClassifiedPlay {
        cards,
        category: play_category,
        components,
    })
}

pub(crate) fn category(card: ShengjiCard, trump: ShengjiTrump) -> Category {
    if trump.is_trump(card) {
        Category::Trump
    } else {
        Category::Suit(card.suit().expect("non-trump cards are suited"))
    }
}

pub(crate) fn strength(card: ShengjiCard, trump: ShengjiTrump) -> u8 {
    trump.strength(card)
}

fn decompose(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    play_category: Category,
) -> Vec<Component> {
    let mut faces: BTreeMap<(Option<ShengjiSuit>, ShengjiRank), Vec<ShengjiCard>> = BTreeMap::new();
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

pub(super) fn component_can_be_beaten(
    component: &Component,
    play_category: Category,
    hand: &[ShengjiCard],
    trump: ShengjiTrump,
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
