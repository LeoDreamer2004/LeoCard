use leocard_protocol::UnoEvent;
use leocard_uno::{UnoActionStatistics, UnoPlayerStatistics, UnoProfileStats};

pub(super) fn analysis_events(statistics: Vec<UnoActionStatistics>) -> Vec<UnoEvent> {
    statistics
        .into_iter()
        .map(|statistics| UnoEvent::ActionAnalyzed {
            player: statistics.player,
            statistics: Box::new(statistics),
        })
        .collect()
}

pub(super) fn profile_statistics(progress: &UnoPlayerStatistics) -> UnoProfileStats {
    UnoProfileStats {
        max_hand_cards: progress.peak_hand,
        max_penalty_cards: progress.max_penalty_cards,
        max_skipped_turns: progress.max_skipped_turns,
        uno_calls: progress.uno_calls,
        uno_penalties: progress.uno_penalties,
        challenges: progress.challenges,
        successful_challenges: progress.successful_challenges,
        challenges_received: progress.challenges_received,
        successful_challenges_received: progress.successful_challenges_received,
        jump_in_opportunities: progress.jump_in_opportunities,
        successful_jump_ins: progress.jump_ins,
        ..Default::default()
    }
}
