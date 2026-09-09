use leocard_protocol::MahjongViolation;

pub(crate) fn mahjong_violation_label(violation: &MahjongViolation) -> String {
    match violation {
        MahjongViolation::InvalidDeveloperHand(reason) => return reason.to_string(),
        MahjongViolation::InvalidPlayer => "玩家身份无效",
        MahjongViolation::NotPlayersTurn => "还没有轮到你出牌",
        MahjongViolation::WrongPhase => "当前阶段不能执行这个操作",
        MahjongViolation::TileNotInHand => "这张牌不在你的手中",
        MahjongViolation::InvalidClaim => "当前不能这样吃、碰、杠或和",
        MahjongViolation::AlreadyResponded => "你已经响应过这张牌",
        MahjongViolation::CannotWin => "当前手牌不能和牌",
        MahjongViolation::CannotKong => "当前不能开杠",
    }
    .to_owned()
}
