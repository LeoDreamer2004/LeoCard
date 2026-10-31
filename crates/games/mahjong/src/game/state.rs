use super::{
    ClaimPriority, MahjongClaim, MahjongClaimOption, MahjongDiscard, MahjongDrawOrigin,
    MahjongGameError, MahjongMatchProgress, MahjongPhase, MahjongPlayerState, MahjongPublicMeld,
    MahjongPublicPlayerState,
};
use crate::{MahjongMeldKind, MahjongRuleSet, MahjongTile, MahjongWind, build_deck};
use leocard_game_common::PlayerId;

use leocard_game_common::validate_deck as validate_physical_deck;
use std::{array, collections::VecDeque};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MahjongGameState {
    pub(super) rules: MahjongRuleSet,
    pub(super) players: [MahjongPlayerState; MahjongRuleSet::PLAYER_COUNT as usize],
    pub(super) wall: VecDeque<MahjongTile>,
    pub(super) discards: Vec<MahjongDiscard>,
    pub(super) initial_dealer: PlayerId,
    pub(super) dealer: PlayerId,
    pub(super) prevalent_wind: MahjongWind,
    pub(super) sequence_index: u8,
    pub(super) match_progress: MahjongMatchProgress,
    pub(super) current_player: PlayerId,
    pub(super) last_drawn: Option<MahjongTile>,
    pub(super) draw_origin: MahjongDrawOrigin,
    pub(super) hand_deltas: [i32; MahjongRuleSet::PLAYER_COUNT as usize],
    pub(super) match_scores: [i32; MahjongRuleSet::PLAYER_COUNT as usize],
    pub(super) phase: MahjongPhase,
}

impl MahjongGameState {
    /// `deck[0]` 是牌墙前端第一张牌；补花和杠牌从牌墙尾端取牌。
    pub fn new_with_deck(
        rules: MahjongRuleSet,
        deck: Vec<MahjongTile>,
        initial_dealer: PlayerId,
    ) -> Result<Self, MahjongGameError> {
        let rules = rules.validate()?;
        validate_player(initial_dealer)?;
        validate_deck(&deck)?;
        let players = array::from_fn(|index| MahjongPlayerState {
            id: PlayerId::from_index(index),
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
            match_progress: MahjongMatchProgress::default(),
            current_player: initial_dealer,
            last_drawn: None,
            draw_origin: MahjongDrawOrigin::Normal,
            hand_deltas: [0; MahjongRuleSet::PLAYER_COUNT as usize],
            match_scores: [0; MahjongRuleSet::PLAYER_COUNT as usize],
            phase: MahjongPhase::Dealing { batch: 0 },
        };
        Ok(game)
    }

    pub const fn rules(&self) -> &MahjongRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[MahjongPlayerState; MahjongRuleSet::PLAYER_COUNT as usize] {
        &self.players
    }

    pub fn player(&self, player: PlayerId) -> Option<&MahjongPlayerState> {
        self.players.get(player.index())
    }

    pub fn public_player(
        &self,
        viewer: PlayerId,
        player: PlayerId,
    ) -> Option<MahjongPublicPlayerState> {
        validate_player(viewer).ok()?;
        let state = self.players.get(player.index())?;
        let settled = matches!(self.phase, MahjongPhase::Finished(_));
        let reveal_hand = viewer == player || state.hand_revealed;
        let reveal_concealed_kong = reveal_hand || settled;
        Some(MahjongPublicPlayerState {
            id: player,
            concealed_count: state.hand.len(),
            revealed_hand: reveal_hand.then(|| state.hand.clone()),
            melds: state
                .melds
                .iter()
                .map(|meld| MahjongPublicMeld {
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

    pub fn discards(&self) -> &[MahjongDiscard] {
        &self.discards
    }

    pub const fn dealer(&self) -> PlayerId {
        self.dealer
    }

    pub const fn prevalent_wind(&self) -> MahjongWind {
        self.prevalent_wind
    }

    pub const fn sequence_index(&self) -> u8 {
        self.sequence_index
    }

    pub const fn current_player(&self) -> PlayerId {
        self.current_player
    }

    pub const fn last_drawn(&self) -> Option<MahjongTile> {
        self.last_drawn
    }

    pub fn wall_len(&self) -> usize {
        self.wall.len()
    }

    pub const fn phase(&self) -> &MahjongPhase {
        &self.phase
    }

    pub const fn match_scores(&self) -> &[i32; MahjongRuleSet::PLAYER_COUNT as usize] {
        &self.match_scores
    }

    pub fn seat_wind(&self, player: PlayerId) -> Option<MahjongWind> {
        validate_player(player).ok()?;
        let distance = (player.index() + usize::from(MahjongRuleSet::PLAYER_COUNT)
            - self.dealer.index())
            % usize::from(MahjongRuleSet::PLAYER_COUNT);
        Some(MahjongWind::ALL[distance])
    }

    pub(super) fn ensure_playing_turn(&self, player: PlayerId) -> Result<(), MahjongGameError> {
        validate_player(player)?;
        if !matches!(self.phase, MahjongPhase::Playing) {
            return Err(MahjongGameError::WrongPhase);
        }
        if self.current_player != player {
            return Err(MahjongGameError::NotPlayersTurn {
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

pub(super) fn validate_player(player: PlayerId) -> Result<(), MahjongGameError> {
    (player.index() < usize::from(MahjongRuleSet::PLAYER_COUNT))
        .then_some(())
        .ok_or(MahjongGameError::InvalidPlayer(player))
}

pub(super) fn claim_priority(
    source: PlayerId,
    player: PlayerId,
    claim: MahjongClaim,
) -> ClaimPriority {
    let category = match claim {
        MahjongClaim::Win => 3,
        MahjongClaim::Pung | MahjongClaim::Kong => 2,
        MahjongClaim::Chow { .. } => 1,
        MahjongClaim::Pass => 0,
    };
    let distance = (player.index() + usize::from(MahjongRuleSet::PLAYER_COUNT) - source.index())
        % usize::from(MahjongRuleSet::PLAYER_COUNT);
    ClaimPriority {
        category,
        proximity: MahjongRuleSet::PLAYER_COUNT - distance as u8,
    }
}

pub(super) fn claim_option_priority(
    source: PlayerId,
    player: PlayerId,
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

pub(super) fn validate_deck(deck: &[MahjongTile]) -> Result<(), MahjongGameError> {
    validate_physical_deck(deck, &build_deck()).map_err(MahjongGameError::from)
}

pub(super) const fn next_player(player: PlayerId) -> PlayerId {
    PlayerId::from_index((player.index() + 1) % MahjongRuleSet::PLAYER_COUNT as usize)
}

pub(super) fn players_after(player: PlayerId) -> impl Iterator<Item = PlayerId> {
    (1..usize::from(MahjongRuleSet::PLAYER_COUNT)).map(move |offset| {
        PlayerId::from_index((player.index() + offset) % usize::from(MahjongRuleSet::PLAYER_COUNT))
    })
}
