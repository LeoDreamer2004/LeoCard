use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_protocol::GameKind;
use leocard_uno::UnoRuleSet;

impl GameMetadata for UnoRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Uno,
        title: "UNO",
        description: "最后一张，记得喊 UNO!",
    };
}

impl GameLobbyRules for UnoRuleSet {
    fn seat_count(&self) -> u8 {
        UnoRuleSet::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        (UnoRuleSet::MIN_PLAYERS..=UnoRuleSet::MAX_PLAYERS).contains(&count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        if self.is_no_mercy() {
            [
                "No Mercy".to_owned(),
                "+2 至 +10 递增堆叠".to_owned(),
                if self.no_mercy.mercy_elimination {
                    "25 张淘汰".to_owned()
                } else {
                    "不启用慈悲淘汰".to_owned()
                },
            ]
        } else if self.is_flip() {
            [
                "UNO FLIP".to_owned(),
                if self.flip.random_pairing {
                    "随机双面配对".to_owned()
                } else {
                    "固定双面配对".to_owned()
                },
                if self.flip.action_stacking {
                    "功能牌可堆叠".to_owned()
                } else {
                    "功能牌不堆叠".to_owned()
                },
            ]
        } else {
            [
                if self.action_stacking {
                    "功能牌可堆叠".to_owned()
                } else {
                    "功能牌不堆叠".to_owned()
                },
                if self.jump_in {
                    "允许抢出".to_owned()
                } else {
                    "不抢出".to_owned()
                },
                if self.uno_callout {
                    "UNO 检举".to_owned()
                } else {
                    "不检举".to_owned()
                },
            ]
        }
    }
}
