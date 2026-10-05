use std::{collections::HashSet, hash::Hash};

/// 依据物理牌身份判断是否重复；不同副本的同牌面牌可以同时出现。
pub fn has_unique_cards<Card: Eq + Hash>(cards: &[Card]) -> bool {
    let mut seen = HashSet::with_capacity(cards.len());
    cards.iter().all(|card| seen.insert(card))
}

/// 选择的物理牌必须互不重复且全在手牌中。空选择合法，由具体游戏决定是否允许。
pub fn contains_unique_cards<Card: Eq + Hash>(hand: &[Card], selected: &[Card]) -> bool {
    has_unique_cards(selected) && selected.iter().all(|card| hand.contains(card))
}
