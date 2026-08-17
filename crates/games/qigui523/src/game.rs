use std::collections::{HashSet, VecDeque};
use std::fmt;

use crate::{Card, ClassifiedPlay, PlayError, RuleError, RuleSet, build_deck, can_beat, classify};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlayerId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    id: PlayerId,
    hand: Vec<Card>,
    score: u32,
}

impl PlayerState {
    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[Card] {
        &self.hand
    }

    pub fn score(&self) -> u32 {
        self.score
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartingCard {
    pub player: PlayerId,
    pub card: Card,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayRecord {
    Played {
        player: PlayerId,
        play: ClassifiedPlay,
    },
    Passed {
        player: PlayerId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrickState {
    leader: PlayerId,
    current_player: PlayerId,
    winning_player: Option<PlayerId>,
    winning_play: Option<ClassifiedPlay>,
    records: Vec<PlayRecord>,
    table_points: u32,
    passes_after_winning_play: usize,
}

impl TrickState {
    fn new(leader: PlayerId) -> Self {
        Self {
            leader,
            current_player: leader,
            winning_player: None,
            winning_play: None,
            records: Vec::new(),
            table_points: 0,
            passes_after_winning_play: 0,
        }
    }

    pub fn leader(&self) -> PlayerId {
        self.leader
    }

    pub fn current_player(&self) -> PlayerId {
        self.current_player
    }

    pub fn winning_player(&self) -> Option<PlayerId> {
        self.winning_player
    }

    pub fn winning_play(&self) -> Option<&ClassifiedPlay> {
        self.winning_play.as_ref()
    }

    pub fn records(&self) -> &[PlayRecord] {
        &self.records
    }

    pub fn table_points(&self) -> u32 {
        self.table_points
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameResult {
    /// 摸牌堆耗尽后，第一个出完手牌的玩家。
    pub finisher: PlayerId,
    pub scores: Vec<u32>,
    pub captured_hand_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Playing,
    Finished(GameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    Played {
        player: PlayerId,
        next_player: PlayerId,
    },
    Passed {
        player: PlayerId,
        next_player: PlayerId,
    },
    TrickCompleted {
        winner: PlayerId,
        points: u32,
        next_player: PlayerId,
        cards_drawn: usize,
    },
    GameFinished(GameResult),
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
    NotPlayersTurn {
        expected: PlayerId,
        actual: PlayerId,
    },
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand(Card),
    InvalidPlay(PlayError),
    PlayDoesNotBeatCurrent,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或不属于当前副数的牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}；当前应由 {:?} 操作", actual, expected)
            }
            Self::GameAlreadyFinished => f.write_str("本局已经结束"),
            Self::MustLeadWithCards => f.write_str("本轮首家不能跳过，必须出牌"),
            Self::CardNotInHand(card) => write!(f, "玩家手中没有这张牌：{card}"),
            Self::InvalidPlay(error) => error.fmt(f),
            Self::PlayDoesNotBeatCurrent => f.write_str("所出牌型不能压过当前最大牌型"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<PlayError> for GameError {
    fn from(value: PlayError) -> Self {
        Self::InvalidPlay(value)
    }
}

/// 权威游戏状态。所有修改都只能通过 [`GameState::play_cards`] 或
/// [`GameState::pass`] 完成。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: RuleSet,
    players: Vec<PlayerState>,
    draw_pile: VecDeque<Card>,
    starting_card: StartingCard,
    trick: Option<TrickState>,
    phase: Phase,
}

impl GameState {
    /// `deck` 的第 0 张是最先发出的牌。洗牌应由房主在调用前完成。
    pub fn new_with_deck(rules: RuleSet, deck: Vec<Card>) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        validate_deck(rules.deck_count, &deck)?;

        Ok(Self::deal_validated_deck(rules, deck))
    }

    /// 开发构建专用：只接受刚好够发初始手牌的合法牌堆。
    ///
    /// 该入口由 Cargo feature 在编译期移除，正常发行版本不能创建短牌堆。
    #[cfg(feature = "developer")]
    pub fn new_with_development_deck(rules: RuleSet, deck: Vec<Card>) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let expected = usize::from(rules.player_count) * usize::from(rules.hand_size);
        if deck.len() != expected {
            return Err(GameError::InvalidDeckSize {
                expected,
                actual: deck.len(),
            });
        }
        let full_deck: HashSet<_> = build_deck(rules.deck_count).into_iter().collect();
        let actual: HashSet<_> = deck.iter().copied().collect();
        if actual.len() != deck.len() || !actual.is_subset(&full_deck) {
            return Err(GameError::InvalidDeckContents);
        }

        Ok(Self::deal_validated_deck(rules, deck))
    }

    fn deal_validated_deck(rules: RuleSet, deck: Vec<Card>) -> Self {
        let mut draw_pile = VecDeque::from(deck);
        let mut players: Vec<_> = (0..usize::from(rules.player_count))
            .map(|index| PlayerState {
                id: PlayerId(index),
                hand: Vec::with_capacity(usize::from(rules.hand_size)),
                score: 0,
            })
            .collect();
        let mut deal_order =
            Vec::with_capacity(usize::from(rules.player_count) * usize::from(rules.hand_size));

        // 逐张、顺时针发牌，而不是一次给一个玩家发满。
        for _ in 0..rules.hand_size {
            for player in &mut players {
                let card = draw_pile
                    .pop_front()
                    .expect("RuleSet::validate ensured enough cards");
                player.hand.push(card);
                deal_order.push((player.id, card));
            }
        }

        // 多副牌的最小牌可能重复。洗牌后的牌序本身就是随机源，最先发出的那张
        // 最小牌决定先手，避免把另一个随机数源耦合进规则核心。
        let starting_card = find_starting_card(&deal_order);
        sort_hands(&mut players);
        let trick = Some(TrickState::new(starting_card.player));

        Self {
            rules,
            players,
            draw_pile,
            starting_card,
            trick,
            phase: Phase::Playing,
        }
    }

    pub fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub fn player(&self, id: PlayerId) -> Option<&PlayerState> {
        self.players.get(id.0)
    }

    pub fn draw_pile_len(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn starting_card(&self) -> StartingCard {
        self.starting_card
    }

    pub fn trick(&self) -> Option<&TrickState> {
        self.trick.as_ref()
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// 开发构建专用：完整替换某名玩家的手牌，不检查牌副范围或实体牌重复。
    #[cfg(feature = "developer")]
    pub fn replace_player_hand(
        &mut self,
        player: PlayerId,
        cards: Vec<Card>,
    ) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            return Err(GameError::GameAlreadyFinished);
        }
        if player.0 >= self.players.len() {
            return Err(GameError::InvalidPlayer(player));
        }
        if cards.is_empty() {
            return Err(GameError::InvalidDeckContents);
        }
        self.players[player.0].hand = cards;
        self.players[player.0].hand.sort_by(Card::display_cmp);
        Ok(())
    }

    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[Card],
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        let play = classify(cards, &self.rules)?;
        self.ensure_cards_in_hand(player, cards)?;

        if let Some(current) = self
            .trick
            .as_ref()
            .and_then(|trick| trick.winning_play.as_ref())
            && !can_beat(&play, current, &self.rules)
        {
            return Err(GameError::PlayDoesNotBeatCurrent);
        }

        remove_cards(&mut self.players[player.0].hand, cards);

        let trick = self.trick.as_mut().expect("playing games have a trick");
        trick.table_points += u32::from(play.score());
        trick.winning_player = Some(player);
        trick.winning_play = Some(play.clone());
        trick.passes_after_winning_play = 0;
        trick.records.push(PlayRecord::Played { player, play });

        // 只有摸牌堆已经为空，出完手牌才会立即结束本局。
        if self.draw_pile.is_empty() && self.players[player.0].hand.is_empty() {
            let result = self.finish_game(player, true);
            return Ok(ActionOutcome::GameFinished(result));
        }

        let next_player = self.next_player(player);
        self.trick
            .as_mut()
            .expect("playing games have a trick")
            .current_player = next_player;
        Ok(ActionOutcome::Played {
            player,
            next_player,
        })
    }

    pub fn pass(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        let trick = self.trick.as_mut().expect("playing games have a trick");
        if trick.winning_play.is_none() {
            return Err(GameError::MustLeadWithCards);
        }

        trick.records.push(PlayRecord::Passed { player });
        trick.passes_after_winning_play += 1;

        if trick.passes_after_winning_play == self.players.len() - 1 {
            return Ok(self.complete_trick());
        }

        let next_player = self.next_player(player);
        self.trick
            .as_mut()
            .expect("playing games have a trick")
            .current_player = next_player;
        Ok(ActionOutcome::Passed {
            player,
            next_player,
        })
    }

    fn ensure_turn(&self, player: PlayerId) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            return Err(GameError::GameAlreadyFinished);
        }
        if player.0 >= self.players.len() {
            return Err(GameError::InvalidPlayer(player));
        }
        let expected = self
            .trick
            .as_ref()
            .expect("playing games have a trick")
            .current_player;
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        Ok(())
    }

    fn ensure_cards_in_hand(&self, player: PlayerId, cards: &[Card]) -> Result<(), GameError> {
        for card in cards {
            if !self.players[player.0].hand.contains(card) {
                return Err(GameError::CardNotInHand(*card));
            }
        }
        Ok(())
    }

    fn next_player(&self, player: PlayerId) -> PlayerId {
        PlayerId((player.0 + self.players.len() - 1) % self.players.len())
    }

    fn complete_trick(&mut self) -> ActionOutcome {
        let old_trick = self.trick.take().expect("playing games have a trick");
        let winner = old_trick
            .winning_player
            .expect("a trick cannot complete without a play");
        let points = old_trick.table_points;
        self.players[winner.0].score += points;

        let cards_drawn = self.refill_hands_from(winner);
        if self.draw_pile.is_empty()
            && let Some(finisher) = self.first_empty_player_from(winner)
        {
            let result = self.finish_game(finisher, false);
            return ActionOutcome::GameFinished(result);
        }

        self.trick = Some(TrickState::new(winner));
        ActionOutcome::TrickCompleted {
            winner,
            points,
            next_player: winner,
            cards_drawn,
        }
    }

    /// 从上轮赢家开始，顺时针每人一次摸一张，循环至补满或牌堆为空。
    fn refill_hands_from(&mut self, winner: PlayerId) -> usize {
        let mut cards_drawn = 0;
        loop {
            let mut drew_in_cycle = false;
            for offset in 0..self.players.len() {
                let player_index = (winner.0 + offset) % self.players.len();
                if self.players[player_index].hand.len() < usize::from(self.rules.hand_size)
                    && let Some(card) = self.draw_pile.pop_front()
                {
                    self.players[player_index].hand.push(card);
                    cards_drawn += 1;
                    drew_in_cycle = true;
                }
            }
            if !drew_in_cycle
                || self.draw_pile.is_empty()
                || self
                    .players
                    .iter()
                    .all(|player| player.hand.len() >= usize::from(self.rules.hand_size))
            {
                break;
            }
        }
        sort_hands(&mut self.players);
        cards_drawn
    }

    fn first_empty_player_from(&self, from: PlayerId) -> Option<PlayerId> {
        (0..self.players.len())
            .map(|offset| PlayerId((from.0 + offset) % self.players.len()))
            .find(|player| self.players[player.0].hand.is_empty())
    }

    fn finish_game(&mut self, finisher: PlayerId, collect_table_points: bool) -> GameResult {
        if collect_table_points {
            let table_points = self.trick.as_ref().map_or(0, |trick| trick.table_points);
            self.players[finisher.0].score += table_points;
        }

        let captured_hand_points = self
            .players
            .iter()
            .flat_map(|player| player.hand.iter())
            .map(|card| u32::from(card.score()))
            .sum();
        self.players[finisher.0].score += captured_hand_points;
        self.trick = None;

        let result = GameResult {
            finisher,
            scores: self.players.iter().map(|player| player.score).collect(),
            captured_hand_points,
        };
        self.phase = Phase::Finished(result.clone());
        result
    }
}

fn validate_deck(deck_count: u8, deck: &[Card]) -> Result<(), GameError> {
    let expected_deck = build_deck(deck_count);
    if deck.len() != expected_deck.len() {
        return Err(GameError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = expected_deck.into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}

fn find_starting_card(deal_order: &[(PlayerId, Card)]) -> StartingCard {
    let minimum_strength = deal_order
        .iter()
        .map(|(_, card)| card.semantic_strength())
        .min()
        .expect("every player receives at least one card");
    deal_order
        .iter()
        .copied()
        .find(|(_, card)| card.semantic_strength() == minimum_strength)
        .map(|(player, card)| StartingCard { player, card })
        .expect("minimum strength came from the deal order")
}

fn remove_cards(hand: &mut Vec<Card>, cards: &[Card]) {
    for card in cards {
        let index = hand
            .iter()
            .position(|candidate| candidate == card)
            .expect("cards were checked before removal");
        hand.remove(index);
    }
}

fn sort_hands(players: &mut [PlayerState]) {
    for player in players {
        player.hand.sort_by(Card::display_cmp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rank, Suit};

    fn deck_with_prefix(prefix: &[Card], deck_count: u8) -> Vec<Card> {
        let prefix_set: HashSet<_> = prefix.iter().copied().collect();
        let mut deck = prefix.to_vec();
        deck.extend(
            build_deck(deck_count)
                .into_iter()
                .filter(|card| !prefix_set.contains(card)),
        );
        deck
    }

    #[test]
    fn deals_clockwise_and_exposes_lowest_card_owner() {
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let diamond_four = Card::suited(0, Suit::Diamond, Rank::Four);
        let deck = deck_with_prefix(
            &[
                Card::suited(0, Suit::Spade, Rank::Ace),
                diamond_four,
                Card::suited(0, Suit::Heart, Rank::Six),
            ],
            1,
        );
        let game = GameState::new_with_deck(rules, deck).unwrap();

        assert_eq!(game.starting_card().player, PlayerId(1));
        assert_eq!(game.starting_card().card, diamond_four);
        assert_eq!(game.trick().unwrap().current_player(), PlayerId(1));
        assert!(game.players().iter().all(|player| player.hand().len() == 5));
        assert_eq!(game.draw_pile_len(), 39);
    }

    #[test]
    fn duplicate_lowest_cards_use_the_shuffled_deal_order_as_tie_breaker() {
        let rules = RuleSet {
            deck_count: 2,
            player_count: 3,
            ..RuleSet::default()
        };
        let first_diamond_four = Card::suited(1, Suit::Diamond, Rank::Four);
        let later_diamond_four = Card::suited(0, Suit::Diamond, Rank::Four);
        let deck = deck_with_prefix(
            &[
                Card::suited(0, Suit::Heart, Rank::Six),
                Card::suited(0, Suit::Spade, Rank::Six),
                first_diamond_four,
                later_diamond_four,
            ],
            2,
        );
        let game = GameState::new_with_deck(rules, deck).unwrap();

        // 玩家 2 先从洗好的牌序中拿到最小牌，所以不按较小座位号选择玩家 0。
        assert_eq!(game.starting_card().player, PlayerId(2));
        assert_eq!(game.starting_card().card, first_diamond_four);
    }

    #[test]
    fn winner_collects_points_then_everyone_refills() {
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let diamond_four = Card::suited(0, Suit::Diamond, Rank::Four);
        let diamond_five = Card::suited(0, Suit::Diamond, Rank::Five);
        let club_five = Card::suited(0, Suit::Club, Rank::Five);
        let deck = deck_with_prefix(
            &[
                diamond_four,
                club_five,
                Card::suited(0, Suit::Heart, Rank::Six),
                diamond_five,
            ],
            1,
        );
        let mut game = GameState::new_with_deck(rules, deck).unwrap();

        game.play_cards(PlayerId(0), &[diamond_five]).unwrap();
        game.pass(PlayerId(2)).unwrap();
        game.play_cards(PlayerId(1), &[club_five]).unwrap();
        game.pass(PlayerId(0)).unwrap();
        let outcome = game.pass(PlayerId(2)).unwrap();

        assert_eq!(
            outcome,
            ActionOutcome::TrickCompleted {
                winner: PlayerId(1),
                points: 10,
                next_player: PlayerId(1),
                cards_drawn: 2,
            }
        );
        assert_eq!(game.player(PlayerId(1)).unwrap().score(), 10);
        assert!(game.players().iter().all(|player| player.hand().len() == 5));
    }

    #[test]
    fn bomb_can_change_the_required_card_count() {
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let pair = [
            Card::suited(0, Suit::Diamond, Rank::Four),
            Card::suited(0, Suit::Club, Rank::Four),
        ];
        let bomb = [
            Card::suited(0, Suit::Diamond, Rank::Six),
            Card::suited(0, Suit::Club, Rank::Six),
            Card::suited(0, Suit::Heart, Rank::Six),
            Card::suited(0, Suit::Spade, Rank::Six),
        ];
        let mut prefix = vec![pair[0], bomb[0], Card::suited(0, Suit::Heart, Rank::Eight)];
        prefix.extend([pair[1], bomb[1], Card::suited(0, Suit::Club, Rank::Eight)]);
        prefix.extend([
            Card::suited(0, Suit::Diamond, Rank::Nine),
            bomb[2],
            Card::suited(0, Suit::Heart, Rank::Nine),
        ]);
        prefix.extend([
            Card::suited(0, Suit::Diamond, Rank::Ten),
            bomb[3],
            Card::suited(0, Suit::Heart, Rank::Ten),
        ]);
        let deck = deck_with_prefix(&prefix, 1);
        let mut game = GameState::new_with_deck(rules, deck).unwrap();

        game.play_cards(PlayerId(0), &pair).unwrap();
        game.pass(PlayerId(2)).unwrap();
        assert!(game.play_cards(PlayerId(1), &bomb).is_ok());
    }

    #[test]
    fn emptying_a_hand_after_draw_pile_is_empty_collects_remaining_points() {
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let deck = build_deck(1);
        let mut game = GameState::new_with_deck(rules, deck).unwrap();
        let finisher = game.starting_card.player;
        let last_card = game.players[finisher.0].hand[0];

        // 构造规则边界状态：牌堆已耗尽，当前玩家只剩一张牌并领出。
        game.draw_pile.clear();
        game.players[finisher.0].hand = vec![last_card];
        game.trick = Some(TrickState::new(finisher));
        let hand_points_before: u32 = game
            .players
            .iter()
            .flat_map(|player| player.hand.iter())
            .map(|card| u32::from(card.score()))
            .sum();

        let outcome = game.play_cards(finisher, &[last_card]).unwrap();
        let ActionOutcome::GameFinished(result) = outcome else {
            panic!("game should finish");
        };
        assert_eq!(result.finisher, finisher);
        assert_eq!(
            result.captured_hand_points,
            hand_points_before - u32::from(last_card.score())
        );
    }

    #[test]
    fn a_complete_deterministic_game_conserves_all_points() {
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let mut game = GameState::new_with_deck(rules, build_deck(1)).unwrap();

        for _ in 0..1_000 {
            if let Phase::Finished(result) = game.phase() {
                assert_eq!(result.scores.iter().sum::<u32>(), 100);
                return;
            }

            let trick = game.trick().unwrap();
            let current_player = trick.current_player();
            let current_play = trick.winning_play().cloned();
            let hand = game.player(current_player).unwrap().hand().to_vec();
            let playable = hand.into_iter().find(|card| {
                let candidate = classify(&[*card], game.rules()).unwrap();
                current_play
                    .as_ref()
                    .is_none_or(|current| can_beat(&candidate, current, game.rules()))
            });

            if let Some(card) = playable {
                game.play_cards(current_player, &[card]).unwrap();
            } else {
                game.pass(current_player).unwrap();
            }
        }

        panic!("deterministic game did not terminate");
    }
}
