use std::collections::{HashSet, VecDeque};
use std::fmt;

#[cfg(test)]
use crate::build_deck;
use crate::play::compare_for_trick;
use crate::{
    BidError, BidState, Card, ClassifiedPlay, FollowError, PlayerId, Rank, RuleError, RuleSet,
    TeamId, TrickPlay, Trump, build_deck_for, classify_lead, level_after, validate_follow,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamProgress {
    pub levels: [Rank; 2],
}

impl Default for TeamProgress {
    fn default() -> Self {
        Self {
            levels: [Rank::Two, Rank::Two],
        }
    }
}

impl TeamProgress {
    pub const fn for_rules(rules: &RuleSet) -> Self {
        let starting_level = if rules.constant_trump {
            Rank::Three
        } else {
            Rank::Two
        };
        Self {
            levels: [starting_level, starting_level],
        }
    }

    pub fn level(&self, team: TeamId) -> Rank {
        self.levels[usize::from(team.0)]
    }

    fn promote(&mut self, team: TeamId, steps: u8, mandatory: bool) {
        let slot = &mut self.levels[usize::from(team.0)];
        *slot = level_after(*slot, steps, mandatory);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    pub id: PlayerId,
    pub hand: Vec<Card>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrickRecord {
    pub leader: PlayerId,
    pub plays: Vec<(PlayerId, ClassifiedPlay)>,
    pub winner: PlayerId,
    pub points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandResult {
    pub dealer: PlayerId,
    pub dealer_team: TeamId,
    pub collecting_team: TeamId,
    pub trick_points: u32,
    pub penalty_adjustment: i32,
    pub kitty_points: u16,
    pub kitty_multiplier: u32,
    pub collecting_score: u32,
    pub promoted_team: TeamId,
    pub promoted_steps: u8,
    pub next_dealer: PlayerId,
    pub levels: [Rank; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Dealing,
    BiddingGrace,
    BottomFlipping,
    Burying,
    BottomCopying,
    BottomCopyBurying,
    FiveTrumpCrossing,
    Playing,
    Finished(HandResult),
    RedealRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FiveTrumpCrossingStage {
    Deciding,
    Returning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottomFlipMatch {
    pub player: PlayerId,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottomFlipReveal {
    pub card: Card,
    pub matches: Vec<BottomFlipMatch>,
    pub dealer: Option<PlayerId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottomCopyState {
    next_player: PlayerId,
    remaining_without_copy: u8,
    current: Option<PlayerId>,
    bottom_holder: Option<PlayerId>,
    last_burier: PlayerId,
}

impl BottomCopyState {
    pub const fn current(&self) -> Option<PlayerId> {
        self.current
    }

    pub const fn bottom_holder(&self) -> Option<PlayerId> {
        self.bottom_holder
    }
}

/// 五主过江只在埋底后的固定窗口执行一次。先同时收集所有过江决定并统一
/// 交牌，再同时收集所有回牌并统一归还，避免行动顺序改变资格或可选手牌。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FiveTrumpCrossingState {
    stage: FiveTrumpCrossingStage,
    eligible: [bool; RuleSet::PLAYER_COUNT],
    declined: [bool; RuleSet::PLAYER_COUNT],
    outgoing: [Option<Vec<Card>>; RuleSet::PLAYER_COUNT],
    returned: [Option<Vec<Card>>; RuleSet::PLAYER_COUNT],
}

impl FiveTrumpCrossingState {
    pub const fn stage(&self) -> FiveTrumpCrossingStage {
        self.stage
    }

    pub fn eligible(&self, player: PlayerId) -> bool {
        self.eligible
            .get(usize::from(player.0))
            .copied()
            .unwrap_or(false)
    }

    pub fn decision_made(&self, player: PlayerId) -> bool {
        let index = usize::from(player.0);
        self.eligible.get(index).copied().unwrap_or(false)
            && (self.declined.get(index).copied().unwrap_or(false)
                || self.outgoing.get(index).is_some_and(Option::is_some))
    }

    pub fn crossing(&self, player: PlayerId) -> bool {
        self.outgoing
            .get(usize::from(player.0))
            .is_some_and(Option::is_some)
    }

    /// 该玩家是否需要给发起过江的对家回五张牌。
    pub fn return_required(&self, player: PlayerId) -> bool {
        self.crossing(partner(player))
    }

    pub fn return_made(&self, player: PlayerId) -> bool {
        self.returned
            .get(usize::from(player.0))
            .is_some_and(Option::is_some)
    }

    pub fn pending_players(&self) -> Vec<PlayerId> {
        (0..RuleSet::PLAYER_COUNT as u8)
            .map(PlayerId)
            .filter(|player| match self.stage {
                FiveTrumpCrossingStage::Deciding => {
                    self.eligible(*player) && !self.decision_made(*player)
                }
                FiveTrumpCrossingStage::Returning => {
                    self.return_required(*player) && !self.return_made(*player)
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct CurrentTrick {
    leader: PlayerId,
    lead: ClassifiedPlay,
    plays: Vec<(PlayerId, ClassifiedPlay)>,
    winner_index: usize,
    points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    CardDealt {
        player: PlayerId,
        card: Card,
    },
    DealComplete,
    DeclarationChanged,
    DealerTookKitty {
        dealer: PlayerId,
    },
    PowerOutageDealerChanged {
        dealer: PlayerId,
    },
    BottomFlipStarted,
    BottomCardRevealed(BottomFlipReveal),
    BottomCopyDecision {
        player: PlayerId,
        copied: bool,
        inquiry_complete: bool,
    },
    BottomCopyBuryComplete {
        player: PlayerId,
        inquiry_complete: bool,
    },
    BuryComplete {
        leader: PlayerId,
    },
    FiveTrumpCrossingDecision {
        player: PlayerId,
        crossing: bool,
        decisions_complete: bool,
    },
    FiveTrumpCrossingReturn {
        player: PlayerId,
        crossing_complete: bool,
    },
    Played {
        player: PlayerId,
        next: PlayerId,
    },
    ThrowFailed {
        player: PlayerId,
        attempted: Vec<Card>,
        forced: ClassifiedPlay,
        penalty_points: u16,
        next: PlayerId,
    },
    TrickComplete(TrickRecord),
    HandComplete(HandResult),
}

#[derive(Clone, Debug)]
pub struct GameState {
    rules: RuleSet,
    teams: TeamProgress,
    fixed_dealer: Option<PlayerId>,
    first_recipient: PlayerId,
    bidding_dealer: PlayerId,
    power_outage_used: bool,
    bottom_flip_index: usize,
    bottom_flip_winner: Option<(PlayerId, Trump)>,
    dealer_from_bottom_flip: bool,
    deck: VecDeque<Card>,
    players: Vec<PlayerState>,
    bidding: BidState,
    trump: Option<Trump>,
    dealer: Option<PlayerId>,
    kitty: Vec<Card>,
    buried: Vec<Card>,
    /// 当前这批底牌的实际埋底者。普通流程为庄家；每次成功
    /// 抄底取走旧底后暂时为空，待抄底者重埋后转移给该玩家。
    bottom_burier: Option<PlayerId>,
    bottom_copy: Option<BottomCopyState>,
    five_trump_crossing: Option<FiveTrumpCrossingState>,
    phase: Phase,
    current_player: Option<PlayerId>,
    trick: Option<CurrentTrick>,
    history: Vec<TrickRecord>,
    collecting_score: u32,
    trick_points: u32,
    penalty_adjustment: i32,
}

impl GameState {
    pub fn new(
        rules: RuleSet,
        teams: TeamProgress,
        fixed_dealer: Option<PlayerId>,
        first_recipient: PlayerId,
        shuffled_deck: Vec<Card>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        if first_recipient.0 >= RuleSet::PLAYER_COUNT as u8 {
            return Err(GameError::InvalidPlayer(first_recipient));
        }
        if fixed_dealer.is_some_and(|dealer| dealer.0 >= RuleSet::PLAYER_COUNT as u8) {
            return Err(GameError::InvalidPlayer(fixed_dealer.unwrap()));
        }
        validate_deck(&shuffled_deck, rules)?;
        let first_hand = fixed_dealer.is_none();
        let teams = if first_hand {
            TeamProgress::for_rules(&rules)
        } else {
            teams
        };
        let provisional_dealer = fixed_dealer.unwrap_or(first_recipient);
        let level = teams.level(provisional_dealer.team());
        let hand_size = rules.hand_size();
        let kitty_size = rules.kitty_size();
        Ok(Self {
            rules,
            teams,
            fixed_dealer,
            first_recipient,
            bidding_dealer: provisional_dealer,
            power_outage_used: false,
            bottom_flip_index: 0,
            bottom_flip_winner: None,
            dealer_from_bottom_flip: false,
            deck: shuffled_deck.into(),
            players: (0..RuleSet::PLAYER_COUNT)
                .map(|id| PlayerState {
                    id: PlayerId(id as u8),
                    hand: Vec::with_capacity(hand_size + kitty_size),
                })
                .collect(),
            bidding: BidState::new_with_rules(level, rules.deck_count, rules.bid_with_joker),
            trump: None,
            // 后续小局的庄家已由上一局结算确定，发牌开始就应公开；首局则
            // 等第一位玩家亮主后再随当前最高声明实时转移。
            dealer: fixed_dealer,
            kitty: Vec::new(),
            buried: Vec::new(),
            bottom_burier: None,
            bottom_copy: None,
            five_trump_crossing: None,
            phase: Phase::Dealing,
            current_player: None,
            trick: None,
            history: Vec::with_capacity(hand_size),
            collecting_score: 0,
            trick_points: 0,
            penalty_adjustment: 0,
        })
    }

    pub fn standard(shuffled_deck: Vec<Card>) -> Result<Self, GameError> {
        Self::new(
            RuleSet::default(),
            TeamProgress::default(),
            None,
            PlayerId(0),
            shuffled_deck,
        )
    }

    pub const fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub const fn bidding(&self) -> &BidState {
        &self.bidding
    }

    /// 亮主阶段用于断电换庄和扳底顺序判定的当前庄家。首局尚未有人亮主时
    /// 为首位收牌者，普通亮主成功后仍由最终亮主结果覆盖实际庄家。
    pub const fn bidding_dealer(&self) -> PlayerId {
        self.bidding_dealer
    }

    pub const fn power_outage_used(&self) -> bool {
        self.power_outage_used
    }

    pub const fn five_trump_crossing(&self) -> Option<&FiveTrumpCrossingState> {
        self.five_trump_crossing.as_ref()
    }

    pub const fn bottom_copy(&self) -> Option<&BottomCopyState> {
        self.bottom_copy.as_ref()
    }

    pub const fn trump(&self) -> Option<Trump> {
        self.trump
    }

    pub const fn dealer(&self) -> Option<PlayerId> {
        self.dealer
    }

    pub const fn current_player(&self) -> Option<PlayerId> {
        self.current_player
    }

    pub fn buried(&self) -> &[Card] {
        &self.buried
    }

    pub const fn bottom_burier(&self) -> Option<PlayerId> {
        self.bottom_burier
    }

    pub fn history(&self) -> &[TrickRecord] {
        &self.history
    }

    /// 当前一圈的公开状态。返回副本以免向宿主暴露内部胜者索引。
    pub fn current_trick(&self) -> Option<TrickRecord> {
        let trick = self.trick.as_ref()?;
        Some(TrickRecord {
            leader: trick.leader,
            plays: trick.plays.clone(),
            winner: trick.plays[trick.winner_index].0,
            points: trick.points,
        })
    }

    pub const fn teams(&self) -> &TeamProgress {
        &self.teams
    }

    pub const fn collecting_score(&self) -> u32 {
        self.collecting_score
    }

    pub fn deal_next(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::Dealing {
            return Err(GameError::WrongPhase);
        }
        let dealt = self
            .players
            .iter()
            .map(|player| player.hand.len())
            .sum::<usize>();
        if dealt == RuleSet::PLAYER_COUNT * self.rules.hand_size() {
            self.phase = Phase::BiddingGrace;
            return Ok(ActionOutcome::DealComplete);
        }
        let player = PlayerId(
            (usize::from(self.first_recipient.0) + dealt) as u8 % RuleSet::PLAYER_COUNT as u8,
        );
        let card = self
            .deck
            .pop_front()
            .expect("validated deck has enough cards");
        self.players[usize::from(player.0)].hand.push(card);
        Ok(ActionOutcome::CardDealt { player, card })
    }

    pub fn deal_all(&mut self) -> Result<(), GameError> {
        while self.phase == Phase::Dealing {
            self.deal_next()?;
        }
        Ok(())
    }

    pub fn declare(
        &mut self,
        player: PlayerId,
        cards: &[Card],
    ) -> Result<ActionOutcome, GameError> {
        if !matches!(self.phase, Phase::Dealing | Phase::BiddingGrace) {
            return Err(GameError::WrongPhase);
        }
        let hand = self.player(player)?.hand.clone();
        self.bidding.declare(player, cards, &hand)?;
        if self.fixed_dealer.is_none() {
            self.dealer = self.bidding.current().map(|declaration| declaration.player);
        }
        Ok(ActionOutcome::DeclarationChanged)
    }

    /// 宿主在发牌结束后的五秒亮主窗口结束时调用。
    pub fn close_bidding_and_take_kitty(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BiddingGrace {
            return Err(GameError::WrongPhase);
        }
        let trump = match self.bidding.close() {
            Ok(trump) => trump.with_constant_trump(self.rules.constant_trump),
            Err(BidError::NoDeclaration) => {
                if self.rules.power_outage_dealer && !self.power_outage_used {
                    let dealer = next_player(self.bidding_dealer);
                    self.power_outage_used = true;
                    self.fixed_dealer = Some(dealer);
                    self.bidding_dealer = dealer;
                    self.dealer = Some(dealer);
                    self.bidding = BidState::new_with_rules(
                        self.teams.level(dealer.team()),
                        self.rules.deck_count,
                        self.rules.bid_with_joker,
                    );
                    return Ok(ActionOutcome::PowerOutageDealerChanged { dealer });
                }
                if self.rules.bottom_flip {
                    self.bottom_flip_index = 0;
                    self.bottom_flip_winner = None;
                    self.phase = Phase::BottomFlipping;
                    return Ok(ActionOutcome::BottomFlipStarted);
                }
                self.phase = Phase::RedealRequired;
                return Err(GameError::RedealRequired);
            }
            Err(error) => return Err(error.into()),
        };
        let dealer = self
            .fixed_dealer
            .unwrap_or_else(|| self.bidding.current().unwrap().player);
        self.bidding_dealer = dealer;
        self.take_kitty_for_dealer(dealer, trump);
        Ok(ActionOutcome::DealerTookKitty { dealer })
    }

    /// 扳底阶段依次查看仍留在牌堆中的底牌。翻牌本身不从底牌中移除；一旦
    /// 确定庄家，整副底牌仍会完整交给庄家。
    pub fn flip_next_bottom_card(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BottomFlipping {
            return Err(GameError::WrongPhase);
        }
        let Some(card) = self.deck.get(self.bottom_flip_index).copied() else {
            self.phase = Phase::RedealRequired;
            return Err(GameError::RedealRequired);
        };
        self.bottom_flip_index += 1;
        let matches = self
            .players
            .iter()
            .filter_map(|player| {
                let cards = player
                    .hand
                    .iter()
                    .copied()
                    .filter(|candidate| candidate.same_face(card))
                    .collect::<Vec<_>>();
                (!cards.is_empty()).then_some(BottomFlipMatch {
                    player: player.id,
                    cards,
                })
            })
            .collect::<Vec<_>>();
        let dealer = select_bottom_flip_dealer(&matches, self.bidding_dealer);
        if let Some(dealer) = dealer {
            let trump = Trump::new(self.teams.level(dealer.team()), card.suit())
                .expect("扳底只会从完整牌堆中翻出合法牌")
                .with_constant_trump(self.rules.constant_trump);
            self.fixed_dealer = Some(dealer);
            self.bidding_dealer = dealer;
            self.dealer = Some(dealer);
            self.trump = Some(trump);
            self.dealer_from_bottom_flip = true;
            self.bottom_flip_winner = Some((dealer, trump));
        } else if self.bottom_flip_index == self.deck.len() {
            // 所有底牌均无法打破两队同牌数平局时已没有下一张可翻。
            self.phase = Phase::RedealRequired;
        }
        Ok(ActionOutcome::BottomCardRevealed(BottomFlipReveal {
            card,
            matches,
            dealer,
        }))
    }

    /// 最终翻牌展示结束后再把完整底牌交给已经选中的庄家。
    pub fn complete_bottom_flip(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BottomFlipping {
            return Err(GameError::WrongPhase);
        }
        let Some((dealer, trump)) = self.bottom_flip_winner.take() else {
            return Err(GameError::WrongPhase);
        };
        self.take_kitty_for_dealer(dealer, trump);
        Ok(ActionOutcome::DealerTookKitty { dealer })
    }

    fn take_kitty_for_dealer(&mut self, dealer: PlayerId, trump: Trump) {
        self.kitty = self.deck.drain(..).collect();
        debug_assert_eq!(self.kitty.len(), self.rules.kitty_size());
        self.players[usize::from(dealer.0)]
            .hand
            .extend(self.kitty.iter().copied());
        self.trump = Some(trump);
        self.dealer = Some(dealer);
        self.phase = Phase::Burying;
    }

    pub fn bury(&mut self, player: PlayerId, cards: &[Card]) -> Result<ActionOutcome, GameError> {
        let dealer = self.dealer.unwrap();
        let bottom_copy_bury = match self.phase {
            Phase::Burying => {
                if player != dealer {
                    return Err(GameError::NotDealer);
                }
                false
            }
            Phase::BottomCopyBurying => {
                if self
                    .bottom_copy
                    .as_ref()
                    .and_then(BottomCopyState::bottom_holder)
                    != Some(player)
                {
                    return Err(GameError::NotBottomCopyPlayer);
                }
                true
            }
            _ => return Err(GameError::WrongPhase),
        };
        if cards.len() != self.rules.kitty_size() {
            return Err(GameError::WrongBuryCount {
                expected: self.rules.kitty_size(),
                actual: cards.len(),
            });
        }
        validate_cards_owned(cards, &self.players[usize::from(player.0)].hand)?;
        remove_cards(&mut self.players[usize::from(player.0)].hand, cards);
        self.buried = cards.to_vec();
        self.bottom_burier = Some(player);
        if bottom_copy_bury {
            let state = self.bottom_copy.as_mut().unwrap();
            state.bottom_holder = None;
            state.last_burier = player;
            let inquiry_complete = !self.advance_bottom_copy_inquiry();
            return Ok(ActionOutcome::BottomCopyBuryComplete {
                player,
                inquiry_complete,
            });
        }
        if self.rules.bottom_copy
            && !self.dealer_from_bottom_flip
            && self.start_bottom_copy_inquiry()
        {
            return Ok(ActionOutcome::BuryComplete { leader: dealer });
        }
        self.finish_after_burying();
        Ok(ActionOutcome::BuryComplete { leader: dealer })
    }

    pub fn choose_bottom_copy(
        &mut self,
        player: PlayerId,
        cards: Option<&[Card]>,
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BottomCopying {
            return Err(GameError::WrongPhase);
        }
        if self
            .bottom_copy
            .as_ref()
            .is_none_or(|state| state.current != Some(player) || state.last_burier == player)
        {
            return Err(GameError::NotBottomCopyPlayer);
        }
        if let Some(cards) = cards {
            let hand = self.players[usize::from(player.0)].hand.clone();
            let declaration = self
                .bidding
                .counter_after_close(player, cards, &hand)?
                .clone();
            self.trump = Some(
                Trump::new(self.bidding.level(), declaration.trump.trump_suit())
                    .expect("抄底反主保持普通级牌")
                    .with_constant_trump(self.rules.constant_trump),
            );
            let old_bottom = std::mem::take(&mut self.buried);
            self.bottom_burier = None;
            self.players[usize::from(player.0)].hand.extend(old_bottom);
            let state = self.bottom_copy.as_mut().unwrap();
            state.current = None;
            state.bottom_holder = Some(player);
            // 一次成功抄底会开启新一轮完整询问。当前抄底者不能立即反
            // 自己，但此前已经放弃或当时不符合条件的玩家仍可能用更强
            // 的牌继续抄底。
            state.remaining_without_copy = RuleSet::PLAYER_COUNT as u8;
            self.phase = Phase::BottomCopyBurying;
            return Ok(ActionOutcome::BottomCopyDecision {
                player,
                copied: true,
                inquiry_complete: false,
            });
        }
        self.bottom_copy.as_mut().unwrap().current = None;
        let inquiry_complete = !self.advance_bottom_copy_inquiry();
        Ok(ActionOutcome::BottomCopyDecision {
            player,
            copied: false,
            inquiry_complete,
        })
    }

    fn start_bottom_copy_inquiry(&mut self) -> bool {
        let dealer = self.dealer.unwrap();
        self.bottom_copy = Some(BottomCopyState {
            next_player: next_player(dealer),
            remaining_without_copy: RuleSet::PLAYER_COUNT as u8,
            current: None,
            bottom_holder: None,
            last_burier: dealer,
        });
        self.advance_bottom_copy_inquiry()
    }

    fn advance_bottom_copy_inquiry(&mut self) -> bool {
        loop {
            if self
                .bottom_copy
                .as_ref()
                .is_some_and(|state| state.remaining_without_copy == 0)
            {
                self.bottom_copy = None;
                self.finish_after_burying();
                return false;
            }
            let player = {
                let state = self.bottom_copy.as_mut().unwrap();
                let player = state.next_player;
                state.next_player = next_player(player);
                state.remaining_without_copy -= 1;
                player
            };
            // 当前声明者不能反自己的声明，上一位埋底者也不能立刻抄回
            // 自己刚埋的底。首轮的上一位埋底者是庄家；他人抄底并重埋后，
            // 原庄家与此前的声明者都可以在牌力足够时重新参与反抄。
            if self
                .bottom_copy
                .as_ref()
                .is_some_and(|state| state.last_burier == player)
                || self
                    .bidding
                    .current()
                    .is_some_and(|bid| bid.player == player)
            {
                continue;
            }
            let hand = &self.players[usize::from(player.0)].hand;
            if !self.bidding.counter_options(player, hand).is_empty() {
                self.bottom_copy.as_mut().unwrap().current = Some(player);
                self.phase = Phase::BottomCopying;
                self.current_player = None;
                return true;
            }
        }
    }

    fn finish_after_burying(&mut self) {
        let trump = self.trump.expect("庄家埋底前已经确定主牌");
        if self.rules.five_trump_crossing && trump.suit.is_some() {
            let eligible = std::array::from_fn(|index| {
                self.players[index]
                    .hand
                    .iter()
                    .filter(|card| trump.is_trump(**card))
                    .count()
                    <= 5
            });
            if eligible.iter().any(|eligible| *eligible) {
                self.five_trump_crossing = Some(FiveTrumpCrossingState {
                    stage: FiveTrumpCrossingStage::Deciding,
                    eligible,
                    declined: [false; RuleSet::PLAYER_COUNT],
                    outgoing: std::array::from_fn(|_| None),
                    returned: std::array::from_fn(|_| None),
                });
                self.phase = Phase::FiveTrumpCrossing;
                self.current_player = None;
            } else {
                self.start_playing();
            }
        } else {
            self.start_playing();
        }
    }

    /// 符合资格的玩家提交五张过江牌；`None` 表示明确放弃。本阶段的资格
    /// 固定取自埋底后的原始手牌，交换产生的新主牌数量不会再次触发过江。
    pub fn choose_five_trump_crossing(
        &mut self,
        player: PlayerId,
        cards: Option<&[Card]>,
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::FiveTrumpCrossing
            || self
                .five_trump_crossing
                .as_ref()
                .is_none_or(|state| state.stage != FiveTrumpCrossingStage::Deciding)
        {
            return Err(GameError::WrongPhase);
        }
        self.player(player)?;
        let state = self.five_trump_crossing.as_ref().unwrap();
        if !state.eligible(player) {
            return Err(GameError::CrossingNotEligible);
        }
        if state.decision_made(player) {
            return Err(GameError::CrossingAlreadyDecided);
        }

        let crossing = cards.is_some();
        if let Some(cards) = cards {
            if cards.len() != 5 {
                return Err(GameError::WrongCrossingCount {
                    expected: 5,
                    actual: cards.len(),
                });
            }
            let hand = &self.players[usize::from(player.0)].hand;
            validate_cards_owned(cards, hand)?;
            let trump = self.trump.unwrap();
            if hand
                .iter()
                .filter(|card| trump.is_trump(**card))
                .any(|card| !cards.contains(card))
            {
                return Err(GameError::CrossingMustIncludeAllTrumps);
            }
        }

        let state = self.five_trump_crossing.as_mut().unwrap();
        let index = usize::from(player.0);
        if let Some(cards) = cards {
            state.outgoing[index] = Some(cards.to_vec());
        } else {
            state.declined[index] = true;
        }
        let decisions_complete = state.pending_players().is_empty();
        if decisions_complete {
            self.apply_five_trump_outgoing();
        }
        Ok(ActionOutcome::FiveTrumpCrossingDecision {
            player,
            crossing,
            decisions_complete,
        })
    }

    /// 对家看过收到的过江牌后选择任意五张回交。所有回牌先分别锁定，待双方
    /// 队伍全部完成后再同时移动，因此同队两人可以彼此过江。
    pub fn return_five_trump_crossing(
        &mut self,
        player: PlayerId,
        cards: &[Card],
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::FiveTrumpCrossing
            || self
                .five_trump_crossing
                .as_ref()
                .is_none_or(|state| state.stage != FiveTrumpCrossingStage::Returning)
        {
            return Err(GameError::WrongPhase);
        }
        self.player(player)?;
        let state = self.five_trump_crossing.as_ref().unwrap();
        if !state.return_required(player) {
            return Err(GameError::CrossingReturnNotRequired);
        }
        if state.return_made(player) {
            return Err(GameError::CrossingAlreadyReturned);
        }
        if cards.len() != 5 {
            return Err(GameError::WrongCrossingCount {
                expected: 5,
                actual: cards.len(),
            });
        }
        validate_cards_owned(cards, &self.players[usize::from(player.0)].hand)?;
        self.five_trump_crossing.as_mut().unwrap().returned[usize::from(player.0)] =
            Some(cards.to_vec());
        let crossing_complete = self
            .five_trump_crossing
            .as_ref()
            .unwrap()
            .pending_players()
            .is_empty();
        if crossing_complete {
            self.apply_five_trump_returns();
            self.start_playing();
        }
        Ok(ActionOutcome::FiveTrumpCrossingReturn {
            player,
            crossing_complete,
        })
    }

    fn apply_five_trump_outgoing(&mut self) {
        let outgoing = self.five_trump_crossing.as_ref().unwrap().outgoing.clone();
        for (index, cards) in outgoing.iter().enumerate() {
            if let Some(cards) = cards {
                remove_cards(&mut self.players[index].hand, cards);
            }
        }
        for (index, cards) in outgoing.iter().enumerate() {
            if let Some(cards) = cards {
                self.players[usize::from(partner(PlayerId(index as u8)).0)]
                    .hand
                    .extend(cards.iter().copied());
            }
        }
        let state = self.five_trump_crossing.as_mut().unwrap();
        if state.outgoing.iter().any(Option::is_some) {
            state.stage = FiveTrumpCrossingStage::Returning;
        } else {
            self.start_playing();
        }
    }

    fn apply_five_trump_returns(&mut self) {
        let returned = self.five_trump_crossing.as_ref().unwrap().returned.clone();
        for (index, cards) in returned.iter().enumerate() {
            if let Some(cards) = cards {
                remove_cards(&mut self.players[index].hand, cards);
            }
        }
        for (index, cards) in returned.iter().enumerate() {
            if let Some(cards) = cards {
                self.players[usize::from(partner(PlayerId(index as u8)).0)]
                    .hand
                    .extend(cards.iter().copied());
            }
        }
    }

    fn start_playing(&mut self) {
        self.phase = Phase::Playing;
        self.current_player = self.dealer;
    }

    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[Card],
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::Playing {
            return Err(GameError::WrongPhase);
        }
        if self.current_player != Some(player) {
            return Err(GameError::NotPlayersTurn {
                expected: self.current_player.unwrap(),
                actual: player,
            });
        }
        let trump = self.trump.unwrap();
        let (play, failure) = if let Some(trick) = &self.trick {
            let play = validate_follow(
                &self.players[usize::from(player.0)].hand,
                cards,
                &trick.lead,
                trump,
            )?;
            (play, None)
        } else {
            validate_cards_owned(cards, &self.players[usize::from(player.0)].hand)?;
            let opponents = self
                .players
                .iter()
                .filter(|candidate| candidate.id != player)
                .map(|candidate| candidate.hand.as_slice())
                .collect::<Vec<_>>();
            match classify_lead(cards, trump, &self.rules, &opponents)? {
                TrickPlay::Accepted(play) => (play, None),
                TrickPlay::ThrowFailed(failure) => {
                    let forced = failure.forced.clone();
                    (forced, Some((failure.penalty_points, cards.to_vec())))
                }
            }
        };

        remove_cards(&mut self.players[usize::from(player.0)].hand, &play.cards);
        let play_points = play.cards.iter().map(|card| card.points()).sum::<u16>();
        let was_new_trick = self.trick.is_none();
        if was_new_trick {
            self.trick = Some(CurrentTrick {
                leader: player,
                lead: play.clone(),
                plays: vec![(player, play.clone())],
                winner_index: 0,
                points: play_points,
            });
        } else {
            let trick = self.trick.as_mut().unwrap();
            let winner_play = &trick.plays[trick.winner_index].1;
            if compare_for_trick(&trick.lead, winner_play, &play, trump).is_gt() {
                trick.winner_index = trick.plays.len();
            }
            trick.points = trick.points.saturating_add(play_points);
            trick.plays.push((player, play.clone()));
        }

        if let Some((penalty, _)) = &failure {
            self.apply_throw_penalty(player.team(), *penalty);
        }
        let trick_complete = self.trick.as_ref().unwrap().plays.len() == RuleSet::PLAYER_COUNT;
        if trick_complete {
            return self.complete_trick();
        }
        let next = PlayerId((player.0 + 1) % RuleSet::PLAYER_COUNT as u8);
        self.current_player = Some(next);
        if let Some((penalty_points, attempted)) = failure {
            Ok(ActionOutcome::ThrowFailed {
                player,
                attempted,
                forced: play,
                penalty_points,
                next,
            })
        } else {
            Ok(ActionOutcome::Played { player, next })
        }
    }

    fn complete_trick(&mut self) -> Result<ActionOutcome, GameError> {
        let trick = self.trick.take().unwrap();
        let winner = trick.plays[trick.winner_index].0;
        if winner.team() == self.dealer.unwrap().team().other() {
            self.collecting_score = self
                .collecting_score
                .saturating_add(u32::from(trick.points));
            self.trick_points = self.trick_points.saturating_add(u32::from(trick.points));
        }
        let record = TrickRecord {
            leader: trick.leader,
            plays: trick.plays,
            winner,
            points: trick.points,
        };
        self.history.push(record.clone());
        self.current_player = Some(winner);
        if self.players.iter().all(|player| player.hand.is_empty()) {
            let result = self.finish_hand(winner, &record);
            self.phase = Phase::Finished(result.clone());
            self.current_player = None;
            return Ok(ActionOutcome::HandComplete(result));
        }
        Ok(ActionOutcome::TrickComplete(record))
    }

    fn apply_throw_penalty(&mut self, offender: TeamId, points: u16) {
        let dealer_team = self.dealer.unwrap().team();
        if offender == dealer_team {
            self.collecting_score = self.collecting_score.saturating_add(u32::from(points));
            self.penalty_adjustment = self.penalty_adjustment.saturating_add(i32::from(points));
        } else {
            let before = self.collecting_score;
            self.collecting_score = self.collecting_score.saturating_sub(u32::from(points));
            self.penalty_adjustment = self
                .penalty_adjustment
                .saturating_sub((before - self.collecting_score) as i32);
        }
    }

    fn finish_hand(&mut self, last_winner: PlayerId, last_trick: &TrickRecord) -> HandResult {
        let dealer = self.dealer.unwrap();
        let dealer_team = dealer.team();
        let collecting_team = dealer_team.other();
        let kitty_points = self.buried.iter().map(|card| card.points()).sum::<u16>();
        let (kitty_multiplier, kitty_award) = if last_winner.team() == collecting_team {
            let winner_play = last_trick
                .plays
                .iter()
                .find(|(player, _)| *player == last_winner)
                .map(|(_, play)| play)
                .unwrap();
            let multiplier = winner_play.kitty_multiplier();
            (multiplier, u32::from(kitty_points) * multiplier)
        } else {
            (0, 0)
        };
        self.collecting_score = self.collecting_score.saturating_add(kitty_award);

        let step = self.rules.score_step();
        let takeover = self.rules.takeover_score();
        let (promoted_team, promoted_steps, next_dealer) = if self.collecting_score == 0 {
            (dealer_team, 3, partner(dealer))
        } else if self.collecting_score < step {
            (dealer_team, 2, partner(dealer))
        } else if self.collecting_score < takeover {
            (dealer_team, 1, partner(dealer))
        } else if self.collecting_score < takeover + step {
            (collecting_team, 0, next_player(dealer))
        } else {
            let steps = ((self.collecting_score - takeover) / step) as u8;
            (collecting_team, steps, next_player(dealer))
        };
        self.teams.promote(
            promoted_team,
            promoted_steps,
            self.rules.mandatory_five_ten_king_ace,
        );
        HandResult {
            dealer,
            dealer_team,
            collecting_team,
            trick_points: self.trick_points,
            penalty_adjustment: self.penalty_adjustment,
            kitty_points,
            kitty_multiplier,
            collecting_score: self.collecting_score,
            promoted_team,
            promoted_steps,
            next_dealer,
            levels: self.teams.levels,
        }
    }

    fn player(&self, player: PlayerId) -> Result<&PlayerState, GameError> {
        self.players
            .get(usize::from(player.0))
            .ok_or(GameError::InvalidPlayer(player))
    }
}

fn select_bottom_flip_dealer(
    matches: &[BottomFlipMatch],
    current_dealer: PlayerId,
) -> Option<PlayerId> {
    if matches.len() == 1 {
        return Some(matches[0].player);
    }
    if matches.is_empty() {
        return None;
    }
    let mut team_totals = [0_usize; 2];
    for matched in matches {
        team_totals[usize::from(matched.player.team().0)] += matched.cards.len();
    }
    if team_totals[0] == team_totals[1] {
        return None;
    }
    let winning_team = usize::from(team_totals[1] > team_totals[0]);
    let most_cards = matches
        .iter()
        .filter(|matched| usize::from(matched.player.team().0) == winning_team)
        .map(|matched| matched.cards.len())
        .max()?;
    matches
        .iter()
        .filter(|matched| {
            usize::from(matched.player.team().0) == winning_team
                && matched.cards.len() == most_cards
        })
        .min_by_key(|matched| {
            (matched.player.0 + RuleSet::PLAYER_COUNT as u8 - current_dealer.0)
                % RuleSet::PLAYER_COUNT as u8
        })
        .map(|matched| matched.player)
}

fn next_player(player: PlayerId) -> PlayerId {
    PlayerId((player.0 + 1) % RuleSet::PLAYER_COUNT as u8)
}

const fn partner(player: PlayerId) -> PlayerId {
    PlayerId((player.0 + 2) % RuleSet::PLAYER_COUNT as u8)
}

fn remove_cards(hand: &mut Vec<Card>, cards: &[Card]) {
    for card in cards {
        let index = hand
            .iter()
            .position(|candidate| candidate == card)
            .expect("ownership validated before removal");
        hand.remove(index);
    }
}

fn validate_cards_owned(cards: &[Card], hand: &[Card]) -> Result<(), GameError> {
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() || cards.iter().any(|card| !hand.contains(card)) {
        return Err(GameError::CardsNotOwned);
    }
    Ok(())
}

fn validate_deck(deck: &[Card], rules: RuleSet) -> Result<(), GameError> {
    let expected = build_deck_for(rules.deck_count);
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    Rules(RuleError),
    Bid(BidError),
    Play(crate::PlayError),
    Follow(FollowError),
    InvalidPlayer(PlayerId),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    WrongPhase,
    RedealRequired,
    NotDealer,
    WrongBuryCount {
        expected: usize,
        actual: usize,
    },
    CrossingNotEligible,
    CrossingAlreadyDecided,
    WrongCrossingCount {
        expected: usize,
        actual: usize,
    },
    CrossingMustIncludeAllTrumps,
    CrossingReturnNotRequired,
    CrossingAlreadyReturned,
    NotBottomCopyPlayer,
    CardsNotOwned,
    NotPlayersTurn {
        expected: PlayerId,
        actual: PlayerId,
    },
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rules(error) => error.fmt(f),
            Self::Bid(error) => error.fmt(f),
            Self::Play(error) => error.fmt(f),
            Self::Follow(error) => error.fmt(f),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆应为 {expected} 张，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
            Self::WrongPhase => f.write_str("当前阶段不能执行此操作"),
            Self::RedealRequired => f.write_str("无人亮主，必须重新发牌"),
            Self::NotDealer => f.write_str("只有庄家可以埋底"),
            Self::WrongBuryCount { expected, actual } => {
                write!(f, "必须埋 {expected} 张底牌，实际为 {actual}")
            }
            Self::CrossingNotEligible => f.write_str("你的主牌多于五张，不能五主过江"),
            Self::CrossingAlreadyDecided => f.write_str("你已经完成五主过江选择"),
            Self::WrongCrossingCount { expected, actual } => {
                write!(f, "五主过江必须选择 {expected} 张牌，实际为 {actual}")
            }
            Self::CrossingMustIncludeAllTrumps => f.write_str("五主过江必须选择手中的全部主牌"),
            Self::CrossingReturnNotRequired => f.write_str("当前不需要你归还过江牌"),
            Self::CrossingAlreadyReturned => f.write_str("你已经归还过江牌"),
            Self::NotBottomCopyPlayer => f.write_str("当前没有轮到该玩家抄底或重新埋底"),
            Self::CardsNotOwned => f.write_str("提交的牌不全在玩家手中"),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "当前应由 {:?} 出牌，不是 {:?}", expected, actual)
            }
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::Rules(value)
    }
}

impl From<BidError> for GameError {
    fn from(value: BidError) -> Self {
        Self::Bid(value)
    }
}

impl From<crate::PlayError> for GameError {
    fn from(value: crate::PlayError) -> Self {
        Self::Play(value)
    }
}

impl From<FollowError> for GameError {
    fn from(value: FollowError) -> Self {
        Self::Follow(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::play::classify_cards;
    use crate::{BidTrump, Suit, ThrowPenalty};

    use super::*;

    fn dealt_game(rules: RuleSet) -> GameState {
        let mut deck = build_deck_for(rules.deck_count);
        // 保证 0 号玩家在第一张就拿到方块 2，可以确定性亮主。
        let target = Card::suited(0, Suit::Diamond, Rank::Two);
        let index = deck.iter().position(|card| *card == target).unwrap();
        deck.swap(0, index);
        let mut game =
            GameState::new(rules, TeamProgress::default(), None, PlayerId(0), deck).unwrap();
        game.deal_next().unwrap();
        game.declare(PlayerId(0), &[target]).unwrap();
        game.deal_all().unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        game
    }

    #[test]
    fn first_hand_final_counter_becomes_dealer_but_later_hands_keep_fixed_dealer() {
        let mut deck = build_deck();
        let diamond = [
            Card::suited(0, Suit::Diamond, Rank::Two),
            Card::suited(1, Suit::Diamond, Rank::Two),
        ];
        let spade = [
            Card::suited(0, Suit::Spade, Rank::Two),
            Card::suited(1, Suit::Spade, Rank::Two),
        ];
        for (target_index, card) in [
            (0, diamond[0]),
            (1, spade[0]),
            (4, diamond[1]),
            (5, spade[1]),
        ] {
            let index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, index);
        }
        let later_deck = deck.clone();
        let mut game = GameState::standard(deck).unwrap();
        for _ in 0..6 {
            game.deal_next().unwrap();
        }
        assert_eq!(game.dealer(), None);
        game.declare(PlayerId(0), &diamond[..1]).unwrap();
        assert_eq!(game.dealer(), Some(PlayerId(0)));
        game.declare(PlayerId(1), &spade).unwrap();
        assert_eq!(game.dealer(), Some(PlayerId(1)));
        assert_eq!(
            game.bidding().current().unwrap().trump,
            BidTrump::Suit(Suit::Spade)
        );
        game.deal_all().unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        assert_eq!(game.dealer(), Some(PlayerId(1)));

        let mut later = GameState::new(
            RuleSet::default(),
            TeamProgress::default(),
            Some(PlayerId(0)),
            PlayerId(0),
            later_deck,
        )
        .unwrap();
        assert_eq!(later.dealer(), Some(PlayerId(0)));
        for _ in 0..6 {
            later.deal_next().unwrap();
        }
        later.declare(PlayerId(0), &diamond[..1]).unwrap();
        later.declare(PlayerId(1), &spade).unwrap();
        assert_eq!(later.dealer(), Some(PlayerId(0)));
        later.deal_all().unwrap();
        later.close_bidding_and_take_kitty().unwrap();
        assert_eq!(later.dealer(), Some(PlayerId(0)));
        assert_eq!(later.trump().unwrap().suit, Some(Suit::Spade));
    }

    #[test]
    fn sequential_bottom_copies_change_trump_but_never_the_first_hands_dealer() {
        let rules = RuleSet {
            // 配置开启不等于实际通过扳底定庄；正常亮主仍应允许抄底。
            bottom_flip: true,
            bottom_copy: true,
            ..RuleSet::default()
        };
        let diamond = Card::suited(0, Suit::Diamond, Rank::Two);
        let hearts = [
            Card::suited(0, Suit::Heart, Rank::Two),
            Card::suited(1, Suit::Heart, Rank::Two),
        ];
        let spades = [
            Card::suited(0, Suit::Spade, Rank::Two),
            Card::suited(1, Suit::Spade, Rank::Two),
        ];
        let mut deck = build_deck();
        for (target_index, card) in [
            (0, diamond),
            (1, hearts[0]),
            (5, hearts[1]),
            (2, spades[0]),
            (6, spades[1]),
        ] {
            let index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, index);
        }
        let mut game =
            GameState::new(rules, TeamProgress::default(), None, PlayerId(0), deck).unwrap();
        game.deal_all().unwrap();
        game.declare(PlayerId(0), &[diamond]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        let first_bottom = game.players()[0].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(0), &first_bottom).unwrap();
        assert_eq!(game.phase(), &Phase::BottomCopying);
        assert_eq!(game.bottom_copy().unwrap().current(), Some(PlayerId(1)));

        game.choose_bottom_copy(PlayerId(1), Some(&hearts)).unwrap();
        assert_eq!(game.phase(), &Phase::BottomCopyBurying);
        assert_eq!(game.dealer(), Some(PlayerId(0)));
        assert_eq!(game.trump().unwrap().suit, Some(Suit::Heart));
        assert_eq!(game.players()[1].hand.len(), 33);
        let second_bottom = game.players()[1].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(1), &second_bottom).unwrap();
        assert_eq!(game.bottom_copy().unwrap().current(), Some(PlayerId(2)));

        game.choose_bottom_copy(PlayerId(2), Some(&spades)).unwrap();
        assert_eq!(game.dealer(), Some(PlayerId(0)));
        assert_eq!(game.trump().unwrap().suit, Some(Suit::Spade));
        let final_bottom = game.players()[2].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(2), &final_bottom).unwrap();
        while game.phase() == &Phase::BottomCopying {
            let player = game.bottom_copy().unwrap().current().unwrap();
            game.choose_bottom_copy(player, None).unwrap();
        }
        assert_eq!(game.phase(), &Phase::Playing);
        assert_eq!(game.dealer(), Some(PlayerId(0)));
        assert_eq!(game.current_player(), Some(PlayerId(0)));
    }

    #[test]
    fn successful_bottom_copy_starts_a_new_round_and_non_dealer_original_bidder_can_copy_back() {
        let rules = RuleSet {
            bottom_copy: true,
            ..RuleSet::default()
        };
        let diamond = Card::suited(0, Suit::Diamond, Rank::Two);
        let hearts = [
            Card::suited(0, Suit::Heart, Rank::Two),
            Card::suited(1, Suit::Heart, Rank::Two),
        ];
        let small_jokers = [Card::small_joker(0), Card::small_joker(1)];
        let mut deck = build_deck();
        for (target_index, card) in [
            (0, diamond),
            (1, hearts[0]),
            (5, hearts[1]),
            (4, small_jokers[0]),
            (8, small_jokers[1]),
        ] {
            let index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, index);
        }
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            Some(PlayerId(3)),
            PlayerId(0),
            deck,
        )
        .unwrap();
        game.deal_all().unwrap();
        game.declare(PlayerId(0), &[diamond]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        let first_bottom = game.players()[3]
            .hand
            .iter()
            .copied()
            .filter(|card| *card != diamond && !small_jokers.contains(card))
            .take(rules.kitty_size())
            .collect::<Vec<_>>();
        game.bury(PlayerId(3), &first_bottom).unwrap();
        assert_eq!(game.bottom_copy().unwrap().current(), Some(PlayerId(1)));

        game.choose_bottom_copy(PlayerId(1), Some(&hearts)).unwrap();
        let second_bottom = game.players()[1].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(1), &second_bottom).unwrap();

        // 2、3 号没有更强反主牌，询问会绕回最初亮主的 0 号。
        assert_eq!(game.phase(), &Phase::BottomCopying);
        assert_eq!(game.bottom_copy().unwrap().current(), Some(PlayerId(0)));
        game.choose_bottom_copy(PlayerId(0), Some(&small_jokers))
            .unwrap();
        assert_eq!(game.phase(), &Phase::BottomCopyBurying);
        assert_eq!(game.dealer(), Some(PlayerId(3)));
        assert_eq!(game.trump().unwrap().suit, None);
    }

    #[test]
    fn dealer_cannot_copy_their_own_bottom_when_another_player_declared_trump() {
        let rules = RuleSet {
            bottom_copy: true,
            ..RuleSet::default()
        };
        let diamond = Card::suited(0, Suit::Diamond, Rank::Two);
        let hearts = [
            Card::suited(0, Suit::Heart, Rank::Two),
            Card::suited(1, Suit::Heart, Rank::Two),
        ];
        let mut deck = build_deck();
        for (target_index, card) in [(1, diamond), (0, hearts[0]), (4, hearts[1])] {
            let index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, index);
        }
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            Some(PlayerId(0)),
            PlayerId(0),
            deck,
        )
        .unwrap();
        game.deal_all().unwrap();
        game.declare(PlayerId(1), &[diamond]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        assert_eq!(game.dealer(), Some(PlayerId(0)));

        let buried = game.players()[0]
            .hand
            .iter()
            .copied()
            .filter(|card| !hearts.contains(card))
            .take(rules.kitty_size())
            .collect::<Vec<_>>();
        game.bury(PlayerId(0), &buried).unwrap();

        // 只有庄家持有能反方块单张的一对红桃级牌，但庄家不能抄自己的底。
        assert_eq!(game.phase(), &Phase::Playing);
        assert!(game.bottom_copy().is_none());
        assert_eq!(game.trump().unwrap().suit, Some(Suit::Diamond));
    }

    #[test]
    fn original_dealer_can_copy_after_another_player_reburies_the_bottom() {
        let rules = RuleSet {
            bottom_copy: true,
            ..RuleSet::default()
        };
        let diamond = Card::suited(0, Suit::Diamond, Rank::Two);
        let hearts = [
            Card::suited(0, Suit::Heart, Rank::Two),
            Card::suited(1, Suit::Heart, Rank::Two),
        ];
        let small_jokers = [Card::small_joker(0), Card::small_joker(1)];
        let mut deck = build_deck();
        for (target_index, card) in [
            (0, diamond),
            (1, hearts[0]),
            (5, hearts[1]),
            (3, small_jokers[0]),
            (7, small_jokers[1]),
        ] {
            let index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, index);
        }
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            Some(PlayerId(3)),
            PlayerId(0),
            deck,
        )
        .unwrap();
        game.deal_all().unwrap();
        game.declare(PlayerId(0), &[diamond]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();

        let first_bottom = game.players()[3]
            .hand
            .iter()
            .copied()
            .filter(|card| !small_jokers.contains(card))
            .take(rules.kitty_size())
            .collect::<Vec<_>>();
        game.bury(PlayerId(3), &first_bottom).unwrap();
        assert_eq!(game.bottom_copy().unwrap().current(), Some(PlayerId(1)));

        game.choose_bottom_copy(PlayerId(1), Some(&hearts)).unwrap();
        let second_bottom = game.players()[1].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(1), &second_bottom).unwrap();

        // 底牌已由 1 号重新埋过，最初的庄家 3 号不再是“上一位埋底者”。
        assert_eq!(game.bottom_copy().unwrap().current(), Some(PlayerId(3)));
        game.choose_bottom_copy(PlayerId(3), Some(&small_jokers))
            .unwrap();
        assert_eq!(game.phase(), &Phase::BottomCopyBurying);
        assert_eq!(game.dealer(), Some(PlayerId(3)));
        assert_eq!(game.trump().unwrap().suit, None);
    }

    #[test]
    fn bottom_flip_dealer_skips_bottom_copy_even_when_enabled() {
        let rules = RuleSet {
            bottom_flip: true,
            bottom_copy: true,
            ..RuleSet::default()
        };
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            None,
            PlayerId(0),
            build_deck(),
        )
        .unwrap();
        game.deal_all().unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        game.flip_next_bottom_card().unwrap();
        let dealer = game.dealer().unwrap();
        game.complete_bottom_flip().unwrap();
        let buried = game.players()[usize::from(dealer.0)].hand[..rules.kitty_size()].to_vec();
        game.bury(dealer, &buried).unwrap();
        assert_eq!(game.phase(), &Phase::Playing);
        assert!(game.bottom_copy().is_none());
    }

    #[test]
    fn nobody_declaring_requires_a_redeal() {
        let mut game = GameState::standard(build_deck()).unwrap();
        game.deal_all().unwrap();
        assert_eq!(
            game.close_bidding_and_take_kitty(),
            Err(GameError::RedealRequired)
        );
        assert_eq!(game.phase(), &Phase::RedealRequired);
    }

    #[test]
    fn power_outage_keeps_hands_rotates_dealer_and_uses_the_new_teams_level() {
        let rules = RuleSet {
            power_outage_dealer: true,
            ..RuleSet::default()
        };
        let teams = TeamProgress {
            levels: [Rank::Ten, Rank::Five],
        };
        let mut game =
            GameState::new(rules, teams, Some(PlayerId(0)), PlayerId(0), build_deck()).unwrap();
        game.deal_all().unwrap();
        let hands = game
            .players()
            .iter()
            .map(|player| player.hand.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            game.close_bidding_and_take_kitty().unwrap(),
            ActionOutcome::PowerOutageDealerChanged {
                dealer: PlayerId(1)
            }
        );
        assert_eq!(game.phase(), &Phase::BiddingGrace);
        assert_eq!(game.dealer(), Some(PlayerId(1)));
        assert_eq!(game.bidding().level(), Rank::Five);
        assert_eq!(
            game.players()
                .iter()
                .map(|player| player.hand.clone())
                .collect::<Vec<_>>(),
            hands
        );

        let bidder = PlayerId(3);
        let bid = game.players()[usize::from(bidder.0)]
            .hand
            .iter()
            .copied()
            .find(|card| card.rank() == Rank::Five)
            .unwrap();
        game.declare(bidder, &[bid]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        assert_eq!(game.dealer(), Some(PlayerId(1)));
        assert_eq!(game.trump().unwrap().level, Rank::Five);
        assert_eq!(game.trump().unwrap().suit, bid.suit());
    }

    #[test]
    fn a_second_power_outage_redeals_unless_bottom_flip_is_enabled() {
        let rules = RuleSet {
            power_outage_dealer: true,
            ..RuleSet::default()
        };
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            Some(PlayerId(0)),
            PlayerId(0),
            build_deck(),
        )
        .unwrap();
        game.deal_all().unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        assert_eq!(
            game.close_bidding_and_take_kitty(),
            Err(GameError::RedealRequired)
        );
    }

    #[test]
    fn bottom_flip_reveals_matches_and_sets_dealer_level_and_suit() {
        let rules = RuleSet {
            bottom_flip: true,
            ..RuleSet::default()
        };
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            None,
            PlayerId(0),
            build_deck(),
        )
        .unwrap();
        game.deal_all().unwrap();
        assert_eq!(
            game.close_bidding_and_take_kitty().unwrap(),
            ActionOutcome::BottomFlipStarted
        );
        let ActionOutcome::BottomCardRevealed(reveal) = game.flip_next_bottom_card().unwrap()
        else {
            unreachable!();
        };
        assert_eq!(reveal.card, Card::suited(1, Suit::Spade, Rank::Nine));
        assert_eq!(reveal.dealer, Some(PlayerId(2)));
        assert_eq!(reveal.matches.len(), 1);
        assert_eq!(reveal.matches[0].player, PlayerId(2));
        assert_eq!(game.phase(), &Phase::BottomFlipping);
        assert_eq!(game.dealer(), Some(PlayerId(2)));
        assert_eq!(game.trump().unwrap().level, Rank::Two);
        assert_eq!(game.trump().unwrap().suit, Some(Suit::Spade));
        assert_eq!(game.players()[2].hand.len(), 25);
        game.complete_bottom_flip().unwrap();
        assert_eq!(game.phase(), &Phase::Burying);
        assert_eq!(game.players()[2].hand.len(), 33);
    }

    #[test]
    fn bottom_flip_dealer_selection_obeys_team_total_personal_count_and_distance() {
        let cards = |count: usize| {
            (0..count)
                .map(|deck| Card::suited(deck as u8, Suit::Heart, Rank::Ace))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            select_bottom_flip_dealer(
                &[BottomFlipMatch {
                    player: PlayerId(3),
                    cards: cards(1),
                }],
                PlayerId(0),
            ),
            Some(PlayerId(3))
        );
        assert_eq!(
            select_bottom_flip_dealer(
                &[
                    BottomFlipMatch {
                        player: PlayerId(0),
                        cards: cards(1),
                    },
                    BottomFlipMatch {
                        player: PlayerId(1),
                        cards: cards(1),
                    },
                ],
                PlayerId(0),
            ),
            None
        );
        assert_eq!(
            select_bottom_flip_dealer(
                &[
                    BottomFlipMatch {
                        player: PlayerId(0),
                        cards: cards(2),
                    },
                    BottomFlipMatch {
                        player: PlayerId(2),
                        cards: cards(1),
                    },
                    BottomFlipMatch {
                        player: PlayerId(1),
                        cards: cards(2),
                    },
                ],
                PlayerId(1),
            ),
            Some(PlayerId(0))
        );
        assert_eq!(
            select_bottom_flip_dealer(
                &[
                    BottomFlipMatch {
                        player: PlayerId(0),
                        cards: cards(2),
                    },
                    BottomFlipMatch {
                        player: PlayerId(2),
                        cards: cards(2),
                    },
                    BottomFlipMatch {
                        player: PlayerId(1),
                        cards: cards(3),
                    },
                ],
                PlayerId(1),
            ),
            Some(PlayerId(2))
        );
    }

    #[test]
    fn dealer_may_bury_any_eight_cards() {
        let mut game = dealt_game(RuleSet::default());
        let dealer = game.dealer().unwrap();
        let buried = game.players()[usize::from(dealer.0)].hand[..8].to_vec();
        game.bury(dealer, &buried).unwrap();
        assert_eq!(game.buried(), buried);
        assert_eq!(game.bottom_burier(), Some(dealer));
        assert_eq!(game.players()[usize::from(dealer.0)].hand.len(), 25);
        assert_eq!(game.current_player(), Some(dealer));
    }

    #[test]
    fn constant_trump_first_hand_starts_both_teams_at_three() {
        let rules = RuleSet {
            constant_trump: true,
            ..RuleSet::default()
        };
        let game = GameState::new(
            rules,
            TeamProgress::default(),
            None,
            PlayerId(0),
            build_deck(),
        )
        .unwrap();

        assert_eq!(game.teams().levels, [Rank::Three, Rank::Three]);
        assert_eq!(game.bidding().level(), Rank::Three);
    }

    #[test]
    fn dealer_throw_penalty_adds_to_collectors_and_collectors_never_go_below_zero() {
        let mut game = dealt_game(RuleSet {
            throw_penalty: ThrowPenalty::TenPerCard,
            ..RuleSet::default()
        });
        game.dealer = Some(PlayerId(0));
        game.apply_throw_penalty(TeamId(0), 50);
        assert_eq!(game.collecting_score(), 50);
        game.apply_throw_penalty(TeamId(1), 80);
        assert_eq!(game.collecting_score(), 0);
        game.apply_throw_penalty(TeamId(1), 10);
        assert_eq!(game.collecting_score(), 0);
    }

    #[test]
    fn failed_throw_keeps_the_attempt_visible_but_only_plays_the_forced_low_card() {
        let mut game = GameState::standard(build_deck()).unwrap();
        let low = Card::suited(0, Suit::Diamond, Rank::Three);
        let high = Card::suited(0, Suit::Diamond, Rank::Nine);
        let beating_card = Card::suited(0, Suit::Diamond, Rank::Four);
        game.rules.throw_penalty = ThrowPenalty::TenPerCard;
        game.phase = Phase::Playing;
        game.trump = Some(Trump::new(Rank::Two, Some(Suit::Spade)).unwrap());
        game.dealer = Some(PlayerId(0));
        game.current_player = Some(PlayerId(0));
        game.players[0].hand = vec![low, high];
        game.players[1].hand = vec![beating_card];

        let attempted = vec![high, low];
        let outcome = game.play_cards(PlayerId(0), &attempted).unwrap();
        let ActionOutcome::ThrowFailed {
            player,
            attempted: visible_attempt,
            forced,
            penalty_points,
            next,
        } = outcome
        else {
            panic!("the beatable mixed single throw should fail");
        };

        assert_eq!(player, PlayerId(0));
        assert_eq!(visible_attempt, attempted);
        assert_eq!(forced.cards, vec![low]);
        assert_eq!(penalty_points, 20);
        assert_eq!(next, PlayerId(1));
        assert_eq!(game.players[0].hand, vec![high]);
        assert_eq!(game.collecting_score(), 20);
        assert_eq!(game.current_player(), Some(PlayerId(1)));
    }

    #[test]
    fn settlement_thresholds_and_next_dealers_match_the_declared_rules() {
        let cases = [
            (0, TeamId(0), 3, PlayerId(2)),
            (35, TeamId(0), 2, PlayerId(2)),
            (75, TeamId(0), 1, PlayerId(2)),
            (80, TeamId(1), 0, PlayerId(1)),
            (120, TeamId(1), 1, PlayerId(1)),
            (160, TeamId(1), 2, PlayerId(1)),
            (200, TeamId(1), 3, PlayerId(1)),
        ];
        for (score, promoted_team, steps, next_dealer) in cases {
            let mut game = dealt_game(RuleSet::default());
            game.dealer = Some(PlayerId(0));
            game.collecting_score = score;
            game.buried.clear();
            let play = classify_cards(
                &[Card::suited(0, Suit::Diamond, Rank::Three)],
                game.trump.unwrap(),
            )
            .unwrap();
            let last = TrickRecord {
                leader: PlayerId(0),
                plays: vec![(PlayerId(0), play)],
                winner: PlayerId(0),
                points: 0,
            };
            let result = game.finish_hand(PlayerId(0), &last);
            assert_eq!(result.promoted_team, promoted_team);
            assert_eq!(result.promoted_steps, steps);
            assert_eq!(result.next_dealer, next_dealer);
        }
    }

    #[test]
    fn three_deck_settlement_uses_sixty_point_bands() {
        let cases = [
            (0, TeamId(0), 3, PlayerId(2)),
            (59, TeamId(0), 2, PlayerId(2)),
            (60, TeamId(0), 1, PlayerId(2)),
            (119, TeamId(0), 1, PlayerId(2)),
            (120, TeamId(1), 0, PlayerId(1)),
            (179, TeamId(1), 0, PlayerId(1)),
            (180, TeamId(1), 1, PlayerId(1)),
            (240, TeamId(1), 2, PlayerId(1)),
        ];
        for (score, promoted_team, steps, next_dealer) in cases {
            let mut game = dealt_game(RuleSet {
                deck_count: 3,
                ..RuleSet::default()
            });
            game.dealer = Some(PlayerId(0));
            game.collecting_score = score;
            game.buried.clear();
            let play = classify_cards(
                &[Card::suited(0, Suit::Diamond, Rank::Three)],
                game.trump.unwrap(),
            )
            .unwrap();
            let last = TrickRecord {
                leader: PlayerId(0),
                plays: vec![(PlayerId(0), play)],
                winner: PlayerId(0),
                points: 0,
            };
            let result = game.finish_hand(PlayerId(0), &last);
            assert_eq!(result.promoted_team, promoted_team, "score={score}");
            assert_eq!(result.promoted_steps, steps, "score={score}");
            assert_eq!(result.next_dealer, next_dealer, "score={score}");
        }
    }

    #[test]
    fn four_deck_settlement_uses_eighty_point_bands() {
        let cases = [
            (0, TeamId(0), 3),
            (79, TeamId(0), 2),
            (80, TeamId(0), 1),
            (159, TeamId(0), 1),
            (160, TeamId(1), 0),
            (239, TeamId(1), 0),
            (240, TeamId(1), 1),
            (320, TeamId(1), 2),
        ];
        for (score, promoted_team, steps) in cases {
            let mut game = dealt_game(RuleSet {
                deck_count: 4,
                ..RuleSet::default()
            });
            game.dealer = Some(PlayerId(0));
            game.collecting_score = score;
            game.buried.clear();
            let play = classify_cards(
                &[Card::suited(0, Suit::Diamond, Rank::Three)],
                game.trump.unwrap(),
            )
            .unwrap();
            let last = TrickRecord {
                leader: PlayerId(0),
                plays: vec![(PlayerId(0), play)],
                winner: PlayerId(0),
                points: 0,
            };
            let result = game.finish_hand(PlayerId(0), &last);
            assert_eq!(result.promoted_team, promoted_team, "score={score}");
            assert_eq!(result.promoted_steps, steps, "score={score}");
        }
    }

    fn five_trump_crossing_game(trump_suit: Option<Suit>) -> (GameState, Vec<Card>) {
        let rules = RuleSet {
            five_trump_crossing: true,
            ..RuleSet::default()
        };
        let mut game = GameState::new(
            rules,
            TeamProgress::default(),
            None,
            PlayerId(0),
            build_deck(),
        )
        .unwrap();
        let buried = [
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Jack,
        ]
        .into_iter()
        .map(|rank| Card::suited(0, Suit::Diamond, rank))
        .collect::<Vec<_>>();
        let player_zero = [
            Card::suited(0, Suit::Heart, Rank::Three),
            Card::suited(0, Suit::Heart, Rank::Four),
            Card::big_joker(0),
            Card::suited(0, Suit::Club, Rank::Three),
            Card::suited(0, Suit::Club, Rank::Four),
            Card::suited(0, Suit::Club, Rank::Five),
            Card::suited(0, Suit::Club, Rank::Six),
            Card::suited(0, Suit::Club, Rank::Seven),
            Card::suited(0, Suit::Club, Rank::Eight),
        ];
        let player_two = [
            Card::suited(0, Suit::Heart, Rank::Five),
            Card::small_joker(0),
            Card::suited(0, Suit::Spade, Rank::Three),
            Card::suited(0, Suit::Spade, Rank::Four),
            Card::suited(0, Suit::Spade, Rank::Five),
            Card::suited(0, Suit::Spade, Rank::Six),
            Card::suited(0, Suit::Spade, Rank::Seven),
            Card::suited(0, Suit::Spade, Rank::Eight),
        ];
        game.players[0].hand = [buried.as_slice(), player_zero.as_slice()].concat();
        game.players[1].hand = [
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Jack,
            Rank::Queen,
        ]
        .into_iter()
        .map(|rank| Card::suited(0, Suit::Heart, rank))
        .collect();
        game.players[2].hand = player_two.to_vec();
        game.players[3].hand = vec![
            Card::suited(0, Suit::Heart, Rank::King),
            Card::suited(0, Suit::Heart, Rank::Ace),
            Card::suited(0, Suit::Diamond, Rank::Ten),
            Card::suited(0, Suit::Club, Rank::Ten),
            Card::suited(0, Suit::Spade, Rank::Ten),
            Card::big_joker(1),
        ];
        game.trump = Some(Trump::new(Rank::Ten, trump_suit).unwrap());
        game.dealer = Some(PlayerId(0));
        game.phase = Phase::Burying;
        (game, buried)
    }

    #[test]
    fn teammates_can_cross_each_other_once_with_both_transfers_applied_simultaneously() {
        let (mut game, buried) = five_trump_crossing_game(Some(Suit::Heart));
        game.bury(PlayerId(0), &buried).unwrap();
        assert_eq!(game.phase(), &Phase::FiveTrumpCrossing);
        let state = game.five_trump_crossing().unwrap();
        assert!(state.eligible(PlayerId(0)));
        assert!(state.eligible(PlayerId(2)));
        assert!(!state.eligible(PlayerId(1)));
        assert!(!state.eligible(PlayerId(3)));
        assert_eq!(
            game.choose_five_trump_crossing(PlayerId(1), None),
            Err(GameError::CrossingNotEligible)
        );

        let initial_zero = game.players[0].hand.clone();
        let initial_two = game.players[2].hand.clone();
        let zero_out = vec![
            Card::suited(0, Suit::Heart, Rank::Three),
            Card::suited(0, Suit::Heart, Rank::Four),
            Card::big_joker(0),
            Card::suited(0, Suit::Club, Rank::Three),
            Card::suited(0, Suit::Club, Rank::Four),
        ];
        let invalid_zero = vec![
            Card::suited(0, Suit::Heart, Rank::Three),
            Card::suited(0, Suit::Heart, Rank::Four),
            Card::suited(0, Suit::Club, Rank::Three),
            Card::suited(0, Suit::Club, Rank::Four),
            Card::suited(0, Suit::Club, Rank::Five),
        ];
        assert_eq!(
            game.choose_five_trump_crossing(PlayerId(0), Some(&invalid_zero)),
            Err(GameError::CrossingMustIncludeAllTrumps)
        );
        game.choose_five_trump_crossing(PlayerId(0), Some(&zero_out))
            .unwrap();
        assert_eq!(game.players[0].hand, initial_zero, "决定阶段不能提前移动牌");

        let two_out = vec![
            Card::suited(0, Suit::Heart, Rank::Five),
            Card::small_joker(0),
            Card::suited(0, Suit::Spade, Rank::Three),
            Card::suited(0, Suit::Spade, Rank::Four),
            Card::suited(0, Suit::Spade, Rank::Five),
        ];
        game.choose_five_trump_crossing(PlayerId(2), Some(&two_out))
            .unwrap();
        assert_eq!(
            game.five_trump_crossing().unwrap().stage(),
            FiveTrumpCrossingStage::Returning
        );
        assert_eq!(game.players[0].hand.len(), initial_zero.len());
        assert_eq!(game.players[2].hand.len(), initial_two.len());

        let after_outgoing_zero = game.players[0].hand.clone();
        game.return_five_trump_crossing(PlayerId(0), &two_out)
            .unwrap();
        assert_eq!(
            game.players[0].hand, after_outgoing_zero,
            "回牌也必须等所有人选定后再同时移动"
        );
        game.return_five_trump_crossing(PlayerId(2), &zero_out)
            .unwrap();
        assert_eq!(game.phase(), &Phase::Playing);

        let mut final_zero = game.players[0].hand.clone();
        let mut final_two = game.players[2].hand.clone();
        let mut initial_zero = initial_zero;
        let mut initial_two = initial_two;
        for hand in [
            &mut final_zero,
            &mut final_two,
            &mut initial_zero,
            &mut initial_two,
        ] {
            hand.sort_by(Card::identity_cmp);
        }
        assert_eq!(final_zero, initial_zero);
        assert_eq!(final_two, initial_two);
        assert_eq!(
            game.choose_five_trump_crossing(PlayerId(0), None),
            Err(GameError::WrongPhase),
            "一局只能进行一次五主过江"
        );
    }

    #[test]
    fn five_trump_crossing_is_skipped_in_no_trump_and_all_declines_start_play() {
        let (mut no_trump, buried) = five_trump_crossing_game(None);
        no_trump.bury(PlayerId(0), &buried).unwrap();
        assert_eq!(no_trump.phase(), &Phase::Playing);
        assert!(no_trump.five_trump_crossing().is_none());

        let (mut suited, buried) = five_trump_crossing_game(Some(Suit::Heart));
        suited.bury(PlayerId(0), &buried).unwrap();
        suited
            .choose_five_trump_crossing(PlayerId(0), None)
            .unwrap();
        suited
            .choose_five_trump_crossing(PlayerId(2), None)
            .unwrap();
        assert_eq!(suited.phase(), &Phase::Playing);
        assert_eq!(suited.current_player(), suited.dealer());
    }
}
