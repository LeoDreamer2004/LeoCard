#[path = "game_actions.rs"]
mod actions;
#[path = "game_lifecycle.rs"]
mod lifecycle;
#[path = "game_resolution.rs"]
mod resolution;
#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;

use crate::{
    MahjongMatchLength, MahjongMeldKind, MahjongPlayerId, MahjongRuleSet, MahjongScoreResult,
    MahjongTile, MahjongTileKind, MahjongWind, Meld, RuleError, ScoreError, ScoreInput, WinContext,
    WinSource, build_deck, is_complete_hand, score_hand,
};
use std::array;
use std::collections::{HashSet, VecDeque};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongDrawOrigin {
    Normal,
    KongReplacement,
    FlowerReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerState {
    id: MahjongPlayerId,
    hand: Vec<MahjongTile>,
    melds: Vec<Meld>,
    flowers: Vec<MahjongTile>,
    dead_hand: bool,
    hand_revealed: bool,
}

impl PlayerState {
    pub const fn id(&self) -> MahjongPlayerId {
        self.id
    }

    pub fn hand(&self) -> &[MahjongTile] {
        &self.hand
    }

    pub fn melds(&self) -> &[Meld] {
        &self.melds
    }

    pub fn flowers(&self) -> &[MahjongTile] {
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
    pub id: MahjongPlayerId,
    pub concealed_count: usize,
    pub revealed_hand: Option<Vec<MahjongTile>>,
    pub melds: Vec<PublicMeld>,
    pub flowers: Vec<MahjongTile>,
    pub dead_hand: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicMeld {
    pub kind: MahjongMeldKind,
    /// 他人的暗杠在本盘结算前只公开为四张牌背。
    pub tile: Option<MahjongTileKind>,
    pub claimed_from: Option<MahjongPlayerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Discard {
    pub player: MahjongPlayerId,
    pub tile: MahjongTile,
    pub claimed_by: Option<MahjongPlayerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongClaimOption {
    Chow { start: u8 },
    Pung,
    Kong,
    Win,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongClaim {
    Pass,
    Chow { start: u8 },
    Pung,
    Kong,
    Win,
}

impl MahjongClaim {
    const fn option(self) -> Option<MahjongClaimOption> {
        match self {
            Self::Pass => None,
            Self::Chow { start } => Some(MahjongClaimOption::Chow { start }),
            Self::Pung => Some(MahjongClaimOption::Pung),
            Self::Kong => Some(MahjongClaimOption::Kong),
            Self::Win => Some(MahjongClaimOption::Win),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ClaimPriority {
    category: u8,
    proximity: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum ClaimTrigger {
    Discard {
        discard_index: usize,
        from: MahjongPlayerId,
        tile: MahjongTile,
    },
    AddedKong {
        player: MahjongPlayerId,
        tile: MahjongTile,
        meld_index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PendingClaim {
    trigger: ClaimTrigger,
    options: [Vec<MahjongClaimOption>; MahjongRuleSet::PLAYER_COUNT],
    responses: [Option<MahjongClaim>; MahjongRuleSet::PLAYER_COUNT],
}

impl PendingClaim {
    pub const fn source_player(&self) -> MahjongPlayerId {
        match self.trigger {
            ClaimTrigger::Discard { from, .. } => from,
            ClaimTrigger::AddedKong { player, .. } => player,
        }
    }

    pub const fn tile(&self) -> MahjongTile {
        match self.trigger {
            ClaimTrigger::Discard { tile, .. } | ClaimTrigger::AddedKong { tile, .. } => tile,
        }
    }

    pub const fn is_robbing_kong_window(&self) -> bool {
        matches!(self.trigger, ClaimTrigger::AddedKong { .. })
    }

    pub fn options_for(&self, player: MahjongPlayerId) -> Option<&[MahjongClaimOption]> {
        self.options.get(player.0).map(Vec::as_slice)
    }

    pub fn response_from(&self, player: MahjongPlayerId) -> Option<MahjongClaim> {
        self.responses.get(player.0).copied().flatten()
    }

    pub fn waiting_for(&self) -> Vec<MahjongPlayerId> {
        (0..MahjongRuleSet::PLAYER_COUNT)
            .filter(|index| !self.options[*index].is_empty() && self.responses[*index].is_none())
            .map(MahjongPlayerId)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WinRecord {
    pub player: MahjongPlayerId,
    pub from: Option<MahjongPlayerId>,
    pub winning_tile: MahjongTile,
    pub score: MahjongScoreResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandResult {
    pub winners: Vec<WinRecord>,
    pub exhaustive_draw: bool,
    pub deltas: [i32; MahjongRuleSet::PLAYER_COUNT],
    pub match_scores: [i32; MahjongRuleSet::PLAYER_COUNT],
    pub match_complete: bool,
    pub sequence_index: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Phase {
    Dealing { batch: u8 },
    ReplacingFlower { player: MahjongPlayerId },
    Playing,
    WaitingForClaims(PendingClaim),
    Finished(HandResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionOutcome {
    Discarded {
        player: MahjongPlayerId,
        tile: MahjongTile,
    },
    ClaimRecorded {
        player: MahjongPlayerId,
    },
    Claimed {
        player: MahjongPlayerId,
        source: MahjongPlayerId,
        tile: MahjongTile,
        claim: MahjongClaim,
    },
    Drew {
        player: MahjongPlayerId,
        tile: MahjongTile,
        origin: MahjongDrawOrigin,
    },
    KongDeclared {
        player: MahjongPlayerId,
        tile: MahjongTileKind,
        added: bool,
    },
    FalseWin {
        player: MahjongPlayerId,
        deltas: [i32; MahjongRuleSet::PLAYER_COUNT],
    },
    HandFinished(HandResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongHandReplacementError {
    WrongTileCount {
        expected: u16,
        actual: u16,
    },
    FlowerNotAllowed {
        tile: MahjongTileKind,
    },
    TileUnavailable {
        tile: MahjongTileKind,
        requested: u16,
        available: u16,
    },
}

impl fmt::Display for MahjongHandReplacementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongTileCount { expected, actual } => {
                write!(f, "手牌张数不对：需要 {expected} 张，实际输入 {actual} 张")
            }
            Self::FlowerNotAllowed { tile } => {
                write!(f, "开发者手牌不能包含花牌（{tile}）")
            }
            Self::TileUnavailable {
                tile,
                requested,
                available,
            } => write!(
                f,
                "{tile}存量不足：需要 {requested} 张，当前手牌和牌山中只有 {available} 张"
            ),
        }
    }
}

impl std::error::Error for MahjongHandReplacementError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(MahjongPlayerId),
    NotPlayersTurn {
        expected: MahjongPlayerId,
        actual: MahjongPlayerId,
    },
    WrongPhase,
    TileNotInHand(MahjongTile),
    InvalidClaim,
    AlreadyResponded,
    CannotWin,
    CannotKong,
    InvalidHandReplacement(MahjongHandReplacementError),
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
            Self::InvalidHandReplacement(error) => error.fmt(f),
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
    rules: MahjongRuleSet,
    players: [PlayerState; MahjongRuleSet::PLAYER_COUNT],
    wall: VecDeque<MahjongTile>,
    discards: Vec<Discard>,
    initial_dealer: MahjongPlayerId,
    dealer: MahjongPlayerId,
    prevalent_wind: MahjongWind,
    sequence_index: u8,
    hands_in_match: u8,
    current_player: MahjongPlayerId,
    last_drawn: Option<MahjongTile>,
    draw_origin: MahjongDrawOrigin,
    hand_deltas: [i32; MahjongRuleSet::PLAYER_COUNT],
    match_scores: [i32; MahjongRuleSet::PLAYER_COUNT],
    phase: Phase,
}

impl GameState {
    /// `deck[0]` 是牌墙前端第一张牌；补花和杠牌从牌墙尾端取牌。
    pub fn new_with_deck(
        rules: MahjongRuleSet,
        deck: Vec<MahjongTile>,
        initial_dealer: MahjongPlayerId,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        validate_player(initial_dealer)?;
        validate_deck(&deck)?;
        let players = array::from_fn(|index| PlayerState {
            id: MahjongPlayerId(index),
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
            prevalent_wind: MahjongWind::East,
            sequence_index: 0,
            hands_in_match: 0,
            current_player: initial_dealer,
            last_drawn: None,
            draw_origin: MahjongDrawOrigin::Normal,
            hand_deltas: [0; MahjongRuleSet::PLAYER_COUNT],
            match_scores: [0; MahjongRuleSet::PLAYER_COUNT],
            phase: Phase::Dealing { batch: 0 },
        };
        Ok(game)
    }

    pub const fn rules(&self) -> &MahjongRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState; MahjongRuleSet::PLAYER_COUNT] {
        &self.players
    }

    pub fn player(&self, player: MahjongPlayerId) -> Option<&PlayerState> {
        self.players.get(player.0)
    }

    pub fn public_player(
        &self,
        viewer: MahjongPlayerId,
        player: MahjongPlayerId,
    ) -> Option<PublicPlayerState> {
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
                    tile: (!matches!(
                        meld.kind(),
                        MahjongMeldKind::Kong(crate::MahjongKongKind::Concealed)
                    ) || reveal_concealed_kong)
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

    pub const fn dealer(&self) -> MahjongPlayerId {
        self.dealer
    }

    pub const fn prevalent_wind(&self) -> MahjongWind {
        self.prevalent_wind
    }

    pub const fn sequence_index(&self) -> u8 {
        self.sequence_index
    }

    pub const fn current_player(&self) -> MahjongPlayerId {
        self.current_player
    }

    pub const fn last_drawn(&self) -> Option<MahjongTile> {
        self.last_drawn
    }

    pub fn wall_len(&self) -> usize {
        self.wall.len()
    }

    pub fn replace_player_hand_from_wall(
        &mut self,
        player: MahjongPlayerId,
        kinds: &[MahjongTileKind],
    ) -> Result<(), GameError> {
        validate_player(player)?;
        if !matches!(self.phase, Phase::Playing) {
            return Err(GameError::WrongPhase);
        }
        let old_hand = &self.players[player.0].hand;
        if kinds.len() != old_hand.len() {
            return Err(GameError::InvalidHandReplacement(
                MahjongHandReplacementError::WrongTileCount {
                    expected: old_hand.len() as u16,
                    actual: kinds.len() as u16,
                },
            ));
        }
        if let Some(tile) = kinds.iter().copied().find(|kind| kind.is_flower()) {
            return Err(GameError::InvalidHandReplacement(
                MahjongHandReplacementError::FlowerNotAllowed { tile },
            ));
        }
        let mut checked = HashSet::new();
        for tile in kinds.iter().copied() {
            if !checked.insert(tile) {
                continue;
            }
            let requested = kinds.iter().filter(|kind| **kind == tile).count();
            let available = old_hand
                .iter()
                .chain(self.wall.iter())
                .filter(|candidate| candidate.kind() == tile)
                .count();
            if requested > available {
                return Err(GameError::InvalidHandReplacement(
                    MahjongHandReplacementError::TileUnavailable {
                        tile,
                        requested: requested as u16,
                        available: available as u16,
                    },
                ));
            }
        }

        let mut old_used = vec![false; old_hand.len()];
        let mut replacement = vec![None; kinds.len()];
        let mut missing = Vec::new();
        for (desired_index, kind) in kinds.iter().copied().enumerate() {
            if let Some((old_index, tile)) = old_hand
                .iter()
                .copied()
                .enumerate()
                .find(|(index, tile)| !old_used[*index] && tile.kind() == kind)
            {
                old_used[old_index] = true;
                replacement[desired_index] = Some(tile);
            } else {
                missing.push((desired_index, kind));
            }
        }

        let returned = old_hand
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, tile)| (!old_used[index]).then_some(tile))
            .collect::<Vec<_>>();
        let mut wall_used = vec![false; self.wall.len()];
        let mut swaps = Vec::with_capacity(missing.len());
        for ((desired_index, kind), returned_tile) in
            missing.into_iter().zip(returned.iter().copied())
        {
            let Some((wall_index, wall_tile)) = self
                .wall
                .iter()
                .copied()
                .enumerate()
                .find(|(index, tile)| !wall_used[*index] && tile.kind() == kind)
            else {
                return Err(GameError::InvalidHandReplacement(
                    MahjongHandReplacementError::TileUnavailable {
                        tile: kind,
                        requested: kinds.iter().filter(|candidate| **candidate == kind).count()
                            as u16,
                        available: old_hand
                            .iter()
                            .chain(self.wall.iter())
                            .filter(|candidate| candidate.kind() == kind)
                            .count() as u16,
                    },
                ));
            };
            wall_used[wall_index] = true;
            replacement[desired_index] = Some(wall_tile);
            swaps.push((wall_index, returned_tile));
        }

        let new_hand = replacement
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .expect("validated replacement fills every hand position");
        for (wall_index, returned_tile) in swaps {
            self.wall[wall_index] = returned_tile;
        }
        if player == self.current_player && self.last_drawn.is_some() {
            self.last_drawn = new_hand.last().copied();
        }
        self.players[player.0].hand = new_hand;
        Ok(())
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub const fn match_scores(&self) -> &[i32; MahjongRuleSet::PLAYER_COUNT] {
        &self.match_scores
    }

    pub fn seat_wind(&self, player: MahjongPlayerId) -> Option<MahjongWind> {
        validate_player(player).ok()?;
        let distance = (player.0 + MahjongRuleSet::PLAYER_COUNT - self.dealer.0)
            % MahjongRuleSet::PLAYER_COUNT;
        Some(MahjongWind::ALL[distance])
    }

    fn ensure_playing_turn(&self, player: MahjongPlayerId) -> Result<(), GameError> {
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

fn validate_player(player: MahjongPlayerId) -> Result<(), GameError> {
    (player.0 < MahjongRuleSet::PLAYER_COUNT)
        .then_some(())
        .ok_or(GameError::InvalidPlayer(player))
}

fn claim_priority(
    source: MahjongPlayerId,
    player: MahjongPlayerId,
    claim: MahjongClaim,
) -> ClaimPriority {
    let category = match claim {
        MahjongClaim::Win => 3,
        MahjongClaim::Pung | MahjongClaim::Kong => 2,
        MahjongClaim::Chow { .. } => 1,
        MahjongClaim::Pass => 0,
    };
    let distance =
        (player.0 + MahjongRuleSet::PLAYER_COUNT - source.0) % MahjongRuleSet::PLAYER_COUNT;
    ClaimPriority {
        category,
        proximity: MahjongRuleSet::PLAYER_COUNT as u8 - distance as u8,
    }
}

fn claim_option_priority(
    source: MahjongPlayerId,
    player: MahjongPlayerId,
    option: MahjongClaimOption,
) -> ClaimPriority {
    let claim = match option {
        MahjongClaimOption::Chow { start } => MahjongClaim::Chow { start },
        MahjongClaimOption::Pung => MahjongClaim::Pung,
        MahjongClaimOption::Kong => MahjongClaim::Kong,
        MahjongClaimOption::Win => MahjongClaim::Win,
    };
    claim_priority(source, player, claim)
}

fn validate_deck(deck: &[MahjongTile]) -> Result<(), GameError> {
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

const fn next_player(player: MahjongPlayerId) -> MahjongPlayerId {
    MahjongPlayerId((player.0 + 1) % MahjongRuleSet::PLAYER_COUNT)
}

fn players_after(player: MahjongPlayerId) -> impl Iterator<Item = MahjongPlayerId> {
    (1..MahjongRuleSet::PLAYER_COUNT)
        .map(move |offset| MahjongPlayerId((player.0 + offset) % MahjongRuleSet::PLAYER_COUNT))
}
