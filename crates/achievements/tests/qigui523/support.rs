pub(super) use leocard_achievements::{AchievementBook, AchievementTrigger, achievement_by_id};
pub(super) use leocard_protocol::{GameEvent, PlayerId, QiGui523Event};
pub(super) use leocard_qigui523::{
    QiGuiActionStatistics, QiGuiPlayerId, QiGuiPlayerStatistics, QiGuiRuleSet,
};

pub(super) fn statistics() -> QiGuiActionStatistics {
    QiGuiActionStatistics {
        player: QiGuiPlayerId(0),
        rules: QiGuiRuleSet::default(),
        progress: QiGuiPlayerStatistics::default(),
        played_kind: None,
        played_count: 0,
        diamond_four: false,
        spade_seven_follow: false,
        heaven_over_heaven: false,
        late_bomb: false,
        straight_from_four: false,
        full_hand_play: false,
        bomb_revenge: false,
        completed_game: false,
        won: false,
        all_points: false,
        comeback: false,
        uncontested_win: false,
        captured_fives: 0,
        captured_tens_and_kings: 0,
        captured_points: 0,
    }
}

pub(super) fn trigger(statistics: QiGuiActionStatistics) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(0),
        event: GameEvent::QiGui523(QiGui523Event::ActionAnalyzed {
            player: PlayerId(0),
            statistics: Box::new(statistics),
        }),
    }
}

pub(super) fn amount(id: &str, stats: QiGuiActionStatistics) -> u64 {
    let definition = achievement_by_id(&format!("leocard:qigui523/{id}")).unwrap();
    (definition.criteria[0].amount)(&trigger(stats))
}
