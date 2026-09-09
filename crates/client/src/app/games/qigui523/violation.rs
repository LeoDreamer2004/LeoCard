use leocard_protocol::RuleViolation;

pub(crate) fn qigui523_violation_label(violation: &RuleViolation) -> Option<String> {
    Some(
        match violation {
            RuleViolation::NotPlayersTurn => return None,
            RuleViolation::MustLeadWithCards => "领出时必须出牌",
            RuleViolation::CardNotInHand => "选择的牌不在手中",
            RuleViolation::InvalidPattern => "所选牌不能组成合法牌型",
            RuleViolation::PlayDoesNotBeatCurrent => "这手牌无法压过当前牌",
            RuleViolation::GameAlreadyFinished => "游戏已经结束",
            RuleViolation::InvalidPlayer => "玩家身份无效",
        }
        .to_owned(),
    )
}
