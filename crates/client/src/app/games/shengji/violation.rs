use leocard_protocol::ShengjiViolation;

pub(crate) fn shengji_violation_label(violation: &ShengjiViolation) -> String {
    match violation {
        ShengjiViolation::WrongBuryCount { expected, .. } => {
            return format!("必须埋下 {expected} 张底牌");
        }
        ShengjiViolation::InvalidPlayer => "玩家身份无效",
        ShengjiViolation::WrongPhase => "当前阶段不能执行这个操作",
        ShengjiViolation::InvalidDeclaration => "这些牌不能用于亮主或反主",
        ShengjiViolation::DeclarationCardsNotOwned => "亮出的牌不全在你的手中",
        ShengjiViolation::CounterRequiresPair => "反主必须亮出规则要求的同张牌",
        ShengjiViolation::CounterNotStronger => "只能用更强的主牌反主",
        ShengjiViolation::ProtectedSuitCanOnlyBeCounteredByNoTrump => {
            "自保后只能用无主或更多级牌反主"
        }
        ShengjiViolation::DeclarationRequiresJoker => "带王亮必须同时亮出对应颜色的王",
        ShengjiViolation::NoTrumpCannotOpen => "带王亮时无主只能用于反主",
        ShengjiViolation::NotDealer => "只有庄家可以埋底",
        ShengjiViolation::CardsNotOwned => "选择的牌不全在你的手中",
        ShengjiViolation::CrossingNotEligible => "你本局不符合五主过江条件",
        ShengjiViolation::CrossingAlreadyDecided => "你已经完成过江选择",
        ShengjiViolation::WrongCrossingCount { .. } => "五主过江和归还都必须正好选择五张牌",
        ShengjiViolation::CrossingMustIncludeAllTrumps => "过江牌必须包含你当前的全部主牌",
        ShengjiViolation::CrossingReturnNotRequired => "当前不需要你归还过江牌",
        ShengjiViolation::CrossingAlreadyReturned => "你已经归还过江牌",
        ShengjiViolation::NotBottomCopyPlayer => "当前没有轮到你抄底或重新埋底",
        ShengjiViolation::NotPlayersTurn => "还没有轮到你出牌",
        ShengjiViolation::MustLeadWithCards => "领出时必须出牌",
        ShengjiViolation::ThrowDisabled => "本房间不允许甩牌",
        ShengjiViolation::InvalidPattern => "所选牌不能组成合法牌型",
        ShengjiViolation::WrongCardCount { .. } => "跟牌张数必须与首家相同",
        ShengjiViolation::MustFollowCategory => "手中有该门牌时必须先跟该门",
        ShengjiViolation::MustFollowStructure => "必须优先跟泰坦尼克、拖拉机、三同张或对子结构",
    }
    .to_owned()
}
