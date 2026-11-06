use leocard_protocol::GameRules;

/// Capacity and short lobby labels belong to the game's concrete rule set.
pub(crate) trait GameLobbyRules {
    fn seat_count(&self) -> u8;

    fn accepts_player_count(&self, count: u8) -> bool;

    fn lobby_rule_labels(&self) -> [String; 3];
}

impl GameLobbyRules for GameRules {
    fn seat_count(&self) -> u8 {
        match self {
            Self::QiGui523(rules) => rules.seat_count(),
            Self::Texas(rules) => rules.seat_count(),
            Self::Shengji(rules) => rules.seat_count(),
            Self::Uno(rules) => rules.seat_count(),
            Self::Mahjong(rules) => rules.seat_count(),
            Self::McGuess(rules) => rules.seat_count(),
            Self::Blackjack(rules) => rules.seat_count(),
            Self::Doudizhu(rules) => rules.seat_count(),
            Self::Guandan(rules) => rules.seat_count(),
            Self::MahjongLink(rules) => rules.seat_count(),
        }
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        match self {
            Self::QiGui523(rules) => rules.accepts_player_count(count),
            Self::Texas(rules) => rules.accepts_player_count(count),
            Self::Shengji(rules) => rules.accepts_player_count(count),
            Self::Uno(rules) => rules.accepts_player_count(count),
            Self::Mahjong(rules) => rules.accepts_player_count(count),
            Self::McGuess(rules) => rules.accepts_player_count(count),
            Self::Blackjack(rules) => rules.accepts_player_count(count),
            Self::Doudizhu(rules) => rules.accepts_player_count(count),
            Self::Guandan(rules) => rules.accepts_player_count(count),
            Self::MahjongLink(rules) => rules.accepts_player_count(count),
        }
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        match self {
            Self::QiGui523(rules) => rules.lobby_rule_labels(),
            Self::Texas(rules) => rules.lobby_rule_labels(),
            Self::Shengji(rules) => rules.lobby_rule_labels(),
            Self::Uno(rules) => rules.lobby_rule_labels(),
            Self::Mahjong(rules) => rules.lobby_rule_labels(),
            Self::McGuess(rules) => rules.lobby_rule_labels(),
            Self::Blackjack(rules) => rules.lobby_rule_labels(),
            Self::Doudizhu(rules) => rules.lobby_rule_labels(),
            Self::Guandan(rules) => rules.lobby_rule_labels(),
            Self::MahjongLink(rules) => rules.lobby_rule_labels(),
        }
    }
}
