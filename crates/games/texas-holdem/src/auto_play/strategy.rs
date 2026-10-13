use crate::{GameState, Phase, TexasHoldemAction, TexasHoldemPlayerId};

/// 保守德州扑克托管策略作出一次决定所需的最小公开状态。
///
/// 这个请求不包含手牌或其他玩家暗牌，托管策略独立于机器人策略维护。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassiveAutoPlayRequest {
    pub must_post_blind: bool,
    pub fold_allowed: bool,
    pub amount_to_call: u32,
}

/// 固定采用“能弃牌就弃牌，否则只投入最低所需筹码”的确定性策略。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PassiveAutoPlay;

impl PassiveAutoPlay {
    pub const fn choose(request: PassiveAutoPlayRequest) -> TexasHoldemAction {
        if request.must_post_blind {
            TexasHoldemAction::PostBlind
        } else if request.fold_allowed {
            TexasHoldemAction::Fold
        } else if request.amount_to_call > 0 {
            TexasHoldemAction::Call
        } else {
            TexasHoldemAction::Check
        }
    }

    /// 从权威规则状态生成请求并作出决定。不是该玩家的行动回合或牌局已经结束时
    /// 返回 `None`，且绝不修改传入状态。
    pub fn choose_for_game(
        game: &GameState,
        player: TexasHoldemPlayerId,
    ) -> Option<TexasHoldemAction> {
        if game.current_player() != Some(player) || matches!(game.phase(), Phase::Complete(_)) {
            return None;
        }
        let must_post_blind = game
            .blind_to_post()
            .is_some_and(|(blind_player, _, _)| blind_player == player);
        let amount_to_call = game.amount_to_call(player).ok()?;
        Some(Self::choose(PassiveAutoPlayRequest {
            must_post_blind,
            fold_allowed: !must_post_blind,
            amount_to_call,
        }))
    }
}
