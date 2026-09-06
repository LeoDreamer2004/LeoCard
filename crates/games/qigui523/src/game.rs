#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;

use crate::{
    ClassifiedPlay, PlayError, QiGuiCard, QiGuiRuleSet, RuleError, build_deck, can_beat, classify,
};
use std::collections::{HashSet, VecDeque};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct QiGuiPlayerId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    id: QiGuiPlayerId,
    hand: Vec<QiGuiCard>,
    score: u32,
}

impl PlayerState {
    pub fn id(&self) -> QiGuiPlayerId {
        self.id
    }

    pub fn hand(&self) -> &[QiGuiCard] {
        &self.hand
    }

    pub fn score(&self) -> u32 {
        self.score
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartingCard {
    pub player: QiGuiPlayerId,
    pub card: QiGuiCard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayRecord {
    Played {
        player: QiGuiPlayerId,
        play: ClassifiedPlay,
    },
    Passed {
        player: QiGuiPlayerId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrickState {
    leader: QiGuiPlayerId,
    current_player: QiGuiPlayerId,
    winning_player: Option<QiGuiPlayerId>,
    winning_play: Option<ClassifiedPlay>,
    records: Vec<PlayRecord>,
    table_points: u32,
    passes_after_winning_play: usize,
}

impl TrickState {
    fn new(leader: QiGuiPlayerId) -> Self {
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

    pub fn leader(&self) -> QiGuiPlayerId {
        self.leader
    }

    pub fn current_player(&self) -> QiGuiPlayerId {
        self.current_player
    }

    pub fn winning_player(&self) -> Option<QiGuiPlayerId> {
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
    pub finisher: QiGuiPlayerId,
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
        player: QiGuiPlayerId,
        next_player: QiGuiPlayerId,
    },
    Passed {
        player: QiGuiPlayerId,
        next_player: QiGuiPlayerId,
    },
    TrickCompleted {
        winner: QiGuiPlayerId,
        points: u32,
        next_player: QiGuiPlayerId,
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
    InvalidPlayer(QiGuiPlayerId),
    NotPlayersTurn {
        expected: QiGuiPlayerId,
        actual: QiGuiPlayerId,
    },
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand(QiGuiCard),
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
    rules: QiGuiRuleSet,
    players: Vec<PlayerState>,
    draw_pile: VecDeque<QiGuiCard>,
    starting_card: StartingCard,
    trick: Option<TrickState>,
    phase: Phase,
}

impl GameState {
    /// `deck` 的第 0 张是最先发出的牌。洗牌应由房主在调用前完成。
    pub fn new_with_deck(rules: QiGuiRuleSet, deck: Vec<QiGuiCard>) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        validate_deck(rules.deck_count, &deck)?;

        Ok(Self::deal_validated_deck(rules, deck))
    }

    /// 开发构建专用：只接受刚好够发初始手牌的合法牌堆。
    ///
    /// 该入口由 Cargo feature 在编译期移除，正常发行版本不能创建短牌堆。
    #[cfg(feature = "developer")]
    pub fn new_with_development_deck(
        rules: QiGuiRuleSet,
        deck: Vec<QiGuiCard>,
    ) -> Result<Self, GameError> {
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

    fn deal_validated_deck(rules: QiGuiRuleSet, deck: Vec<QiGuiCard>) -> Self {
        let mut draw_pile = VecDeque::from(deck);
        let mut players: Vec<_> = (0..usize::from(rules.player_count))
            .map(|index| PlayerState {
                id: QiGuiPlayerId(index),
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
                    .expect("QiGuiRuleSet::validate ensured enough cards");
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

    pub fn rules(&self) -> &QiGuiRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub fn player(&self, id: QiGuiPlayerId) -> Option<&PlayerState> {
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
        player: QiGuiPlayerId,
        cards: Vec<QiGuiCard>,
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
        self.players[player.0].hand.sort_by(QiGuiCard::display_cmp);
        Ok(())
    }

    pub fn play_cards(
        &mut self,
        player: QiGuiPlayerId,
        cards: &[QiGuiCard],
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

    pub fn pass(&mut self, player: QiGuiPlayerId) -> Result<ActionOutcome, GameError> {
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

    fn ensure_turn(&self, player: QiGuiPlayerId) -> Result<(), GameError> {
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

    fn ensure_cards_in_hand(
        &self,
        player: QiGuiPlayerId,
        cards: &[QiGuiCard],
    ) -> Result<(), GameError> {
        for card in cards {
            if !self.players[player.0].hand.contains(card) {
                return Err(GameError::CardNotInHand(*card));
            }
        }
        Ok(())
    }

    fn next_player(&self, player: QiGuiPlayerId) -> QiGuiPlayerId {
        QiGuiPlayerId((player.0 + self.players.len() - 1) % self.players.len())
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
    fn refill_hands_from(&mut self, winner: QiGuiPlayerId) -> usize {
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

    fn first_empty_player_from(&self, from: QiGuiPlayerId) -> Option<QiGuiPlayerId> {
        (0..self.players.len())
            .map(|offset| QiGuiPlayerId((from.0 + offset) % self.players.len()))
            .find(|player| self.players[player.0].hand.is_empty())
    }

    fn finish_game(&mut self, finisher: QiGuiPlayerId, collect_table_points: bool) -> GameResult {
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

fn validate_deck(deck_count: u8, deck: &[QiGuiCard]) -> Result<(), GameError> {
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

fn find_starting_card(deal_order: &[(QiGuiPlayerId, QiGuiCard)]) -> StartingCard {
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

fn remove_cards(hand: &mut Vec<QiGuiCard>, cards: &[QiGuiCard]) {
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
        player.hand.sort_by(QiGuiCard::display_cmp);
    }
}
