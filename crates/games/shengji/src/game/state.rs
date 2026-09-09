use super::{GameError, partner, validate_deck};
use crate::{
    BidState, ShengjiCard, ShengjiClassifiedPlay, ShengjiPlayerId, ShengjiRank, ShengjiRuleSet,
    ShengjiTeamId, ShengjiTrump, level_after,
};
use std::collections::VecDeque;

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

    pub(super) fn promote(&mut self, team: ShengjiTeamId, steps: u8, mandatory: bool) {
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
    pub(super) next_player: ShengjiPlayerId,
    pub(super) remaining_without_copy: u8,
    pub(super) current: Option<ShengjiPlayerId>,
    pub(super) bottom_holder: Option<ShengjiPlayerId>,
    pub(super) last_burier: ShengjiPlayerId,
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
    pub(super) stage: FiveTrumpCrossingStage,
    pub(super) eligible: [bool; ShengjiRuleSet::PLAYER_COUNT],
    pub(super) declined: [bool; ShengjiRuleSet::PLAYER_COUNT],
    pub(super) outgoing: [Option<Vec<ShengjiCard>>; ShengjiRuleSet::PLAYER_COUNT],
    pub(super) returned: [Option<Vec<ShengjiCard>>; ShengjiRuleSet::PLAYER_COUNT],
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
pub(super) struct CurrentTrick {
    pub(super) leader: ShengjiPlayerId,
    pub(super) lead: ShengjiClassifiedPlay,
    pub(super) plays: Vec<(ShengjiPlayerId, ShengjiClassifiedPlay)>,
    pub(super) winner_index: usize,
    pub(super) points: u16,
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
    pub(super) rules: ShengjiRuleSet,
    pub(super) teams: TeamProgress,
    pub(super) fixed_dealer: Option<ShengjiPlayerId>,
    pub(super) first_recipient: ShengjiPlayerId,
    pub(super) bidding_dealer: ShengjiPlayerId,
    pub(super) power_outage_used: bool,
    pub(super) bottom_flip_index: usize,
    pub(super) bottom_flip_winner: Option<(ShengjiPlayerId, ShengjiTrump)>,
    pub(super) dealer_from_bottom_flip: bool,
    pub(super) deck: VecDeque<ShengjiCard>,
    pub(super) players: Vec<PlayerState>,
    pub(super) bidding: BidState,
    pub(super) trump: Option<ShengjiTrump>,
    pub(super) dealer: Option<ShengjiPlayerId>,
    pub(super) kitty: Vec<ShengjiCard>,
    pub(super) buried: Vec<ShengjiCard>,
    /// 当前这批底牌的实际埋底者。普通流程为庄家；每次成功
    /// 抄底取走旧底后暂时为空，待抄底者重埋后转移给该玩家。
    pub(super) bottom_burier: Option<ShengjiPlayerId>,
    pub(super) bottom_copy: Option<BottomCopyState>,
    pub(super) five_trump_crossing: Option<FiveTrumpCrossingState>,
    pub(super) phase: Phase,
    pub(super) current_player: Option<ShengjiPlayerId>,
    pub(super) trick: Option<CurrentTrick>,
    pub(super) history: Vec<TrickRecord>,
    pub(super) collecting_score: u32,
    pub(super) trick_points: u32,
    pub(super) penalty_adjustment: i32,
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

    /// 宿主在发牌结束后的五秒亮主窗口结束时调用。
    pub(super) fn start_playing(&mut self) {
        self.phase = Phase::Playing;
        self.current_player = self.dealer;
    }

    pub(super) fn player(&self, player: ShengjiPlayerId) -> Result<&PlayerState, GameError> {
        self.players
            .get(usize::from(player.0))
            .ok_or(GameError::InvalidPlayer(player))
    }
}
