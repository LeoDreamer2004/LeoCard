use super::time_control_label;
use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_protocol::GameKind;
use leocard_qigui523::QiGuiRuleSet;

impl GameMetadata for QiGuiRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::QiGui523,
        title: "七鬼五二三",
        description: "放空大脑, 有牌就出",
    };
}

impl GameLobbyRules for QiGuiRuleSet {
    fn seat_count(&self) -> u8 {
        QiGuiRuleSet::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        (2..=QiGuiRuleSet::MAX_PLAYERS).contains(&count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            format!("{}副", self.deck_count),
            format!("{}张", self.hand_size),
            time_control_label(self.time_control).to_owned(),
        ]
    }
}
