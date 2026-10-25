use leocard_mahjong::{
    ActionOutcome, GameError, HandResult, MahjongPlayerId, MahjongTile, build_deck,
};
use leocard_protocol::{MahjongEvent, MahjongHandResultView, MahjongWinView, PlayerId};

pub(super) fn events_for_outcome(outcome: &ActionOutcome) -> Vec<MahjongEvent> {
    match outcome {
        ActionOutcome::Discarded { .. } | ActionOutcome::ClaimRecorded { .. } => Vec::new(),
        ActionOutcome::Claimed {
            player,
            source,
            tile,
            claim,
        } => vec![MahjongEvent::ClaimResolved {
            player: from_core_player(*player),
            source: from_core_player(*source),
            tile: *tile,
            claim: *claim,
        }],
        ActionOutcome::Drew { player, origin, .. } => vec![MahjongEvent::TileDrawn {
            player: from_core_player(*player),
            origin: *origin,
        }],
        ActionOutcome::KongDeclared {
            player,
            tile,
            added,
        } => vec![MahjongEvent::KongDeclared {
            player: from_core_player(*player),
            tile: *tile,
            added: *added,
        }],
        ActionOutcome::FalseWin { player, deltas } => vec![MahjongEvent::FalseWin {
            player: from_core_player(*player),
            deltas: *deltas,
        }],
        ActionOutcome::HandFinished(result) => vec![MahjongEvent::HandFinished {
            result: hand_result_view(result),
        }],
    }
}

pub(super) fn hand_result_view(result: &HandResult) -> MahjongHandResultView {
    MahjongHandResultView {
        winners: result
            .winners
            .iter()
            .map(|winner| MahjongWinView {
                player: from_core_player(winner.player),
                from: winner.from.map(from_core_player),
                concealed_hand: winner.concealed_hand,
                winning_tile: winner.winning_tile,
                score: winner.score.clone(),
                wait_kind_count: winner.wait_kind_count,
            })
            .collect(),
        exhaustive_draw: result.exhaustive_draw,
        match_length: result.match_length,
        match_progress: result.match_progress,
        deltas: result.deltas,
        match_scores: result.match_scores,
        match_complete: result.match_complete,
        sequence_index: result.sequence_index,
        reference_changes: Vec::new(),
    }
}

pub(super) fn validate_deck(deck: &[MahjongTile]) -> Result<(), GameError> {
    if deck.len() != 144 {
        return Err(GameError::InvalidDeckSize {
            expected: 144,
            actual: deck.len(),
        });
    }
    let mut actual = deck.to_vec();
    actual.sort_unstable();
    let mut expected = build_deck();
    expected.sort_unstable();
    if actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn shuffled_deck() -> Vec<MahjongTile> {
    let mut deck = build_deck();
    fastrand::shuffle(&mut deck);
    deck
}

pub(super) const fn to_core_player(player: PlayerId) -> MahjongPlayerId {
    MahjongPlayerId(player.0 as usize)
}

pub(super) const fn from_core_player(player: MahjongPlayerId) -> PlayerId {
    PlayerId(player.0 as u8)
}
