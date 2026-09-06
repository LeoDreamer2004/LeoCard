#[path = "game_crossing.rs"]
mod crossing;
#[path = "game_play.rs"]
mod play;
#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;

#[cfg(test)]
use crate::build_deck;
use crate::play::compare_for_trick;
use crate::{
    BidError, BidState, FollowError, RuleError, ShengjiCard, ShengjiClassifiedPlay,
    ShengjiPlayerId, ShengjiRank, ShengjiRuleSet, ShengjiTeamId, ShengjiTrump, TrickPlay,
    build_deck_for, classify_lead, level_after, validate_follow,
};
use std::collections::{HashSet, VecDeque};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamProgress {
    pub levels: [ShengjiRank; 2],
}

impl Default for TeamProgress {
    fn default() -> Self {
        Self {
            levels: [ShengjiRank::Two, ShengjiRank::Two],
        }
    }
}

impl TeamProgress {
    pub const fn for_rules(rules: &ShengjiRuleSet) -> Self {
        let starting_level = if rules.constant_trump {
            ShengjiRank::Three
        } else {
            ShengjiRank::Two
        };
        Self {
            levels: [starting_level, starting_level],
        }
    }

    pub fn level(&self, team: ShengjiTeamId) -> ShengjiRank {
        self.levels[usize::from(team.0)]
    }

    fn promote(&mut self, team: ShengjiTeamId, steps: u8, mandatory: bool) {
        let slot = &mut self.levels[usize::from(team.0)];
        *slot = level_after(*slot, steps, mandatory);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    pub id: ShengjiPlayerId,
    pub hand: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrickRecord {
    pub leader: ShengjiPlayerId,
    pub plays: Vec<(ShengjiPlayerId, ShengjiClassifiedPlay)>,
    pub winner: ShengjiPlayerId,
    pub points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandResult {
    pub dealer: ShengjiPlayerId,
    pub dealer_team: ShengjiTeamId,
    pub collecting_team: ShengjiTeamId,
    pub trick_points: u32,
    pub penalty_adjustment: i32,
    pub kitty_points: u16,
    pub kitty_multiplier: u32,
    pub collecting_score: u32,
    pub promoted_team: ShengjiTeamId,
    pub promoted_steps: u8,
    pub next_dealer: ShengjiPlayerId,
    pub levels: [ShengjiRank; 2],
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
    pub player: ShengjiPlayerId,
    pub cards: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottomFlipReveal {
    pub card: ShengjiCard,
    pub matches: Vec<BottomFlipMatch>,
    pub dealer: Option<ShengjiPlayerId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottomCopyState {
    next_player: ShengjiPlayerId,
    remaining_without_copy: u8,
    current: Option<ShengjiPlayerId>,
    bottom_holder: Option<ShengjiPlayerId>,
    last_burier: ShengjiPlayerId,
}

impl BottomCopyState {
    pub const fn current(&self) -> Option<ShengjiPlayerId> {
        self.current
    }

    pub const fn bottom_holder(&self) -> Option<ShengjiPlayerId> {
        self.bottom_holder
    }
}

/// 五主过江只在埋底后的固定窗口执行一次。先同时收集所有过江决定并统一
/// 交牌，再同时收集所有回牌并统一归还，避免行动顺序改变资格或可选手牌。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FiveTrumpCrossingState {
    stage: FiveTrumpCrossingStage,
    eligible: [bool; ShengjiRuleSet::PLAYER_COUNT],
    declined: [bool; ShengjiRuleSet::PLAYER_COUNT],
    outgoing: [Option<Vec<ShengjiCard>>; ShengjiRuleSet::PLAYER_COUNT],
    returned: [Option<Vec<ShengjiCard>>; ShengjiRuleSet::PLAYER_COUNT],
}

impl FiveTrumpCrossingState {
    pub const fn stage(&self) -> FiveTrumpCrossingStage {
        self.stage
    }

    pub fn eligible(&self, player: ShengjiPlayerId) -> bool {
        self.eligible
            .get(usize::from(player.0))
            .copied()
            .unwrap_or(false)
    }

    pub fn decision_made(&self, player: ShengjiPlayerId) -> bool {
        let index = usize::from(player.0);
        self.eligible.get(index).copied().unwrap_or(false)
            && (self.declined.get(index).copied().unwrap_or(false)
                || self.outgoing.get(index).is_some_and(Option::is_some))
    }

    pub fn crossing(&self, player: ShengjiPlayerId) -> bool {
        self.outgoing
            .get(usize::from(player.0))
            .is_some_and(Option::is_some)
    }

    /// 该玩家是否需要给发起过江的对家回五张牌。
    pub fn return_required(&self, player: ShengjiPlayerId) -> bool {
        self.crossing(partner(player))
    }

    pub fn return_made(&self, player: ShengjiPlayerId) -> bool {
        self.returned
            .get(usize::from(player.0))
            .is_some_and(Option::is_some)
    }

    pub fn pending_players(&self) -> Vec<ShengjiPlayerId> {
        (0..ShengjiRuleSet::PLAYER_COUNT as u8)
            .map(ShengjiPlayerId)
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
    leader: ShengjiPlayerId,
    lead: ShengjiClassifiedPlay,
    plays: Vec<(ShengjiPlayerId, ShengjiClassifiedPlay)>,
    winner_index: usize,
    points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    CardDealt {
        player: ShengjiPlayerId,
        card: ShengjiCard,
    },
    DealComplete,
    DeclarationChanged,
    DealerTookKitty {
        dealer: ShengjiPlayerId,
    },
    PowerOutageDealerChanged {
        dealer: ShengjiPlayerId,
    },
    BottomFlipStarted,
    BottomCardRevealed(BottomFlipReveal),
    BottomCopyDecision {
        player: ShengjiPlayerId,
        copied: bool,
        inquiry_complete: bool,
    },
    BottomCopyBuryComplete {
        player: ShengjiPlayerId,
        inquiry_complete: bool,
    },
    BuryComplete {
        leader: ShengjiPlayerId,
    },
    FiveTrumpCrossingDecision {
        player: ShengjiPlayerId,
        crossing: bool,
        decisions_complete: bool,
    },
    FiveTrumpCrossingReturn {
        player: ShengjiPlayerId,
        crossing_complete: bool,
    },
    Played {
        player: ShengjiPlayerId,
        next: ShengjiPlayerId,
    },
    ThrowFailed {
        player: ShengjiPlayerId,
        attempted: Vec<ShengjiCard>,
        forced: ShengjiClassifiedPlay,
        penalty_points: u16,
        next: ShengjiPlayerId,
    },
    TrickComplete(TrickRecord),
    HandComplete(HandResult),
}

#[derive(Clone, Debug)]
pub struct GameState {
    rules: ShengjiRuleSet,
    teams: TeamProgress,
    fixed_dealer: Option<ShengjiPlayerId>,
    first_recipient: ShengjiPlayerId,
    bidding_dealer: ShengjiPlayerId,
    power_outage_used: bool,
    bottom_flip_index: usize,
    bottom_flip_winner: Option<(ShengjiPlayerId, ShengjiTrump)>,
    dealer_from_bottom_flip: bool,
    deck: VecDeque<ShengjiCard>,
    players: Vec<PlayerState>,
    bidding: BidState,
    trump: Option<ShengjiTrump>,
    dealer: Option<ShengjiPlayerId>,
    kitty: Vec<ShengjiCard>,
    buried: Vec<ShengjiCard>,
    /// 当前这批底牌的实际埋底者。普通流程为庄家；每次成功
    /// 抄底取走旧底后暂时为空，待抄底者重埋后转移给该玩家。
    bottom_burier: Option<ShengjiPlayerId>,
    bottom_copy: Option<BottomCopyState>,
    five_trump_crossing: Option<FiveTrumpCrossingState>,
    phase: Phase,
    current_player: Option<ShengjiPlayerId>,
    trick: Option<CurrentTrick>,
    history: Vec<TrickRecord>,
    collecting_score: u32,
    trick_points: u32,
    penalty_adjustment: i32,
}

impl GameState {
    pub fn new(
        rules: ShengjiRuleSet,
        teams: TeamProgress,
        fixed_dealer: Option<ShengjiPlayerId>,
        first_recipient: ShengjiPlayerId,
        shuffled_deck: Vec<ShengjiCard>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        if first_recipient.0 >= ShengjiRuleSet::PLAYER_COUNT as u8 {
            return Err(GameError::InvalidPlayer(first_recipient));
        }
        if fixed_dealer.is_some_and(|dealer| dealer.0 >= ShengjiRuleSet::PLAYER_COUNT as u8) {
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
            players: (0..ShengjiRuleSet::PLAYER_COUNT)
                .map(|id| PlayerState {
                    id: ShengjiPlayerId(id as u8),
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

    pub fn standard(shuffled_deck: Vec<ShengjiCard>) -> Result<Self, GameError> {
        Self::new(
            ShengjiRuleSet::default(),
            TeamProgress::default(),
            None,
            ShengjiPlayerId(0),
            shuffled_deck,
        )
    }

    pub const fn rules(&self) -> &ShengjiRuleSet {
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
    pub const fn bidding_dealer(&self) -> ShengjiPlayerId {
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

    pub const fn trump(&self) -> Option<ShengjiTrump> {
        self.trump
    }

    pub const fn dealer(&self) -> Option<ShengjiPlayerId> {
        self.dealer
    }

    pub const fn current_player(&self) -> Option<ShengjiPlayerId> {
        self.current_player
    }

    pub fn buried(&self) -> &[ShengjiCard] {
        &self.buried
    }

    pub const fn bottom_burier(&self) -> Option<ShengjiPlayerId> {
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
        if dealt == ShengjiRuleSet::PLAYER_COUNT * self.rules.hand_size() {
            self.phase = Phase::BiddingGrace;
            return Ok(ActionOutcome::DealComplete);
        }
        let player = ShengjiPlayerId(
            (usize::from(self.first_recipient.0) + dealt) as u8
                % ShengjiRuleSet::PLAYER_COUNT as u8,
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
        player: ShengjiPlayerId,
        cards: &[ShengjiCard],
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
            let trump = ShengjiTrump::new(self.teams.level(dealer.team()), card.suit())
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

    fn take_kitty_for_dealer(&mut self, dealer: ShengjiPlayerId, trump: ShengjiTrump) {
        self.kitty = self.deck.drain(..).collect();
        debug_assert_eq!(self.kitty.len(), self.rules.kitty_size());
        self.players[usize::from(dealer.0)]
            .hand
            .extend(self.kitty.iter().copied());
        self.trump = Some(trump);
        self.dealer = Some(dealer);
        self.phase = Phase::Burying;
    }

    pub fn bury(
        &mut self,
        player: ShengjiPlayerId,
        cards: &[ShengjiCard],
    ) -> Result<ActionOutcome, GameError> {
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
        player: ShengjiPlayerId,
        cards: Option<&[ShengjiCard]>,
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
                ShengjiTrump::new(self.bidding.level(), declaration.trump.trump_suit())
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
            state.remaining_without_copy = ShengjiRuleSet::PLAYER_COUNT as u8;
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
            remaining_without_copy: ShengjiRuleSet::PLAYER_COUNT as u8,
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
                    declined: [false; ShengjiRuleSet::PLAYER_COUNT],
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

    fn start_playing(&mut self) {
        self.phase = Phase::Playing;
        self.current_player = self.dealer;
    }

    fn player(&self, player: ShengjiPlayerId) -> Result<&PlayerState, GameError> {
        self.players
            .get(usize::from(player.0))
            .ok_or(GameError::InvalidPlayer(player))
    }
}

fn select_bottom_flip_dealer(
    matches: &[BottomFlipMatch],
    current_dealer: ShengjiPlayerId,
) -> Option<ShengjiPlayerId> {
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
            (matched.player.0 + ShengjiRuleSet::PLAYER_COUNT as u8 - current_dealer.0)
                % ShengjiRuleSet::PLAYER_COUNT as u8
        })
        .map(|matched| matched.player)
}

fn next_player(player: ShengjiPlayerId) -> ShengjiPlayerId {
    ShengjiPlayerId((player.0 + 1) % ShengjiRuleSet::PLAYER_COUNT as u8)
}

const fn partner(player: ShengjiPlayerId) -> ShengjiPlayerId {
    ShengjiPlayerId((player.0 + 2) % ShengjiRuleSet::PLAYER_COUNT as u8)
}

fn remove_cards(hand: &mut Vec<ShengjiCard>, cards: &[ShengjiCard]) {
    for card in cards {
        let index = hand
            .iter()
            .position(|candidate| candidate == card)
            .expect("ownership validated before removal");
        hand.remove(index);
    }
}

fn validate_cards_owned(cards: &[ShengjiCard], hand: &[ShengjiCard]) -> Result<(), GameError> {
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() || cards.iter().any(|card| !hand.contains(card)) {
        return Err(GameError::CardsNotOwned);
    }
    Ok(())
}

fn validate_deck(deck: &[ShengjiCard], rules: ShengjiRuleSet) -> Result<(), GameError> {
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
    InvalidPlayer(ShengjiPlayerId),
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
        expected: ShengjiPlayerId,
        actual: ShengjiPlayerId,
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
