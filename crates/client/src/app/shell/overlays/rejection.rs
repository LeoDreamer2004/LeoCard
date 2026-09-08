//! 将协议拒绝原因转换为客户端提示文本。

use leocard_protocol::{
    GameViolation, MahjongViolation, PlayerViolation, RejectReason, RoomViolation, RuleViolation,
    ShengjiViolation, TexasHoldemViolation, UnoViolation,
};

pub fn rejection_label(reason: &RejectReason) -> Option<String> {
    if matches!(
        reason,
        RejectReason::Game(GameViolation::QiGui523(RuleViolation::NotPlayersTurn,))
    ) {
        return None;
    }
    if let RejectReason::Player(PlayerViolation::NameTooLong { max_chars }) = reason {
        return Some(format!("提示：玩家名称不能超过 {max_chars} 个字符"));
    }
    if let RejectReason::Game(GameViolation::Mahjong(MahjongViolation::InvalidDeveloperHand(
        reason,
    ))) = reason
    {
        return Some(format!("提示：{reason}"));
    }
    let detail = match reason {
        RejectReason::Player(PlayerViolation::NameEmpty) => "玩家名称不能为空",
        RejectReason::Player(PlayerViolation::InvalidIdentityProof) => {
            "玩家身份签名无效，请重新生成或恢复玩家档案"
        }
        RejectReason::Player(PlayerViolation::InvalidAvatar) => "头像数据无效",
        RejectReason::Player(PlayerViolation::AvatarAlreadySet) => "本次连接已经上传过头像",
        RejectReason::Room(RoomViolation::InvalidSeat) => "座位编号无效",
        RejectReason::Room(RoomViolation::SeatTaken) => "这个座位已经有人了",
        RejectReason::Room(RoomViolation::MustSelectSeat) => "必须先选择座位",
        RejectReason::Room(RoomViolation::OnlyHostCanConfigure) => "只有房主可以修改游戏配置",
        RejectReason::Room(RoomViolation::OnlyHostCanStart) => "只有房主可以开始游戏",
        RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby) => "只有房主可以返回大厅",
        RejectReason::Room(RoomViolation::OnlyHostCanCloseRoom) => "只有房主可以关闭房间",
        RejectReason::Room(RoomViolation::NotEnoughPlayers { .. }) => "至少需要两名玩家才能开始",
        RejectReason::Room(RoomViolation::WaitingForPlayers { .. }) => "人数尚未到齐",
        RejectReason::Room(RoomViolation::PlayersNotReady { .. }) => "仍有玩家没有准备",
        RejectReason::Game(GameViolation::InvalidRuleConfiguration) => {
            "这组配置无法满足最多六名玩家的初始发牌"
        }
        RejectReason::Game(GameViolation::DeveloperFeatureUnavailable) => {
            "房主程序没有启用开发者功能"
        }
        RejectReason::Game(GameViolation::InvalidDeveloperHand) => "开发者手牌无效",
        RejectReason::Room(RoomViolation::InvalidChatMessage) => "聊天消息为空、过长或快捷语音无效",
        RejectReason::Game(GameViolation::GameAlreadyStarted) => "游戏已经开始",
        RejectReason::Game(GameViolation::GameNotFinished) => "游戏尚未结束",
        RejectReason::Game(GameViolation::QiGui523(violation)) => match violation {
            RuleViolation::NotPlayersTurn => unreachable!("filtered above"),
            RuleViolation::MustLeadWithCards => "领出时必须出牌",
            RuleViolation::CardNotInHand => "选择的牌不在手中",
            RuleViolation::InvalidPattern => "所选牌不能组成合法牌型",
            RuleViolation::PlayDoesNotBeatCurrent => "这手牌无法压过当前牌",
            RuleViolation::GameAlreadyFinished => "游戏已经结束",
            RuleViolation::InvalidPlayer => "玩家身份无效",
        },
        RejectReason::Game(GameViolation::TexasHoldem(violation)) => match violation {
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
        },
        RejectReason::Game(GameViolation::Shengji(violation)) => match violation {
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
            ShengjiViolation::WrongBuryCount { expected, .. } => {
                return Some(format!("提示：必须埋下 {expected} 张底牌"));
            }
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
        },
        RejectReason::Game(GameViolation::Uno(violation)) => match violation {
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
        },
        RejectReason::Game(GameViolation::Mahjong(violation)) => match violation {
            MahjongViolation::InvalidPlayer => "玩家身份无效",
            MahjongViolation::NotPlayersTurn => "还没有轮到你出牌",
            MahjongViolation::WrongPhase => "当前阶段不能执行这个操作",
            MahjongViolation::TileNotInHand => "这张牌不在你的手中",
            MahjongViolation::InvalidClaim => "当前不能这样吃、碰、杠或和",
            MahjongViolation::AlreadyResponded => "你已经响应过这张牌",
            MahjongViolation::CannotWin => "当前手牌不能和牌",
            MahjongViolation::CannotKong => "当前不能开杠",
            MahjongViolation::InvalidDeveloperHand(_) => unreachable!("formatted above"),
        },
        RejectReason::Game(GameViolation::WrongGame { .. }) => "该命令不属于当前房间游戏",
        _ => "请求被房主拒绝",
    };
    Some(format!("提示：{detail}"))
}
