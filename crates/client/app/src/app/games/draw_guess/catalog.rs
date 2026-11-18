use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_draw_guess::DrawGuessRuleSet;
use leocard_protocol::GameKind;

impl GameMetadata for DrawGuessRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::DrawGuess,
        title: "你画我猜",
        description: "灵魂画手",
    };
}

impl GameLobbyRules for DrawGuessRuleSet {
    fn seat_count(&self) -> u8 {
        Self::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        (Self::MIN_PLAYERS..=Self::MAX_PLAYERS).contains(&count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            "竞猜".to_owned(),
            format!("{} 轮", self.rounds),
            format!("{} 秒", self.drawing_seconds),
        ]
    }
}
