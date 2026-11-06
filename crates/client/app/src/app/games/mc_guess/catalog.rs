use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_mc_guess::{McGuessMode, McGuessRuleSet};
use leocard_protocol::GameKind;

impl GameMetadata for McGuessRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::McGuess,
        title: "MC竞猜",
        description: "就你是 MC 高玩啊?",
    };
}

impl GameLobbyRules for McGuessRuleSet {
    fn seat_count(&self) -> u8 {
        McGuessRuleSet::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        (McGuessRuleSet::MIN_PLAYERS..=McGuessRuleSet::MAX_PLAYERS).contains(&count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            self.options().pool.label().to_owned(),
            self.mode.label().to_owned(),
            match self.mode {
                McGuessMode::TurnBased => "允许跳过".to_owned(),
                McGuessMode::Graph => format!("{} 个节点", self.graph.nodes),
                McGuessMode::Race => format!("{} 秒 / {} 局", self.race.seconds, self.race.games),
            },
        ]
    }
}
