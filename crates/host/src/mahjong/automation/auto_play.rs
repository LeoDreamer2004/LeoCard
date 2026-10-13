use super::action::AutomaticMahjongAction;
use leocard_mahjong::{
    GameState, MahjongClaim, MahjongClaimOption, MahjongMeldKind, MahjongPlayerId, MahjongTile,
    MahjongTileKind, Phase,
};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &GameState, player: MahjongPlayerId) -> Option<AutomaticMahjongAction> {
        choose_action(game, player)
    }
}

fn choose_action(game: &GameState, player: MahjongPlayerId) -> Option<AutomaticMahjongAction> {
    match game.phase() {
        Phase::WaitingForClaims(pending) => {
            let options = pending.options_for(player)?;
            if options.contains(&MahjongClaimOption::Win)
                && game.legal_claim_win_available(player).ok()?
            {
                return Some(AutomaticMahjongAction::Respond(MahjongClaim::Win));
            }
            let claim = if options.contains(&MahjongClaimOption::Kong) {
                MahjongClaim::Kong
            } else if options.contains(&MahjongClaimOption::Pung) {
                MahjongClaim::Pung
            } else if let Some(start) = options.iter().find_map(|option| match option {
                MahjongClaimOption::Chow { start } => Some(*start),
                _ => None,
            }) {
                MahjongClaim::Chow { start }
            } else {
                MahjongClaim::Pass
            };
            Some(AutomaticMahjongAction::Respond(claim))
        }
        Phase::Playing if game.current_player() == player => {
            if game.legal_self_draw_available(player).ok()? {
                return Some(AutomaticMahjongAction::SelfDraw);
            }
            let state = game.player(player)?;
            if game.wall_len() > 0
                && let Some(kind) = state.hand().iter().map(|tile| tile.kind()).find(|kind| {
                    state
                        .hand()
                        .iter()
                        .filter(|tile| tile.kind() == *kind)
                        .count()
                        == 4
                })
            {
                return Some(AutomaticMahjongAction::ConcealedKong(kind));
            }
            if game.wall_len() > 0
                && let Some(tile) = state.hand().iter().copied().find(|tile| {
                    state.melds().iter().any(|meld| {
                        meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind()
                    })
                })
            {
                return Some(AutomaticMahjongAction::AddedKong(tile));
            }
            automatic_discard(state.hand()).map(AutomaticMahjongAction::Discard)
        }
        Phase::Dealing { .. }
        | Phase::ReplacingFlower { .. }
        | Phase::Playing
        | Phase::Finished(_) => None,
    }
}

fn automatic_discard(hand: &[MahjongTile]) -> Option<MahjongTile> {
    hand.iter().copied().min_by_key(|tile| {
        let kind = tile.kind();
        let identical = hand
            .iter()
            .filter(|candidate| candidate.kind() == kind)
            .count()
            .saturating_sub(1) as u16;
        let neighbors = match kind {
            MahjongTileKind::Suited { suit, rank } => hand
                .iter()
                .filter_map(|candidate| match candidate.kind() {
                    MahjongTileKind::Suited {
                        suit: candidate_suit,
                        rank: candidate_rank,
                    } if candidate_suit == suit => Some(rank.abs_diff(candidate_rank)),
                    _ => None,
                })
                .map(|distance| match distance {
                    1 => 4,
                    2 => 2,
                    _ => 0,
                })
                .sum::<u16>(),
            MahjongTileKind::Wind(_) | MahjongTileKind::Dragon(_) | MahjongTileKind::Flower(_) => 0,
        };
        (identical * 7 + neighbors, kind, tile.copy())
    })
}
