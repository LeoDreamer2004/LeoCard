use super::MahjongBotAction;
use super::shanten::Counts;
use crate::{
    MahjongClaim, MahjongClaimOption, MahjongDiscard, MahjongGameState, MahjongMeld,
    MahjongMeldKind, MahjongPhase, MahjongPublicPlayerState, MahjongRuleSet, MahjongTile,
    MahjongTileKind, MahjongWinContext, MahjongWinSource, MahjongWind,
};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug, PartialEq)]
pub struct MahjongBotObservation {
    pub(super) player: PlayerId,
    pub(super) hand: Vec<MahjongTile>,
    pub(super) melds: Vec<MahjongMeld>,
    pub(super) public: Vec<MahjongPublicPlayerState>,
    pub(super) discards: Vec<MahjongDiscard>,
    pub(super) unseen: Counts,
    pub(super) options: Vec<MahjongBotAction>,
    pub(super) claim: Option<(PlayerId, MahjongTileKind)>,
    pub(super) rules: MahjongRuleSet,
    pub(super) context: MahjongWinContext,
    pub(super) wall: usize,
    pub(super) scores: [i32; 4],
}

impl MahjongBotObservation {
    /// Builds an information boundary: no wall order or opponent concealed tile is retained.
    pub fn from_game(game: &MahjongGameState, player: PlayerId) -> Option<Self> {
        let own = game.player(player)?;
        let mut options = Vec::new();
        let claim = match game.phase() {
            MahjongPhase::Playing if game.current_player() == player => {
                if game.legal_self_draw_available(player).ok()? {
                    options.push(MahjongBotAction::SelfDraw);
                }
                let mut added = [false; 34];
                for &tile in own.hand() {
                    let index = tile.kind().index34()?;
                    if added[index] {
                        continue;
                    }
                    added[index] = true;
                    options.push(MahjongBotAction::Discard(tile));
                    if game.wall_len() == 0 {
                        continue;
                    }
                    if own
                        .hand()
                        .iter()
                        .filter(|held| held.kind() == tile.kind())
                        .count()
                        == 4
                    {
                        options.push(MahjongBotAction::ConcealedKong(tile.kind()));
                    }
                    if own.melds().iter().any(|meld| {
                        meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind()
                    }) {
                        options.push(MahjongBotAction::AddedKong(tile));
                    }
                }
                None
            }
            MahjongPhase::WaitingForClaims(pending) if pending.response_from(player).is_none() => {
                options.push(MahjongBotAction::Respond(MahjongClaim::Pass));
                for option in pending.options_for(player)? {
                    let claim = match option {
                        MahjongClaimOption::Win
                            if game.legal_claim_win_available(player).ok()? =>
                        {
                            MahjongClaim::Win
                        }
                        MahjongClaimOption::Win => continue,
                        MahjongClaimOption::Kong => MahjongClaim::Kong,
                        MahjongClaimOption::Pung => MahjongClaim::Pung,
                        MahjongClaimOption::Chow { start } => MahjongClaim::Chow { start: *start },
                    };
                    options.push(MahjongBotAction::Respond(claim));
                }
                Some((pending.source_player(), pending.tile().kind()))
            }
            _ => return None,
        };
        let public: Vec<_> = game
            .players()
            .iter()
            .filter_map(|state| game.public_player(player, state.id()))
            .collect();
        let mut seen = [0u8; 34];
        for tile in own.hand() {
            seen[tile.kind().index34()?] += 1;
        }
        for state in &public {
            for meld in &state.melds {
                let Some(tile) = meld.tile else {
                    continue;
                };
                let index = tile.index34()?;
                match meld.kind {
                    MahjongMeldKind::Chow => {
                        for count in &mut seen[index..index + 3] {
                            *count += 1;
                        }
                    }
                    MahjongMeldKind::Pung => seen[index] += 3,
                    MahjongMeldKind::Kong(_) => seen[index] += 4,
                }
            }
            if state.id != player
                && let Some(hand) = &state.revealed_hand
            {
                for tile in hand {
                    seen[tile.kind().index34()?] += 1;
                }
            }
        }
        for discard in game
            .discards()
            .iter()
            .filter(|discard| discard.claimed_by.is_none())
        {
            seen[discard.tile.kind().index34()?] += 1;
        }
        Some(Self {
            player,
            hand: own.hand().to_vec(),
            melds: own.melds().to_vec(),
            public,
            discards: game.discards().to_vec(),
            unseen: seen.map(|count| 4u8.saturating_sub(count)),
            options,
            claim,
            rules: *game.rules(),
            context: MahjongWinContext {
                source: MahjongWinSource::SelfDraw,
                seat_wind: game.seat_wind(player).unwrap_or(MahjongWind::East),
                prevalent_wind: game.prevalent_wind(),
                last_wall_tile: game.wall_len() == 0,
                last_of_kind: false,
                flower_count: own.flowers().len() as u8,
            },
            wall: game.wall_len(),
            scores: *game.match_scores(),
        })
    }

    pub fn immediate_action(&self) -> Option<MahjongBotAction> {
        self.options.iter().copied().find(|action| {
            matches!(
                action,
                MahjongBotAction::SelfDraw | MahjongBotAction::Respond(MahjongClaim::Win)
            )
        })
    }

    pub fn fallback(&self) -> MahjongBotAction {
        self.options[0]
    }
}
