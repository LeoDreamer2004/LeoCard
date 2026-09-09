use leocard_protocol::TexasHoldemViolation;

pub(crate) fn texas_holdem_violation_label(violation: &TexasHoldemViolation) -> String {
    match violation {
        TexasHoldemViolation::InvalidPlayer => "玩家身份无效",
        TexasHoldemViolation::NotPlayersTurn => "还没有轮到你行动",
        TexasHoldemViolation::HandAlreadyComplete => "这一手已经结束",
        TexasHoldemViolation::PlayerCannotAct => "弃牌、全下或离线玩家不能继续行动",
        TexasHoldemViolation::MustPostBlind => "当前只能下盲注",
        TexasHoldemViolation::NoBlindToPost => "当前没有需要下的盲注",
        TexasHoldemViolation::CannotCheckWhileFacingBet { .. } => "面对下注时不能过牌",
        TexasHoldemViolation::NothingToCall => "当前没有需要跟注的筹码",
        TexasHoldemViolation::RaiseMustExceedCurrentBet { .. } => "加注必须超过当前最高注",
        TexasHoldemViolation::RaiseBelowMinimum { .. } => "加注没有达到本轮最小额度",
        TexasHoldemViolation::RaiseExceedsStack { .. } => "加注额度超过了你的可用筹码",
        TexasHoldemViolation::RaiseNotReopened => "本轮下注尚未重新开放加注",
    }
    .to_owned()
}
