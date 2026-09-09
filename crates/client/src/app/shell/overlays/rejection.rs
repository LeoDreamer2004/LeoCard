//! 将通用协议拒绝原因转换为客户端提示文本。

use crate::app::games::game_violation_label;
use leocard_protocol::{GameViolation, PlayerViolation, RejectReason, RoomViolation};

pub(crate) fn rejection_label(reason: &RejectReason) -> Option<String> {
    if let RejectReason::Game(violation) = reason {
        return game_violation_label(violation).map(|detail| format!("提示：{detail}"));
    }
    let detail = match reason {
        RejectReason::Player(PlayerViolation::NameTooLong { max_chars }) => {
            return Some(format!("提示：玩家名称不能超过 {max_chars} 个字符"));
        }
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
        RejectReason::Room(RoomViolation::InvalidChatMessage) => "聊天消息为空、过长或快捷语音无效",
        RejectReason::Game(GameViolation::InvalidRuleConfiguration) => {
            "这组配置无法满足最多六名玩家的初始发牌"
        }
        RejectReason::Game(GameViolation::DeveloperFeatureUnavailable) => {
            "房主程序没有启用开发者功能"
        }
        RejectReason::Game(GameViolation::InvalidDeveloperHand) => "开发者手牌无效",
        RejectReason::Game(GameViolation::GameAlreadyStarted) => "游戏已经开始",
        RejectReason::Game(GameViolation::GameNotStarted) => "游戏尚未开始",
        RejectReason::Game(GameViolation::GameNotFinished) => "游戏尚未结束",
        RejectReason::Game(GameViolation::WrongGame { .. }) => "该命令不属于当前房间游戏",
        RejectReason::Game(
            GameViolation::QiGui523(_)
            | GameViolation::TexasHoldem(_)
            | GameViolation::Shengji(_)
            | GameViolation::Uno(_)
            | GameViolation::Mahjong(_),
        ) => unreachable!("game-specific violations are handled above"),
        _ => "请求被房主拒绝",
    };
    Some(format!("提示：{detail}"))
}
