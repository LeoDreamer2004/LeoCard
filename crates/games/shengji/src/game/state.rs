use super::{HandStatisticsTracker, ShengjiGameError, ShengjiRedealReason, partner, validate_deck};
use crate::{
    ShengjiBidState, ShengjiCard, ShengjiClassifiedPlay, ShengjiRank, ShengjiRuleSet,
    ShengjiTeamId, ShengjiTrump, level_after,
};
use leocard_game_common::PlayerId;

use std::collections::VecDeque;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShengjiTeamProgress {
    pub levels: [ShengjiRank; 2],
}

impl Default for ShengjiTeamProgress {
    fn default() -> Self {
        Self {
            levels: [ShengjiRank::Two, ShengjiRank::Two],
        }
    }
}

impl ShengjiTeamProgress {
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
pub struct ShengjiPlayerState {
    pub id: PlayerId,
    pub hand: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShengjiTrickRecord {
    pub leader: PlayerId,
    pub plays: Vec<(PlayerId, ShengjiClassifiedPlay)>,
    pub winner: PlayerId,
    pub points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShengjiHandResult {
    pub dealer: PlayerId,
    pub dealer_team: ShengjiTeamId,
    pub collecting_team: ShengjiTeamId,
    pub trick_points: u32,
    pub penalty_adjustment: i32,
    pub kitty_points: u16,
    pub kitty_multiplier: u32,
    pub collecting_score: u32,
    pub promoted_team: ShengjiTeamId,
    pub promoted_steps: u8,
    pub next_dealer: PlayerId,
    pub levels: [ShengjiRank; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShengjiPhase {
    Dealing,
    BiddingGrace,
    BottomFlipping,
    Burying,
    BottomCopying,
    BottomCopyBurying,
    FiveTrumpCrossing,
    Playing,
    Finished(ShengjiHandResult),
    RedealRequired(ShengjiRedealReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ShengjiFiveTrumpCrossingStage {
    Deciding,
    Returning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ShengjiBottomFlipMatch {
    pub player: PlayerId,
    pub cards: Vec<ShengjiCard>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ShengjiBottomFlipReveal {
    pub card: ShengjiCard,
    pub matches: Vec<ShengjiBottomFlipMatch>,
    pub dealer: Option<PlayerId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShengjiBottomCopyState {
    pub(super) next_player: PlayerId,
    pub(super) remaining_without_copy: u8,
    pub(super) current: Option<PlayerId>,
    pub(super) bottom_holder: Option<PlayerId>,
    pub(super) last_burier: PlayerId,
}

impl ShengjiBottomCopyState {
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
pub struct ShengjiFiveTrumpCrossingState {
    pub(super) stage: ShengjiFiveTrumpCrossingStage,
    pub(super) eligible: [bool; ShengjiRuleSet::PLAYER_COUNT as usize],
    pub(super) declined: [bool; ShengjiRuleSet::PLAYER_COUNT as usize],
    pub(super) outgoing: [Option<Vec<ShengjiCard>>; ShengjiRuleSet::PLAYER_COUNT as usize],
    pub(super) returned: [Option<Vec<ShengjiCard>>; ShengjiRuleSet::PLAYER_COUNT as usize],
}

impl ShengjiFiveTrumpCrossingState {
    pub const fn stage(&self) -> ShengjiFiveTrumpCrossingStage {
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
        (0..ShengjiRuleSet::PLAYER_COUNT)
            .map(PlayerId)
            .filter(|player| match self.stage {
                ShengjiFiveTrumpCrossingStage::Deciding => {
                    self.eligible(*player) && !self.decision_made(*player)
                }
                ShengjiFiveTrumpCrossingStage::Returning => {
                    self.return_required(*player) && !self.return_made(*player)
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub(super) struct CurrentTrick {
    pub(super) leader: PlayerId,
    pub(super) lead: ShengjiClassifiedPlay,
    pub(super) plays: Vec<(PlayerId, ShengjiClassifiedPlay)>,
    pub(super) winner_index: usize,
    pub(super) points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShengjiActionOutcome {
    CardDealt {
        player: PlayerId,
        card: ShengjiCard,
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
    BottomCardRevealed(ShengjiBottomFlipReveal),
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
        attempted: Vec<ShengjiCard>,
        forced: ShengjiClassifiedPlay,
        penalty_points: u16,
        next: PlayerId,
    },
    TrickComplete(ShengjiTrickRecord),
    HandComplete(ShengjiHandResult),
}

#[derive(Clone, Debug)]
pub struct ShengjiGameState {
    pub(super) statistics: HandStatisticsTracker,
    pub(super) rules: ShengjiRuleSet,
    pub(super) teams: ShengjiTeamProgress,
    pub(super) fixed_dealer: Option<PlayerId>,
    pub(super) first_recipient: PlayerId,
    pub(super) bidding_dealer: PlayerId,
    pub(super) power_outage_used: bool,
    pub(super) bottom_flip_index: usize,
    pub(super) bottom_flip_winner: Option<(PlayerId, ShengjiTrump)>,
    pub(super) dealer_from_bottom_flip: bool,
    pub(super) deck: VecDeque<ShengjiCard>,
    pub(super) players: Vec<ShengjiPlayerState>,
    pub(super) bidding: ShengjiBidState,
    pub(super) trump: Option<ShengjiTrump>,
    pub(super) dealer: Option<PlayerId>,
    pub(super) kitty: Vec<ShengjiCard>,
    pub(super) buried: Vec<ShengjiCard>,
    /// 当前这批底牌的实际埋底者。普通流程为庄家；每次成功
    /// 抄底取走旧底后暂时为空，待抄底者重埋后转移给该玩家。
    pub(super) bottom_burier: Option<PlayerId>,
    pub(super) bottom_copy: Option<ShengjiBottomCopyState>,
    pub(super) five_trump_crossing: Option<ShengjiFiveTrumpCrossingState>,
    pub(super) phase: ShengjiPhase,
    pub(super) current_player: Option<PlayerId>,
    pub(super) trick: Option<CurrentTrick>,
    pub(super) history: Vec<ShengjiTrickRecord>,
    pub(super) collecting_score: u32,
    pub(super) trick_points: u32,
    pub(super) penalty_adjustment: i32,
}

impl ShengjiGameState {
    pub fn new(
        rules: ShengjiRuleSet,
        teams: ShengjiTeamProgress,
        fixed_dealer: Option<PlayerId>,
        first_recipient: PlayerId,
        shuffled_deck: Vec<ShengjiCard>,
    ) -> Result<Self, ShengjiGameError> {
        let rules = rules.validate()?;
        if first_recipient.0 >= rules.player_count() {
            return Err(ShengjiGameError::InvalidPlayer(first_recipient));
        }
        if fixed_dealer.is_some_and(|dealer| dealer.0 >= rules.player_count()) {
            return Err(ShengjiGameError::InvalidPlayer(fixed_dealer.unwrap()));
        }
        validate_deck(&shuffled_deck, rules)?;
        let first_hand = fixed_dealer.is_none();
        let teams = if first_hand {
            ShengjiTeamProgress::for_rules(&rules)
        } else {
            teams
        };
        let provisional_dealer = fixed_dealer.unwrap_or(first_recipient);
        let level = teams.level(ShengjiTeamId::of(provisional_dealer));
        let hand_size = rules.hand_size();
        let kitty_size = rules.kitty_size();
        Ok(Self {
            statistics: HandStatisticsTracker::default(),
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
            players: (0..rules.player_count())
                .map(|id| ShengjiPlayerState {
                    id: PlayerId(id),
                    hand: Vec::with_capacity(hand_size + kitty_size),
                })
                .collect(),
            bidding: ShengjiBidState::new_with_rules(level, rules.deck_count, rules.bid_with_joker)
                .with_player_count(rules.player_count()),
            trump: None,
            // 后续小局的庄家已由上一局结算确定，发牌开始就应公开；首局则
            // 等第一位玩家亮主后再随当前最高声明实时转移。
            dealer: fixed_dealer,
            kitty: Vec::new(),
            buried: Vec::new(),
            bottom_burier: None,
            bottom_copy: None,
            five_trump_crossing: None,
            phase: ShengjiPhase::Dealing,
            current_player: None,
            trick: None,
            history: Vec::with_capacity(hand_size),
            collecting_score: 0,
            trick_points: 0,
            penalty_adjustment: 0,
        })
    }

    pub fn standard(shuffled_deck: Vec<ShengjiCard>) -> Result<Self, ShengjiGameError> {
        Self::new(
            ShengjiRuleSet::default(),
            ShengjiTeamProgress::default(),
            None,
            PlayerId(0),
            shuffled_deck,
        )
    }

    pub const fn rules(&self) -> &ShengjiRuleSet {
        &self.rules
    }

    pub const fn phase(&self) -> &ShengjiPhase {
        &self.phase
    }

    pub fn players(&self) -> &[ShengjiPlayerState] {
        &self.players
    }

    pub const fn bidding(&self) -> &ShengjiBidState {
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

    pub const fn five_trump_crossing(&self) -> Option<&ShengjiFiveTrumpCrossingState> {
        self.five_trump_crossing.as_ref()
    }

    pub const fn bottom_copy(&self) -> Option<&ShengjiBottomCopyState> {
        self.bottom_copy.as_ref()
    }

    pub const fn trump(&self) -> Option<ShengjiTrump> {
        self.trump
    }

    pub const fn dealer(&self) -> Option<PlayerId> {
        self.dealer
    }

    /// The established dealer, or `None` while the opening hand is contested.
    pub const fn fixed_dealer(&self) -> Option<PlayerId> {
        self.fixed_dealer
    }

    pub const fn current_player(&self) -> Option<PlayerId> {
        self.current_player
    }

    pub fn buried(&self) -> &[ShengjiCard] {
        &self.buried
    }

    pub const fn bottom_burier(&self) -> Option<PlayerId> {
        self.bottom_burier
    }

    pub fn history(&self) -> &[ShengjiTrickRecord] {
        &self.history
    }

    /// 当前一圈的公开状态。返回副本以免向宿主暴露内部胜者索引。
    pub fn current_trick(&self) -> Option<ShengjiTrickRecord> {
        let trick = self.trick.as_ref()?;
        Some(ShengjiTrickRecord {
            leader: trick.leader,
            plays: trick.plays.clone(),
            winner: trick.plays[trick.winner_index].0,
            points: trick.points,
        })
    }

    pub const fn teams(&self) -> &ShengjiTeamProgress {
        &self.teams
    }

    pub const fn collecting_score(&self) -> u32 {
        self.collecting_score
    }

    /// 埋底、抄底与五主过江全部完成后，固定本盘完整起手统计并开始出牌。
    pub(super) fn start_playing(&mut self) {
        self.record_opening_hands();
        self.phase = ShengjiPhase::Playing;
        self.current_player = self.dealer;
    }

    pub(super) fn player(&self, player: PlayerId) -> Result<&ShengjiPlayerState, ShengjiGameError> {
        self.players
            .get(usize::from(player.0))
            .ok_or(ShengjiGameError::InvalidPlayer(player))
    }
}
