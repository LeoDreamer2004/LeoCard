use super::{
    ClaimPriority, Discard, GameError, MahjongClaim, MahjongClaimOption, MahjongDrawOrigin, Phase,
    PlayerState, PublicMeld, PublicPlayerState,
};
use crate::{
    MahjongMeldKind, MahjongPlayerId, MahjongRuleSet, MahjongTile, MahjongWind, build_deck,
};
use std::array;
use std::collections::{HashSet, VecDeque};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub(super) rules: MahjongRuleSet,
    pub(super) players: [PlayerState; MahjongRuleSet::PLAYER_COUNT],
    pub(super) wall: VecDeque<MahjongTile>,
    pub(super) discards: Vec<Discard>,
    pub(super) initial_dealer: MahjongPlayerId,
    pub(super) dealer: MahjongPlayerId,
    pub(super) prevalent_wind: MahjongWind,
    pub(super) sequence_index: u8,
    pub(super) hands_in_match: u8,
    pub(super) current_player: MahjongPlayerId,
    pub(super) last_drawn: Option<MahjongTile>,
    pub(super) draw_origin: MahjongDrawOrigin,
    pub(super) hand_deltas: [i32; MahjongRuleSet::PLAYER_COUNT],
    pub(super) match_scores: [i32; MahjongRuleSet::PLAYER_COUNT],
    pub(super) phase: Phase,
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

    pub(super) fn ensure_playing_turn(&self, player: MahjongPlayerId) -> Result<(), GameError> {
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

    pub(super) fn sort_hands(&mut self) {
        for player in &mut self.players {
            player.hand.sort_by_key(|tile| (tile.kind(), tile.copy()));
        }
    }
}

pub(super) fn validate_player(player: MahjongPlayerId) -> Result<(), GameError> {
    (player.0 < MahjongRuleSet::PLAYER_COUNT)
        .then_some(())
        .ok_or(GameError::InvalidPlayer(player))
}

pub(super) fn claim_priority(
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

pub(super) fn claim_option_priority(
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

pub(super) fn validate_deck(deck: &[MahjongTile]) -> Result<(), GameError> {
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

pub(super) const fn next_player(player: MahjongPlayerId) -> MahjongPlayerId {
    MahjongPlayerId((player.0 + 1) % MahjongRuleSet::PLAYER_COUNT)
}

pub(super) fn players_after(player: MahjongPlayerId) -> impl Iterator<Item = MahjongPlayerId> {
    (1..MahjongRuleSet::PLAYER_COUNT)
        .map(move |offset| MahjongPlayerId((player.0 + offset) % MahjongRuleSet::PLAYER_COUNT))
}
