//! UNO 对局内选牌与目标选择状态。

use super::*;
use leocard_protocol::{PlayerId, UnoPendingSwapView};

pub fn toggle_uno_swap_target_selection(
    pending: Option<UnoPendingSwapView>,
    you: PlayerId,
    target: PlayerId,
    selected: &mut Vec<PlayerId>,
) {
    match pending {
        Some(UnoPendingSwapView::SwapOneTarget { player })
        | Some(UnoPendingSwapView::SevenSwap { player })
            if player == you && target != you =>
        {
            if selected.as_slice() == [target] {
                selected.clear();
            } else {
                selected.clear();
                selected.push(target);
            }
        }
        Some(UnoPendingSwapView::ForceTrade { player }) if player == you => {
            if let Some(index) = selected
                .iter()
                .position(|selected_target| *selected_target == target)
            {
                selected.remove(index);
            } else if selected.len() < 2 {
                selected.push(target);
            }
        }
        _ => {}
    }
}
