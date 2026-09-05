use std::array;
use std::collections::{HashSet, VecDeque};
use std::fmt;

use crate::{
    MatchLength, Meld, MeldKind, PlayerId, RuleError, RuleSet, ScoreError, ScoreInput, ScoreResult,
    Tile, TileKind, WinContext, WinSource, Wind, build_deck, is_complete_hand, score_hand,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DrawOrigin {
    Normal,
    KongReplacement,
    FlowerReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerState {
    id: PlayerId,
    hand: Vec<Tile>,
    melds: Vec<Meld>,
    flowers: Vec<Tile>,
    dead_hand: bool,
    hand_revealed: bool,
}

impl PlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[Tile] {
        &self.hand
    }

    pub fn melds(&self) -> &[Meld] {
        &self.melds
    }

    pub fn flowers(&self) -> &[Tile] {
        &self.flowers
    }

    pub const fn is_dead_hand(&self) -> bool {
        self.dead_hand
    }

    pub const fn is_hand_revealed(&self) -> bool {
        self.hand_revealed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicPlayerState {
    pub id: PlayerId,
    pub concealed_count: usize,
    pub revealed_hand: Option<Vec<Tile>>,
    pub melds: Vec<PublicMeld>,
    pub flowers: Vec<Tile>,
    pub dead_hand: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicMeld {
    pub kind: MeldKind,
    /// 他人的暗杠在本盘结算前只公开为四张牌背。
    pub tile: Option<TileKind>,
    pub claimed_from: Option<PlayerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Discard {
    pub player: PlayerId,
    pub tile: Tile,
    pub claimed_by: Option<PlayerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClaimOption {
    Chow { start: u8 },
    Pung,
    Kong,
    Win,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Claim {
    Pass,
    Chow { start: u8 },
    Pung,
    Kong,
    Win,
}

impl Claim {
    const fn option(self) -> Option<ClaimOption> {
        match self {
            Self::Pass => None,
            Self::Chow { start } => Some(ClaimOption::Chow { start }),
            Self::Pung => Some(ClaimOption::Pung),
            Self::Kong => Some(ClaimOption::Kong),
            Self::Win => Some(ClaimOption::Win),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum ClaimTrigger {
    Discard {
        discard_index: usize,
        from: PlayerId,
        tile: Tile,
    },
    AddedKong {
        player: PlayerId,
        tile: Tile,
        meld_index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PendingClaim {
    trigger: ClaimTrigger,
    options: [Vec<ClaimOption>; RuleSet::PLAYER_COUNT],
    responses: [Option<Claim>; RuleSet::PLAYER_COUNT],
}

impl PendingClaim {
    pub const fn source_player(&self) -> PlayerId {
        match self.trigger {
            ClaimTrigger::Discard { from, .. } => from,
            ClaimTrigger::AddedKong { player, .. } => player,
        }
    }

    pub const fn tile(&self) -> Tile {
        match self.trigger {
            ClaimTrigger::Discard { tile, .. } | ClaimTrigger::AddedKong { tile, .. } => tile,
        }
    }

    pub const fn is_robbing_kong_window(&self) -> bool {
        matches!(self.trigger, ClaimTrigger::AddedKong { .. })
    }

    pub fn options_for(&self, player: PlayerId) -> Option<&[ClaimOption]> {
        self.options.get(player.0).map(Vec::as_slice)
    }

    pub fn response_from(&self, player: PlayerId) -> Option<Claim> {
        self.responses.get(player.0).copied().flatten()
    }

    pub fn waiting_for(&self) -> Vec<PlayerId> {
        (0..RuleSet::PLAYER_COUNT)
            .filter(|index| !self.options[*index].is_empty() && self.responses[*index].is_none())
            .map(PlayerId)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WinRecord {
    pub player: PlayerId,
    pub from: Option<PlayerId>,
    pub score: ScoreResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandResult {
    pub winners: Vec<WinRecord>,
    pub exhaustive_draw: bool,
    pub deltas: [i32; RuleSet::PLAYER_COUNT],
    pub match_scores: [i32; RuleSet::PLAYER_COUNT],
    pub match_complete: bool,
    pub sequence_index: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Phase {
    Dealing { batch: u8 },
    ReplacingFlower { player: PlayerId },
    Playing,
    WaitingForClaims(PendingClaim),
    Finished(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionOutcome {
    Discarded {
        player: PlayerId,
        tile: Tile,
    },
    ClaimRecorded {
        player: PlayerId,
    },
    Claimed {
        player: PlayerId,
        source: PlayerId,
        tile: Tile,
        claim: Claim,
    },
    Drew {
        player: PlayerId,
        tile: Tile,
        origin: DrawOrigin,
    },
    KongDeclared {
        player: PlayerId,
        tile: TileKind,
        added: bool,
    },
    FalseWin {
        player: PlayerId,
        deltas: [i32; RuleSet::PLAYER_COUNT],
    },
    HandFinished(HandResult),
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
    WrongPhase,
    TileNotInHand(Tile),
    InvalidClaim,
    AlreadyResponded,
    CannotWin,
    CannotKong,
    Score(ScoreError),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌墙张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌墙不是完整且无重复的 144 张麻将牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}，当前应由 {:?} 操作", actual, expected)
            }
            Self::WrongPhase => f.write_str("当前阶段不能执行该操作"),
            Self::TileNotInHand(tile) => {
                write!(f, "手中没有指定牌：{}#{}", tile.kind(), tile.copy())
            }
            Self::InvalidClaim => f.write_str("当前响应窗口不允许该操作"),
            Self::AlreadyResponded => f.write_str("已经提交过本次响应"),
            Self::CannotWin => f.write_str("当前手牌不能宣布和牌"),
            Self::CannotKong => f.write_str("当前不能开杠"),
            Self::Score(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<ScoreError> for GameError {
    fn from(value: ScoreError) -> Self {
        Self::Score(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: RuleSet,
    players: [PlayerState; RuleSet::PLAYER_COUNT],
    wall: VecDeque<Tile>,
    discards: Vec<Discard>,
    initial_dealer: PlayerId,
    dealer: PlayerId,
    prevalent_wind: Wind,
    sequence_index: u8,
    hands_in_match: u8,
    current_player: PlayerId,
    last_drawn: Option<Tile>,
    draw_origin: DrawOrigin,
    hand_deltas: [i32; RuleSet::PLAYER_COUNT],
    match_scores: [i32; RuleSet::PLAYER_COUNT],
    phase: Phase,
}

impl GameState {
    /// `deck[0]` 是牌墙前端第一张牌；补花和杠牌从牌墙尾端取牌。
    pub fn new_with_deck(
        rules: RuleSet,
        deck: Vec<Tile>,
        initial_dealer: PlayerId,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        validate_player(initial_dealer)?;
        validate_deck(&deck)?;
        let players = array::from_fn(|index| PlayerState {
            id: PlayerId(index),
            hand: Vec::new(),
            melds: Vec::new(),
            flowers: Vec::new(),
            dead_hand: false,
            hand_revealed: false,
        });
        let game = Self {
            rules,
            players,
            wall: VecDeque::from(deck),
            discards: Vec::new(),
            initial_dealer,
            dealer: initial_dealer,
            prevalent_wind: Wind::East,
            sequence_index: 0,
            hands_in_match: 0,
            current_player: initial_dealer,
            last_drawn: None,
            draw_origin: DrawOrigin::Normal,
            hand_deltas: [0; RuleSet::PLAYER_COUNT],
            match_scores: [0; RuleSet::PLAYER_COUNT],
            phase: Phase::Dealing { batch: 0 },
        };
        Ok(game)
    }

    pub const fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState; RuleSet::PLAYER_COUNT] {
        &self.players
    }

    pub fn player(&self, player: PlayerId) -> Option<&PlayerState> {
        self.players.get(player.0)
    }

    pub fn public_player(&self, viewer: PlayerId, player: PlayerId) -> Option<PublicPlayerState> {
        validate_player(viewer).ok()?;
        let state = self.players.get(player.0)?;
        let settled = matches!(self.phase, Phase::Finished(_));
        let reveal_hand = viewer == player || state.hand_revealed;
        let reveal_concealed_kong = reveal_hand || settled;
        Some(PublicPlayerState {
            id: player,
            concealed_count: state.hand.len(),
            revealed_hand: reveal_hand.then(|| state.hand.clone()),
            melds: state
                .melds
                .iter()
                .map(|meld| PublicMeld {
                    kind: meld.kind(),
                    tile: (!matches!(meld.kind(), MeldKind::Kong(crate::KongKind::Concealed))
                        || reveal_concealed_kong)
                        .then_some(meld.tile()),
                    claimed_from: meld.claimed_from(),
                })
                .collect(),
            flowers: state.flowers.clone(),
            dead_hand: state.dead_hand,
        })
    }

    pub fn discards(&self) -> &[Discard] {
        &self.discards
    }

    pub const fn dealer(&self) -> PlayerId {
        self.dealer
    }

    pub const fn prevalent_wind(&self) -> Wind {
        self.prevalent_wind
    }

    pub const fn sequence_index(&self) -> u8 {
        self.sequence_index
    }

    pub const fn current_player(&self) -> PlayerId {
        self.current_player
    }

    pub const fn last_drawn(&self) -> Option<Tile> {
        self.last_drawn
    }

    pub fn wall_len(&self) -> usize {
        self.wall.len()
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub const fn match_scores(&self) -> &[i32; RuleSet::PLAYER_COUNT] {
        &self.match_scores
    }

    pub fn seat_wind(&self, player: PlayerId) -> Option<Wind> {
        validate_player(player).ok()?;
        let distance = (player.0 + RuleSet::PLAYER_COUNT - self.dealer.0) % RuleSet::PLAYER_COUNT;
        Some(Wind::ALL[distance])
    }

    pub fn discard(&mut self, player: PlayerId, tile: Tile) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        let position = self.players[player.0]
            .hand
            .iter()
            .position(|held| *held == tile)
            .ok_or(GameError::TileNotInHand(tile))?;
        self.players[player.0].hand.remove(position);
        self.last_drawn = None;
        let discard_index = self.discards.len();
        self.discards.push(Discard {
            player,
            tile,
            claimed_by: None,
        });
        let pending = self.pending_for_discard(discard_index, player, tile)?;
        if pending.waiting_for().is_empty() {
            return self.advance_after_unclaimed_discard(player);
        }
        self.phase = Phase::WaitingForClaims(pending);
        Ok(ActionOutcome::Discarded { player, tile })
    }

    pub fn respond_to_claim(
        &mut self,
        player: PlayerId,
        claim: Claim,
    ) -> Result<ActionOutcome, GameError> {
        validate_player(player)?;
        let Phase::WaitingForClaims(pending) = &mut self.phase else {
            return Err(GameError::WrongPhase);
        };
        if pending.options[player.0].is_empty() {
            return Err(GameError::InvalidClaim);
        }
        if pending.responses[player.0].is_some() {
            return Err(GameError::AlreadyResponded);
        }
        if let Some(option) = claim.option()
            && !pending.options[player.0].contains(&option)
        {
            return Err(GameError::InvalidClaim);
        }
        pending.responses[player.0] = Some(claim);
        if !pending.waiting_for().is_empty() {
            return Ok(ActionOutcome::ClaimRecorded { player });
        }
        self.resolve_pending_claim()
    }

    pub fn declare_self_draw(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        if self.players[player.0].dead_hand {
            return Err(GameError::CannotWin);
        }
        let winning = self.last_drawn.ok_or(GameError::CannotWin)?;
        let source = match self.draw_origin {
            DrawOrigin::Normal => WinSource::SelfDraw,
            DrawOrigin::KongReplacement => WinSource::KongReplacement,
            DrawOrigin::FlowerReplacement => WinSource::FlowerReplacement,
        };
        let score = self.score_for(player, winning.kind(), source)?;
        if self.is_legal_score(&score) {
            return self.finish_with_winners(vec![WinRecord {
                player,
                from: None,
                score,
            }]);
        }
        if self.rules.false_win {
            let penalty = self.apply_false_win(player);
            return Ok(ActionOutcome::FalseWin {
                player,
                deltas: penalty,
            });
        }
        Err(GameError::CannotWin)
    }

    pub fn self_draw_available(&self, player: PlayerId) -> Result<bool, GameError> {
        Ok(self
            .self_draw_score(player)?
            .is_some_and(|score| self.is_legal_score(&score) || self.rules.false_win))
    }

    /// 供自动玩家判断真正合法的自摸，避免“允许错和”开启时主动报错和。
    pub fn legal_self_draw_available(&self, player: PlayerId) -> Result<bool, GameError> {
        Ok(self
            .self_draw_score(player)?
            .is_some_and(|score| self.is_legal_score(&score)))
    }

    /// 判断当前响应窗口中的和牌是否真正达到起和要求。
    pub fn legal_claim_win_available(&self, player: PlayerId) -> Result<bool, GameError> {
        validate_player(player)?;
        let Phase::WaitingForClaims(pending) = &self.phase else {
            return Ok(false);
        };
        if !pending.options[player.0].contains(&ClaimOption::Win) {
            return Ok(false);
        }
        let (tile, source) = match pending.trigger {
            ClaimTrigger::Discard { from, tile, .. } => (tile, WinSource::Discard(from)),
            ClaimTrigger::AddedKong { player, tile, .. } => (tile, WinSource::RobbingKong(player)),
        };
        let score = self.score_for(player, tile.kind(), source)?;
        Ok(self.is_legal_score(&score))
    }

    fn self_draw_score(&self, player: PlayerId) -> Result<Option<ScoreResult>, GameError> {
        validate_player(player)?;
        if !matches!(self.phase, Phase::Playing)
            || self.current_player != player
            || self.players[player.0].dead_hand
        {
            return Ok(None);
        }
        let Some(winning) = self.last_drawn else {
            return Ok(None);
        };
        let concealed: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .map(|tile| tile.kind())
            .collect();
        if !is_complete_hand(&concealed, &self.players[player.0].melds) {
            return Ok(None);
        }
        let source = match self.draw_origin {
            DrawOrigin::Normal => WinSource::SelfDraw,
            DrawOrigin::KongReplacement => WinSource::KongReplacement,
            DrawOrigin::FlowerReplacement => WinSource::FlowerReplacement,
        };
        let score = self.score_for(player, winning.kind(), source)?;
        Ok(Some(score))
    }

    pub fn declare_concealed_kong(
        &mut self,
        player: PlayerId,
        tile: TileKind,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        if self.wall.is_empty() || tile.is_flower() {
            return Err(GameError::CannotKong);
        }
        let positions: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .enumerate()
            .filter(|(_, held)| held.kind() == tile)
            .map(|(index, _)| index)
            .collect();
        if positions.len() != 4 {
            return Err(GameError::CannotKong);
        }
        for position in positions.into_iter().rev() {
            self.players[player.0].hand.remove(position);
        }
        self.players[player.0]
            .melds
            .push(Meld::concealed_kong(tile));
        self.draw_replacement(player, DrawOrigin::KongReplacement)?;
        Ok(ActionOutcome::KongDeclared {
            player,
            tile,
            added: false,
        })
    }

    pub fn declare_added_kong(
        &mut self,
        player: PlayerId,
        tile: Tile,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing_turn(player)?;
        if self.wall.is_empty() {
            return Err(GameError::CannotKong);
        }
        if !self.players[player.0].hand.contains(&tile) {
            return Err(GameError::TileNotInHand(tile));
        }
        let Some(meld_index) = self.players[player.0]
            .melds
            .iter()
            .position(|meld| meld.kind() == MeldKind::Pung && meld.tile() == tile.kind())
        else {
            return Err(GameError::CannotKong);
        };
        let mut pending = PendingClaim {
            trigger: ClaimTrigger::AddedKong {
                player,
                tile,
                meld_index,
            },
            options: array::from_fn(|_| Vec::new()),
            responses: [None; RuleSet::PLAYER_COUNT],
        };
        for target in 0..RuleSet::PLAYER_COUNT {
            let target = PlayerId(target);
            if target != player
                && self.win_button_available(target, tile.kind(), WinSource::RobbingKong(player))?
            {
                pending.options[target.0].push(ClaimOption::Win);
            }
        }
        if pending.waiting_for().is_empty() {
            self.finalize_added_kong(player, tile, meld_index)?;
            return Ok(ActionOutcome::KongDeclared {
                player,
                tile: tile.kind(),
                added: true,
            });
        }
        self.phase = Phase::WaitingForClaims(pending);
        Ok(ActionOutcome::ClaimRecorded { player })
    }

    /// 结算界面内全员准备后开始下一盘。单局模式重置累计分，但继续轮庄和轮圈风。
    pub fn start_next_hand(&mut self, deck: Vec<Tile>) -> Result<(), GameError> {
        let Phase::Finished(result) = &self.phase else {
            return Err(GameError::WrongPhase);
        };
        validate_deck(&deck)?;
        let completed_match = result.match_complete;
        if self.rules.match_length == MatchLength::SingleHand {
            self.match_scores = [0; RuleSet::PLAYER_COUNT];
            self.sequence_index = (self.sequence_index + 1) % 16;
            self.hands_in_match = 0;
        } else if completed_match {
            self.match_scores = [0; RuleSet::PLAYER_COUNT];
            self.sequence_index = 0;
            self.hands_in_match = 0;
        } else {
            self.sequence_index += 1;
        }
        self.dealer = PlayerId(
            (self.initial_dealer.0 + usize::from(self.sequence_index)) % RuleSet::PLAYER_COUNT,
        );
        self.prevalent_wind = Wind::ALL[usize::from(self.sequence_index / 4)];
        self.wall = VecDeque::from(deck);
        self.discards.clear();
        self.hand_deltas = [0; RuleSet::PLAYER_COUNT];
        self.current_player = self.dealer;
        self.last_drawn = None;
        self.draw_origin = DrawOrigin::Normal;
        self.phase = Phase::Dealing { batch: 0 };
        for player in &mut self.players {
            player.hand.clear();
            player.melds.clear();
            player.flowers.clear();
            player.dead_hand = false;
            player.hand_revealed = false;
        }
        Ok(())
    }

    /// 推进一次真实发牌：前三轮依次给一家四张，随后每家一张，最后庄家跳一张。
    /// 返回 `true` 表示本次发完后已经进入出牌阶段。
    pub fn advance_deal(&mut self) -> Result<bool, GameError> {
        let Phase::Dealing { batch } = self.phase else {
            return Err(GameError::WrongPhase);
        };
        let (player, count) = match batch {
            0..=11 => {
                let offset = usize::from(batch % RuleSet::PLAYER_COUNT as u8);
                (
                    PlayerId((self.dealer.0 + offset) % RuleSet::PLAYER_COUNT),
                    4,
                )
            }
            12..=15 => {
                let offset = usize::from(batch - 12);
                (
                    PlayerId((self.dealer.0 + offset) % RuleSet::PLAYER_COUNT),
                    1,
                )
            }
            16 => (self.dealer, 1),
            17 => return self.advance_initial_flower_replacement(),
            _ => return Err(GameError::WrongPhase),
        };
        let mut last = None;
        for _ in 0..count {
            let tile = self
                .wall
                .pop_front()
                .ok_or(GameError::InvalidDeckContents)?;
            self.players[player.0].hand.push(tile);
            last = Some(tile);
        }
        if batch == 16 {
            let drawn = last.expect("庄家跳张批次必定发出一张牌");
            self.current_player = self.dealer;
            self.last_drawn = Some(drawn);
            self.draw_origin = DrawOrigin::Normal;
        }
        self.phase = Phase::Dealing { batch: batch + 1 };
        Ok(false)
    }

    fn advance_initial_flower_replacement(&mut self) -> Result<bool, GameError> {
        for offset in 0..RuleSet::PLAYER_COUNT {
            let player = PlayerId((self.dealer.0 + offset) % RuleSet::PLAYER_COUNT);
            let Some(position) = self.players[player.0]
                .hand
                .iter()
                .position(|tile| tile.kind().is_flower())
            else {
                continue;
            };
            let flower = self.players[player.0].hand.remove(position);
            self.players[player.0].flowers.push(flower);
            let replacement = self.wall.pop_back().ok_or(GameError::InvalidDeckContents)?;
            self.players[player.0].hand.push(replacement);
            if self.last_drawn == Some(flower) {
                self.last_drawn = Some(replacement);
                self.draw_origin = DrawOrigin::FlowerReplacement;
            }
            return Ok(false);
        }
        self.sort_hands();
        self.phase = Phase::Playing;
        Ok(true)
    }

    pub fn advance_flower_replacement(&mut self) -> Result<PlayerId, GameError> {
        let Phase::ReplacingFlower { player } = self.phase else {
            return Err(GameError::WrongPhase);
        };
        let flower = self.last_drawn.ok_or(GameError::InvalidDeckContents)?;
        if !flower.kind().is_flower() {
            return Err(GameError::InvalidDeckContents);
        }
        let position = self.players[player.0]
            .hand
            .iter()
            .position(|tile| *tile == flower)
            .ok_or(GameError::InvalidDeckContents)?;
        self.players[player.0].hand.remove(position);
        self.players[player.0].flowers.push(flower);
        let replacement = self.wall.pop_back().ok_or(GameError::InvalidDeckContents)?;
        self.players[player.0].hand.push(replacement);
        self.last_drawn = Some(replacement);
        self.draw_origin = DrawOrigin::FlowerReplacement;
        if replacement.kind().is_flower() {
            self.phase = Phase::ReplacingFlower { player };
        } else {
            self.phase = Phase::Playing;
            self.sort_hands();
        }
        Ok(player)
    }

    fn draw_replacement(
        &mut self,
        player: PlayerId,
        origin: DrawOrigin,
    ) -> Result<(Tile, DrawOrigin), GameError> {
        let tile = self.wall.pop_back().ok_or(GameError::CannotKong)?;
        self.players[player.0].hand.push(tile);
        self.current_player = player;
        self.last_drawn = Some(tile);
        self.draw_origin = origin;
        if tile.kind().is_flower() {
            self.phase = Phase::ReplacingFlower { player };
        } else {
            self.phase = Phase::Playing;
            self.sort_hands();
        }
        Ok((tile, origin))
    }

    fn draw_normal(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        let Some(tile) = self.wall.pop_front() else {
            return self.finish_exhaustive_draw();
        };
        self.players[player.0].hand.push(tile);
        self.current_player = player;
        self.last_drawn = Some(tile);
        self.draw_origin = DrawOrigin::Normal;
        if tile.kind().is_flower() {
            self.phase = Phase::ReplacingFlower { player };
        } else {
            self.phase = Phase::Playing;
            self.sort_hands();
        }
        Ok(ActionOutcome::Drew {
            player,
            tile,
            origin: DrawOrigin::Normal,
        })
    }

    fn pending_for_discard(
        &self,
        discard_index: usize,
        from: PlayerId,
        tile: Tile,
    ) -> Result<PendingClaim, GameError> {
        let mut pending = PendingClaim {
            trigger: ClaimTrigger::Discard {
                discard_index,
                from,
                tile,
            },
            options: array::from_fn(|_| Vec::new()),
            responses: [None; RuleSet::PLAYER_COUNT],
        };
        for target_index in 0..RuleSet::PLAYER_COUNT {
            let target = PlayerId(target_index);
            if target == from {
                continue;
            }
            if self.win_button_available(target, tile.kind(), WinSource::Discard(from))? {
                pending.options[target_index].push(ClaimOption::Win);
            }
            let hand = &self.players[target_index].hand;
            let same = hand
                .iter()
                .filter(|held| held.kind() == tile.kind())
                .count();
            if !self.wall.is_empty() && same >= 2 {
                pending.options[target_index].push(ClaimOption::Pung);
            }
            if !self.wall.is_empty() && same >= 3 {
                pending.options[target_index].push(ClaimOption::Kong);
            }
            if target == next_player(from)
                && let TileKind::Suited { suit, rank } = tile.kind()
            {
                for start in rank.saturating_sub(2)..=rank {
                    if (1..=7).contains(&start)
                        && (start..=start + 2)
                            .filter(|value| *value != rank)
                            .all(|value| {
                                hand.iter()
                                    .any(|held| held.kind() == TileKind::suited(suit, value))
                            })
                    {
                        pending.options[target_index].push(ClaimOption::Chow { start });
                    }
                }
            }
        }
        Ok(pending)
    }

    fn resolve_pending_claim(&mut self) -> Result<ActionOutcome, GameError> {
        let Phase::WaitingForClaims(pending) = &self.phase else {
            return Err(GameError::WrongPhase);
        };
        let pending = pending.clone();
        let (from, tile, source) = match pending.trigger {
            ClaimTrigger::Discard { from, tile, .. } => (from, tile, WinSource::Discard(from)),
            ClaimTrigger::AddedKong { player, tile, .. } => {
                (player, tile, WinSource::RobbingKong(player))
            }
        };
        let mut valid_winners = Vec::new();
        for target in players_after(from) {
            if pending.responses[target.0] != Some(Claim::Win) {
                continue;
            }
            let score = self.score_for(target, tile.kind(), source)?;
            if self.is_legal_score(&score) {
                valid_winners.push(WinRecord {
                    player: target,
                    from: Some(from),
                    score,
                });
            } else {
                self.apply_false_win(target);
            }
        }
        if !valid_winners.is_empty() {
            if !self.rules.multiple_winners {
                valid_winners.truncate(1);
            }
            return self.finish_with_winners(valid_winners);
        }

        match pending.trigger {
            ClaimTrigger::AddedKong {
                player,
                tile,
                meld_index,
            } => {
                self.finalize_added_kong(player, tile, meld_index)?;
                Ok(ActionOutcome::KongDeclared {
                    player,
                    tile: tile.kind(),
                    added: true,
                })
            }
            ClaimTrigger::Discard {
                discard_index,
                from,
                tile,
            } => {
                for claim in [Claim::Kong, Claim::Pung] {
                    if let Some(player) = players_after(from)
                        .find(|player| pending.responses[player.0] == Some(claim))
                    {
                        return self.apply_discard_claim(player, claim, tile, discard_index, from);
                    }
                }
                if let Some((player, claim)) = players_after(from).find_map(|player| {
                    let claim = pending.responses[player.0]?;
                    matches!(claim, Claim::Chow { .. }).then_some((player, claim))
                }) {
                    return self.apply_discard_claim(player, claim, tile, discard_index, from);
                }
                self.phase = Phase::Playing;
                self.advance_after_unclaimed_discard(from)
            }
        }
    }

    fn apply_discard_claim(
        &mut self,
        player: PlayerId,
        claim: Claim,
        tile: Tile,
        discard_index: usize,
        from: PlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.discards[discard_index].claimed_by = Some(player);
        match claim {
            Claim::Pung => {
                self.remove_kind_from_hand(player, tile.kind(), 2)?;
                self.players[player.0]
                    .melds
                    .push(Meld::pung(tile.kind(), from));
                self.current_player = player;
                self.last_drawn = None;
                self.phase = Phase::Playing;
            }
            Claim::Kong => {
                self.remove_kind_from_hand(player, tile.kind(), 3)?;
                self.players[player.0]
                    .melds
                    .push(Meld::melded_kong(tile.kind(), from));
                self.current_player = player;
                self.phase = Phase::Playing;
                self.draw_replacement(player, DrawOrigin::KongReplacement)?;
            }
            Claim::Chow { start } => {
                let TileKind::Suited { suit, rank } = tile.kind() else {
                    return Err(GameError::InvalidClaim);
                };
                for value in start..=start + 2 {
                    if value != rank {
                        self.remove_kind_from_hand(player, TileKind::suited(suit, value), 1)?;
                    }
                }
                self.players[player.0]
                    .melds
                    .push(Meld::chow(suit, start, from));
                self.current_player = player;
                self.last_drawn = None;
                self.phase = Phase::Playing;
            }
            Claim::Pass | Claim::Win => return Err(GameError::InvalidClaim),
        }
        if !matches!(self.phase, Phase::ReplacingFlower { .. }) {
            self.sort_hands();
        }
        Ok(ActionOutcome::Claimed {
            player,
            source: from,
            tile,
            claim,
        })
    }

    fn finalize_added_kong(
        &mut self,
        player: PlayerId,
        tile: Tile,
        meld_index: usize,
    ) -> Result<(), GameError> {
        let original = self.players[player.0].melds[meld_index];
        let from = original.claimed_from().ok_or(GameError::CannotKong)?;
        let position = self.players[player.0]
            .hand
            .iter()
            .position(|held| *held == tile)
            .ok_or(GameError::TileNotInHand(tile))?;
        self.players[player.0].hand.remove(position);
        self.players[player.0].melds[meld_index] = Meld::melded_kong(tile.kind(), from);
        self.phase = Phase::Playing;
        self.current_player = player;
        self.draw_replacement(player, DrawOrigin::KongReplacement)?;
        Ok(())
    }

    fn advance_after_unclaimed_discard(
        &mut self,
        from: PlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.phase = Phase::Playing;
        self.draw_normal(next_player(from))
    }

    fn win_button_available(
        &self,
        player: PlayerId,
        tile: TileKind,
        source: WinSource,
    ) -> Result<bool, GameError> {
        if self.players[player.0].dead_hand {
            return Ok(false);
        }
        let mut concealed: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .map(|held| held.kind())
            .collect();
        concealed.push(tile);
        if !is_complete_hand(&concealed, &self.players[player.0].melds) {
            return Ok(false);
        }
        let score = self.score_for_with_concealed(player, tile, source, concealed)?;
        Ok(self.is_legal_score(&score) || self.rules.false_win)
    }

    fn score_for(
        &self,
        player: PlayerId,
        winning_tile: TileKind,
        source: WinSource,
    ) -> Result<ScoreResult, GameError> {
        let mut concealed: Vec<_> = self.players[player.0]
            .hand
            .iter()
            .map(|tile| tile.kind())
            .collect();
        if !source.is_self_draw() {
            concealed.push(winning_tile);
        }
        self.score_for_with_concealed(player, winning_tile, source, concealed)
    }

    fn score_for_with_concealed(
        &self,
        player: PlayerId,
        winning_tile: TileKind,
        source: WinSource,
        concealed: Vec<TileKind>,
    ) -> Result<ScoreResult, GameError> {
        let score = score_hand(&ScoreInput {
            concealed,
            melds: self.players[player.0].melds.clone(),
            winning_tile,
            context: WinContext {
                source,
                seat_wind: self.seat_wind(player).expect("player was validated"),
                prevalent_wind: self.prevalent_wind,
                last_wall_tile: self.wall.is_empty(),
                last_of_kind: self.is_last_of_kind(winning_tile, source),
                flower_count: self.players[player.0].flowers.len() as u8,
            },
        })?;
        Ok(score)
    }

    fn is_legal_score(&self, score: &ScoreResult) -> bool {
        !self.rules.minimum_eight_points || score.points_without_flowers >= 8
    }

    fn is_last_of_kind(&self, tile: TileKind, source: WinSource) -> bool {
        let mut visible = self
            .discards
            .iter()
            .filter(|discard| discard.claimed_by.is_none() && discard.tile.kind() == tile)
            .count();
        for player in &self.players {
            for meld in &player.melds {
                if meld.is_open() {
                    visible += meld
                        .tile_kinds()
                        .iter()
                        .filter(|kind| **kind == tile)
                        .count();
                }
            }
        }
        match source {
            WinSource::Discard(_) => visible >= 4,
            WinSource::RobbingKong(_) => false,
            _ => visible >= 3,
        }
    }

    fn apply_false_win(&mut self, player: PlayerId) -> [i32; RuleSet::PLAYER_COUNT] {
        let mut delta = [10; RuleSet::PLAYER_COUNT];
        delta[player.0] = -30;
        for (hand_delta, penalty) in self.hand_deltas.iter_mut().zip(delta) {
            *hand_delta += penalty;
        }
        self.players[player.0].dead_hand = true;
        self.players[player.0].hand_revealed = true;
        delta
    }

    fn finish_with_winners(&mut self, winners: Vec<WinRecord>) -> Result<ActionOutcome, GameError> {
        let claimed_tile = match &self.phase {
            Phase::WaitingForClaims(pending) => Some(pending.tile()),
            _ => None,
        };
        for winner in &winners {
            let hand = &mut self.players[winner.player.0];
            hand.hand_revealed = true;
            if winner.from.is_some()
                && let Some(tile) = claimed_tile
            {
                hand.hand.push(tile);
                hand.hand.sort_by_key(|tile| (tile.kind(), tile.copy()));
            }
        }
        let winner_ids: HashSet<_> = winners.iter().map(|winner| winner.player).collect();
        for winner in &winners {
            let points = i32::from(winner.score.total_points);
            match winner.from {
                None => {
                    let payment = points + i32::from(self.rules.minimum_eight_points) * 8;
                    for payer in 0..RuleSet::PLAYER_COUNT {
                        if payer != winner.player.0 {
                            self.hand_deltas[payer] -= payment;
                            self.hand_deltas[winner.player.0] += payment;
                        }
                    }
                }
                Some(discarder) => {
                    let base = i32::from(self.rules.minimum_eight_points) * 8;
                    let payment = points + base;
                    self.hand_deltas[discarder.0] -= payment;
                    self.hand_deltas[winner.player.0] += payment;
                    if self.rules.minimum_eight_points {
                        for payer in 0..RuleSet::PLAYER_COUNT {
                            let payer = PlayerId(payer);
                            if payer != discarder && !winner_ids.contains(&payer) {
                                self.hand_deltas[payer.0] -= 8;
                                self.hand_deltas[winner.player.0] += 8;
                            }
                        }
                    }
                }
            }
        }
        self.finish_hand(winners, false)
    }

    fn finish_exhaustive_draw(&mut self) -> Result<ActionOutcome, GameError> {
        self.finish_hand(Vec::new(), true)
    }

    fn finish_hand(
        &mut self,
        winners: Vec<WinRecord>,
        exhaustive_draw: bool,
    ) -> Result<ActionOutcome, GameError> {
        for index in 0..RuleSet::PLAYER_COUNT {
            self.match_scores[index] += self.hand_deltas[index];
        }
        self.hands_in_match += 1;
        let match_complete = self.rules.match_length == MatchLength::SingleHand
            || self.hands_in_match >= self.rules.match_length.hand_count();
        let result = HandResult {
            winners,
            exhaustive_draw,
            deltas: self.hand_deltas,
            match_scores: self.match_scores,
            match_complete,
            sequence_index: self.sequence_index,
        };
        self.phase = Phase::Finished(result.clone());
        Ok(ActionOutcome::HandFinished(result))
    }

    fn remove_kind_from_hand(
        &mut self,
        player: PlayerId,
        kind: TileKind,
        count: usize,
    ) -> Result<(), GameError> {
        for _ in 0..count {
            let Some(position) = self.players[player.0]
                .hand
                .iter()
                .position(|tile| tile.kind() == kind)
            else {
                return Err(GameError::InvalidClaim);
            };
            self.players[player.0].hand.remove(position);
        }
        Ok(())
    }

    fn ensure_playing_turn(&self, player: PlayerId) -> Result<(), GameError> {
        validate_player(player)?;
        if !matches!(self.phase, Phase::Playing) {
            return Err(GameError::WrongPhase);
        }
        if self.current_player != player {
            return Err(GameError::NotPlayersTurn {
                expected: self.current_player,
                actual: player,
            });
        }
        Ok(())
    }

    fn sort_hands(&mut self) {
        for player in &mut self.players {
            player.hand.sort_by_key(|tile| (tile.kind(), tile.copy()));
        }
    }
}

fn validate_player(player: PlayerId) -> Result<(), GameError> {
    (player.0 < RuleSet::PLAYER_COUNT)
        .then_some(())
        .ok_or(GameError::InvalidPlayer(player))
}

fn validate_deck(deck: &[Tile]) -> Result<(), GameError> {
    if deck.len() != 144 {
        return Err(GameError::InvalidDeckSize {
            expected: 144,
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = build_deck().into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}

const fn next_player(player: PlayerId) -> PlayerId {
    PlayerId((player.0 + 1) % RuleSet::PLAYER_COUNT)
}

fn players_after(player: PlayerId) -> impl Iterator<Item = PlayerId> {
    (1..RuleSet::PLAYER_COUNT)
        .map(move |offset| PlayerId((player.0 + offset) % RuleSet::PLAYER_COUNT))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finish_dealing(game: &mut GameState) {
        while matches!(game.phase(), Phase::Dealing { .. }) {
            game.advance_deal().unwrap();
        }
    }

    fn score(points: u16) -> ScoreResult {
        ScoreResult {
            fans: Vec::new(),
            points_without_flowers: points,
            flower_points: 0,
            total_points: points,
        }
    }

    #[test]
    fn deals_thirteen_tiles_and_a_dealer_draw_with_automatic_flowers() {
        let mut game =
            GameState::new_with_deck(RuleSet::default(), build_deck(), PlayerId(0)).unwrap();
        assert!(matches!(game.phase(), Phase::Dealing { batch: 0 }));
        assert_eq!(game.wall_len(), 144);
        game.advance_deal().unwrap();
        assert_eq!(game.players[0].hand.len(), 4);
        assert_eq!(game.wall_len(), 140);
        finish_dealing(&mut game);
        for player in &game.players {
            let expected = if player.id == PlayerId(0) { 14 } else { 13 };
            assert_eq!(player.hand.len(), expected);
            assert!(player.hand.iter().all(|tile| !tile.kind().is_flower()));
        }
        assert_eq!(
            game.players
                .iter()
                .map(|player| player.flowers.len())
                .sum::<usize>(),
            0
        );
        assert_eq!(game.wall_len(), 91);
    }

    #[test]
    fn flowers_remain_in_hand_until_the_timed_replacement_step() {
        let mut deck = build_deck();
        let flower = deck
            .iter()
            .position(|tile| tile.kind().is_flower())
            .expect("standard wall contains flowers");
        deck.swap(0, flower);
        let mut game = GameState::new_with_deck(RuleSet::default(), deck, PlayerId(0)).unwrap();

        game.advance_deal().unwrap();
        assert!(
            game.players[0]
                .hand
                .iter()
                .any(|tile| tile.kind().is_flower())
        );
        assert!(game.players[0].flowers.is_empty());
        while !matches!(game.phase(), Phase::Dealing { batch: 17 }) {
            game.advance_deal().unwrap();
        }
        assert!(
            game.players[0]
                .hand
                .iter()
                .any(|tile| tile.kind().is_flower())
        );

        game.advance_deal().unwrap();
        assert_eq!(game.players[0].flowers.len(), 1);
        while matches!(game.phase(), Phase::Dealing { .. }) {
            game.advance_deal().unwrap();
        }
        assert!(
            game.players[0]
                .hand
                .iter()
                .all(|tile| !tile.kind().is_flower())
        );
    }

    #[test]
    fn a_flower_draw_pauses_play_until_it_is_replaced() {
        let mut deck = build_deck();
        let flower = deck
            .iter()
            .position(|tile| tile.kind().is_flower())
            .expect("standard wall contains flowers");
        deck.swap(53, flower);
        let mut game = GameState::new_with_deck(RuleSet::default(), deck, PlayerId(0)).unwrap();
        finish_dealing(&mut game);

        let outcome = game.draw_normal(PlayerId(1)).unwrap();
        assert!(matches!(outcome, ActionOutcome::Drew { tile, .. } if tile.kind().is_flower()));
        assert!(matches!(
            game.phase(),
            Phase::ReplacingFlower {
                player: PlayerId(1)
            }
        ));
        assert!(
            game.players[1]
                .hand
                .last()
                .is_some_and(|tile| tile.kind().is_flower())
        );

        while matches!(game.phase(), Phase::ReplacingFlower { .. }) {
            assert_eq!(game.advance_flower_replacement().unwrap(), PlayerId(1));
        }
        assert!(matches!(game.phase(), Phase::Playing));
        assert!(!game.players[1].flowers.is_empty());
        assert!(
            game.players[1]
                .hand
                .iter()
                .all(|tile| !tile.kind().is_flower())
        );
    }

    #[test]
    fn single_hand_ready_continues_round_rotation_but_resets_match_score() {
        let mut game =
            GameState::new_with_deck(RuleSet::default(), build_deck(), PlayerId(2)).unwrap();
        finish_dealing(&mut game);
        game.finish_exhaustive_draw().unwrap();
        game.start_next_hand(build_deck()).unwrap();
        assert!(matches!(game.phase(), Phase::Dealing { batch: 0 }));
        assert_eq!(game.sequence_index(), 1);
        assert_eq!(game.dealer(), PlayerId(3));
        assert_eq!(game.prevalent_wind(), Wind::East);
        assert_eq!(game.match_scores(), &[0; 4]);
    }

    #[test]
    fn false_win_penalty_reveals_and_kills_the_hand() {
        let mut game = GameState::new_with_deck(
            RuleSet {
                false_win: true,
                ..RuleSet::default()
            },
            build_deck(),
            PlayerId(0),
        )
        .unwrap();
        finish_dealing(&mut game);
        let delta = game.apply_false_win(PlayerId(1));
        assert_eq!(delta, [10, -30, 10, 10]);
        assert!(game.players[1].dead_hand);
        assert!(
            game.public_player(PlayerId(0), PlayerId(1))
                .unwrap()
                .revealed_hand
                .is_some()
        );
    }

    #[test]
    fn concealed_kong_kind_is_hidden_from_other_players_until_settlement() {
        let mut game =
            GameState::new_with_deck(RuleSet::default(), build_deck(), PlayerId(0)).unwrap();
        finish_dealing(&mut game);
        game.players[0]
            .melds
            .push(Meld::concealed_kong(TileKind::Dragon(crate::Dragon::White)));
        let owner_view = game.public_player(PlayerId(0), PlayerId(0)).unwrap();
        let other_view = game.public_player(PlayerId(1), PlayerId(0)).unwrap();
        assert_eq!(
            owner_view.melds.last().unwrap().tile,
            Some(TileKind::Dragon(crate::Dragon::White))
        );
        assert_eq!(other_view.melds.last().unwrap().tile, None);
        game.finish_exhaustive_draw().unwrap();
        assert_eq!(
            game.public_player(PlayerId(1), PlayerId(0))
                .unwrap()
                .melds
                .last()
                .unwrap()
                .tile,
            Some(TileKind::Dragon(crate::Dragon::White))
        );
    }

    #[test]
    fn a_chow_waits_for_a_possible_higher_priority_pung() {
        let mut game =
            GameState::new_with_deck(RuleSet::default(), build_deck(), PlayerId(0)).unwrap();
        finish_dealing(&mut game);
        for player in &mut game.players {
            player.hand.clear();
        }
        game.players[1].hand.extend([
            Tile::new(TileKind::suited(crate::Suit::Characters, 2), 0),
            Tile::new(TileKind::suited(crate::Suit::Characters, 3), 0),
        ]);
        game.players[2].hand.extend([
            Tile::new(TileKind::suited(crate::Suit::Characters, 1), 1),
            Tile::new(TileKind::suited(crate::Suit::Characters, 1), 2),
        ]);
        let discarded = Tile::new(TileKind::suited(crate::Suit::Characters, 1), 0);
        let pending = game.pending_for_discard(0, PlayerId(0), discarded).unwrap();
        assert!(
            pending
                .options_for(PlayerId(1))
                .unwrap()
                .contains(&ClaimOption::Chow { start: 1 })
        );
        assert!(
            pending
                .options_for(PlayerId(2))
                .unwrap()
                .contains(&ClaimOption::Pung)
        );
        assert_eq!(pending.waiting_for(), vec![PlayerId(1), PlayerId(2)]);
    }

    #[test]
    fn multiple_winners_do_not_pay_each_other_base_points() {
        let mut game = GameState::new_with_deck(
            RuleSet {
                multiple_winners: true,
                ..RuleSet::default()
            },
            build_deck(),
            PlayerId(0),
        )
        .unwrap();
        finish_dealing(&mut game);
        let winning_tile = game.players[0].hand[0];
        let winner_one_count = game.players[1].hand.len();
        let winner_two_count = game.players[2].hand.len();
        game.phase = Phase::WaitingForClaims(PendingClaim {
            trigger: ClaimTrigger::Discard {
                discard_index: 0,
                from: PlayerId(0),
                tile: winning_tile,
            },
            options: array::from_fn(|_| Vec::new()),
            responses: [None; RuleSet::PLAYER_COUNT],
        });
        let outcome = game
            .finish_with_winners(vec![
                WinRecord {
                    player: PlayerId(1),
                    from: Some(PlayerId(0)),
                    score: score(10),
                },
                WinRecord {
                    player: PlayerId(2),
                    from: Some(PlayerId(0)),
                    score: score(20),
                },
            ])
            .unwrap();
        let ActionOutcome::HandFinished(result) = outcome else {
            panic!("expected hand result");
        };
        assert_eq!(result.deltas, [-46, 26, 36, -16]);
        for (winner, previous_count) in [
            (PlayerId(1), winner_one_count),
            (PlayerId(2), winner_two_count),
        ] {
            assert!(game.players[winner.0].hand_revealed);
            assert_eq!(game.players[winner.0].hand.len(), previous_count + 1);
            assert!(game.players[winner.0].hand.contains(&winning_tile));
            assert!(
                game.public_player(PlayerId(3), winner)
                    .unwrap()
                    .revealed_hand
                    .is_some()
            );
        }
    }

    #[test]
    fn disabling_minimum_removes_all_base_payments() {
        let mut game = GameState::new_with_deck(
            RuleSet {
                minimum_eight_points: false,
                multiple_winners: true,
                ..RuleSet::default()
            },
            build_deck(),
            PlayerId(0),
        )
        .unwrap();
        finish_dealing(&mut game);
        let outcome = game
            .finish_with_winners(vec![
                WinRecord {
                    player: PlayerId(1),
                    from: Some(PlayerId(0)),
                    score: score(10),
                },
                WinRecord {
                    player: PlayerId(2),
                    from: Some(PlayerId(0)),
                    score: score(20),
                },
            ])
            .unwrap();
        let ActionOutcome::HandFinished(result) = outcome else {
            panic!("expected hand result");
        };
        assert_eq!(result.deltas, [-30, 10, 20, 0]);
    }
}
