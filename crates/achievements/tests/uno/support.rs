pub(super) use leocard_achievements::{AchievementBook, AchievementTrigger, achievement_by_id};
pub(super) use leocard_protocol::{GameEvent, PlayerId, UnoEvent};
pub(super) use leocard_uno::{
    Mode, UnoActionStatistics, UnoPlayerId, UnoPlayerStatistics, UnoRuleSet,
};

pub(super) fn statistics() -> UnoActionStatistics {
    UnoActionStatistics {
        player: UnoPlayerId(0),
        rules: UnoRuleSet::default(),
        progress: UnoPlayerStatistics::default(),
        wild_cards: 0,
        jump_ins: 0,
        hand_swaps: 0,
        completed_game: false,
        won: false,
        eliminated: false,
        dark_side: false,
        all_opponents_eliminated: false,
    }
}

pub(super) fn trigger(statistics: UnoActionStatistics) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(0),
        event: GameEvent::Uno(UnoEvent::ActionAnalyzed {
            player: PlayerId(0),
            statistics: Box::new(statistics),
        }),
    }
}

pub(super) fn amount(id: &str, stats: UnoActionStatistics) -> u64 {
    let definition = achievement_by_id(&format!("leocard:uno/{id}")).unwrap();
    (definition.criteria[0].amount)(&trigger(stats))
}
