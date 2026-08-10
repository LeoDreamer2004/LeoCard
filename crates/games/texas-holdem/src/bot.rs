use crate::{Action, GameState, Phase, PlayerId};

/// 保守德州扑克机器人作出一次决定所需的最小公开状态。
///
/// 这个请求不包含手牌或其他玩家暗牌，因此同一策略可复用于房主托管、客户端提示
/// 和以后只持有协议快照的机器人。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassiveBotRequest {
    pub must_post_blind: bool,
    pub fold_allowed: bool,
    pub amount_to_call: u32,
}

/// 固定采用“能弃牌就弃牌，否则只投入最低所需筹码”的确定性策略。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PassiveBot;

impl PassiveBot {
    pub const fn choose(request: PassiveBotRequest) -> Action {
        if request.must_post_blind {
            Action::PostBlind
        } else if request.fold_allowed {
            Action::Fold
        } else if request.amount_to_call > 0 {
            Action::Call
        } else {
            Action::Check
        }
    }

    /// 从权威规则状态生成请求并作出决定。不是该玩家的行动回合或牌局已经结束时
    /// 返回 `None`，且绝不修改传入状态。
    pub fn choose_for_game(game: &GameState, player: PlayerId) -> Option<Action> {
        if game.current_player() != Some(player) || matches!(game.phase(), Phase::Complete(_)) {
            return None;
        }
        let must_post_blind = game
            .blind_to_post()
            .is_some_and(|(blind_player, _, _)| blind_player == player);
        let amount_to_call = game.amount_to_call(player).ok()?;
        Some(Self::choose(PassiveBotRequest {
            must_post_blind,
            fold_allowed: !must_post_blind,
            amount_to_call,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuleSet, build_deck};

    #[test]
    fn abstract_policy_folds_first_then_calls_or_checks_for_the_minimum() {
        assert_eq!(
            PassiveBot::choose(PassiveBotRequest {
                must_post_blind: false,
                fold_allowed: true,
                amount_to_call: 20,
            }),
            Action::Fold
        );
        assert_eq!(
            PassiveBot::choose(PassiveBotRequest {
                must_post_blind: false,
                fold_allowed: false,
                amount_to_call: 2,
            }),
            Action::Call
        );
        assert_eq!(
            PassiveBot::choose(PassiveBotRequest {
                must_post_blind: false,
                fold_allowed: false,
                amount_to_call: 0,
            }),
            Action::Check
        );
    }

    #[test]
    fn authoritative_game_adapter_posts_blinds_then_folds() {
        let mut game =
            GameState::new_with_deck(RuleSet::default(), PlayerId(0), build_deck(false)).unwrap();

        let small = game.current_player().unwrap();
        let action = PassiveBot::choose_for_game(&game, small).unwrap();
        assert_eq!(action, Action::PostBlind);
        game.act(small, action).unwrap();

        let big = game.current_player().unwrap();
        let action = PassiveBot::choose_for_game(&game, big).unwrap();
        assert_eq!(action, Action::PostBlind);
        game.act(big, action).unwrap();

        let first = game.current_player().unwrap();
        assert_eq!(
            PassiveBot::choose_for_game(&game, first),
            Some(Action::Fold)
        );
        assert_eq!(PassiveBot::choose_for_game(&game, PlayerId(9)), None);
    }
}
