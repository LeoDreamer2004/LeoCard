//! 双升高频快照的轻量差异判断。

use leocard_protocol::ShengjiPhaseView;
use leocard_protocol::ShengjiSnapshot;

pub fn only_shengji_transient_progress_changed(
    before: Option<&ShengjiSnapshot>,
    after: Option<&ShengjiSnapshot>,
) -> bool {
    let (Some(before), Some(after)) = (before, after) else {
        return false;
    };
    match (&before.phase, &after.phase) {
        (ShengjiPhaseView::Dealing { .. }, ShengjiPhaseView::Dealing { .. }) => {
            if before.your_hand != after.your_hand {
                return false;
            }
            let mut normalized = before.clone();
            normalized.phase = after.phase.clone();
            for player in &mut normalized.players {
                if let Some(latest) = after.players.iter().find(|latest| latest.id == player.id) {
                    player.hand_len = latest.hand_len;
                }
            }
            normalized == *after
        }
        (ShengjiPhaseView::BiddingGrace { .. }, ShengjiPhaseView::BiddingGrace { .. }) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        (ShengjiPhaseView::BottomCopying { .. }, ShengjiPhaseView::BottomCopying { .. }) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::BottomCopying {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::BottomCopying {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        _ => false,
    }
}
