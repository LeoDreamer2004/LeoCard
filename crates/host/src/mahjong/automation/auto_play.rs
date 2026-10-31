use leocard_game_common::PlayerId;
use leocard_mahjong::{
    MahjongBotAction, MahjongClaim, MahjongClaimOption, MahjongGameState, MahjongMeldKind,
    MahjongPhase, MahjongTile, MahjongTileKind,
};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &MahjongGameState, player: PlayerId) -> Option<MahjongBotAction> {
        choose_action(game, player)
    }
}

fn choose_action(game: &MahjongGameState, player: PlayerId) -> Option<MahjongBotAction> {
    match game.phase() {
        MahjongPhase::WaitingForClaims(pending) => {
            let options = pending.options_for(player)?;
            if options.contains(&MahjongClaimOption::Win)
                && game.legal_claim_win_available(player).ok()?
            {
                return Some(MahjongBotAction::Respond(MahjongClaim::Win));
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
            Some(MahjongBotAction::Respond(claim))
        }
        MahjongPhase::Playing if game.current_player() == player => {
            if game.legal_self_draw_available(player).ok()? {
                return Some(MahjongBotAction::SelfDraw);
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
                return Some(MahjongBotAction::ConcealedKong(kind));
            }
            if game.wall_len() > 0
                && let Some(tile) = state.hand().iter().copied().find(|tile| {
                    state.melds().iter().any(|meld| {
                        meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind()
                    })
                })
            {
                return Some(MahjongBotAction::AddedKong(tile));
            }
            automatic_discard(state.hand()).map(MahjongBotAction::Discard)
        }
        MahjongPhase::Dealing { .. }
        | MahjongPhase::ReplacingFlower { .. }
        | MahjongPhase::Playing
        | MahjongPhase::Finished(_) => None,
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
