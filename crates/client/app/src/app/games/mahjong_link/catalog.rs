use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_mahjong_link::MahjongLinkRuleSet;
use leocard_protocol::GameKind;

impl GameMetadata for MahjongLinkRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::MahjongLink,
        title: "麻将连连看",
        description: "小孩子不懂推着玩的",
    };
}

impl GameLobbyRules for MahjongLinkRuleSet {
    fn seat_count(&self) -> u8 {
        MahjongLinkRuleSet::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        (MahjongLinkRuleSet::MIN_PLAYERS..=MahjongLinkRuleSet::MAX_PLAYERS).contains(&count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            format!(
                "{}~{} 人轮流",
                MahjongLinkRuleSet::MIN_PLAYERS,
                MahjongLinkRuleSet::MAX_PLAYERS
            ),
            "136 张麻将".to_owned(),
            "行端推牌".to_owned(),
        ]
    }
}
