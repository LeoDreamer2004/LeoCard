use leocard_protocol::UnoViolation;

pub(crate) fn uno_violation_label(violation: &UnoViolation) -> String {
    match violation {
        UnoViolation::InvalidPlayer => "玩家身份无效",
        UnoViolation::PlayerEliminated => "你已经被淘汰，不能继续操作",
        UnoViolation::NotPlayersTurn => "还没有轮到你行动",
        UnoViolation::GameAlreadyFinished => "游戏已经结束",
        UnoViolation::InitialColorChoiceRequired => "请先为起始万能牌选择颜色",
        UnoViolation::InitialColorAlreadyChosen => "当前不需要选择起始颜色",
        UnoViolation::CardNotInHand => "这张牌不在你的手中",
        UnoViolation::CardDoesNotMatch => "这张牌与当前颜色、数字或符号不匹配",
        UnoViolation::ColorRequired => "万能牌必须选择后续颜色",
        UnoViolation::UnexpectedColor => "普通牌不能指定后续颜色",
        UnoViolation::MustPlayDrawnCard => "摸牌后只能打出刚摸到的牌",
        UnoViolation::MustResolveDrawPenalty => "请先叠加、质疑或接受累计罚牌",
        UnoViolation::NoDrawPenalty => "当前没有待结算的罚牌",
        UnoViolation::CannotStack => "这张牌不能叠加到当前罚牌上",
        UnoViolation::CannotChallenge => "当前没有可质疑的万能摸四",
        UnoViolation::MustDrawBeforePassing => "必须先摸牌，才能结束回合",
        UnoViolation::MustResolveSkip => "请先叠加禁手或接受累计禁手",
        UnoViolation::NoSkipToResolve => "当前没有待结算的禁手",
        UnoViolation::UnoCalloutDisabled => "本房间没有启用 UNO 宣告与检举",
        UnoViolation::CannotCallUno => "你当前不能宣告 UNO",
        UnoViolation::MustPlayAfterUno => "喊出 UNO 后，本回合必须出牌至只剩一张",
        UnoViolation::CannotReportSelf => "不能检举自己",
        UnoViolation::PlayerNotReportable => "该玩家当前不可被检举",
        UnoViolation::CannotPlayTogether => "这些牌当前不能一次打出",
        UnoViolation::CannotJumpIn => "抢出窗口已经关闭",
        UnoViolation::DrawPileExhausted => "摸牌堆已经耗尽",
        UnoViolation::MustResolveSwapEffect => "请先完成当前换牌效果",
        UnoViolation::NoSwapEffect => "当前没有待处理的换牌效果",
        UnoViolation::InvalidSwapTargets => "请选择符合要求且互不重复的玩家",
    }
    .to_owned()
}
