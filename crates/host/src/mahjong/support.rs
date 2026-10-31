use leocard_mahjong::{
    MahjongActionOutcome, MahjongGameError, MahjongHandResult, MahjongTile, build_deck,
};
use leocard_protocol::{MahjongEvent, MahjongHandResultView};

pub(super) fn events_for_outcome(outcome: &MahjongActionOutcome) -> Vec<MahjongEvent> {
    match outcome {
        MahjongActionOutcome::Discarded { .. } | MahjongActionOutcome::ClaimRecorded { .. } => {
            Vec::new()
        }
        MahjongActionOutcome::Claimed {
            player,
            source,
            tile,
            claim,
        } => vec![MahjongEvent::ClaimResolved {
            player: *player,
            source: *source,
            tile: *tile,
            claim: *claim,
        }],
        MahjongActionOutcome::Drew { player, origin, .. } => vec![MahjongEvent::TileDrawn {
            player: *player,
            origin: *origin,
        }],
        MahjongActionOutcome::KongDeclared {
            player,
            tile,
            added,
        } => vec![MahjongEvent::KongDeclared {
            player: *player,
            tile: *tile,
            added: *added,
        }],
        MahjongActionOutcome::FalseWin { player, deltas } => vec![MahjongEvent::FalseWin {
            player: *player,
            deltas: *deltas,
        }],
        MahjongActionOutcome::HandFinished(result) => vec![MahjongEvent::HandFinished {
            result: hand_result_view(result),
        }],
    }
}

pub(super) fn hand_result_view(result: &MahjongHandResult) -> MahjongHandResultView {
    MahjongHandResultView {
        winners: result.winners.clone(),
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

pub(super) fn validate_deck(deck: &[MahjongTile]) -> Result<(), MahjongGameError> {
    if deck.len() != 144 {
        return Err(MahjongGameError::InvalidDeckSize {
            expected: 144,
            actual: deck.len(),
        });
    }
    let mut actual = deck.to_vec();
    actual.sort_unstable();
    let mut expected = build_deck();
    expected.sort_unstable();
    if actual != expected {
        return Err(MahjongGameError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn shuffled_deck() -> Vec<MahjongTile> {
    let mut deck = build_deck();
    fastrand::shuffle(&mut deck);
    deck
}
