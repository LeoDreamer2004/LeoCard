use crate::AchievementTrigger;
use leocard_protocol::{
    GameEvent, PlayerId, ShengjiDeclarationView, ShengjiEvent, ShengjiPublicPlay,
};
use leocard_shengji::{
    Component, ShengjiBidKind, ShengjiHandStatistics, ShengjiMatchStatistics, ShengjiRedealReason,
};

fn event(trigger: &AchievementTrigger) -> Option<(PlayerId, &ShengjiEvent)> {
    match trigger {
        AchievementTrigger::Game {
            player,
            event: GameEvent::Shengji(event),
        } => Some((*player, event)),
        _ => None,
    }
}

pub(super) fn hand(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&ShengjiHandStatistics) -> bool,
) -> u64 {
    analyzed(trigger, |facts, _| predicate(facts))
}

pub(super) fn analyzed(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&ShengjiHandStatistics, &ShengjiMatchStatistics) -> bool,
) -> u64 {
    let Some((
        player,
        ShengjiEvent::HandAnalyzed {
            player: actor,
            statistics,
            match_statistics,
        },
    )) = event(trigger)
    else {
        return 0;
    };
    u64::from(
        player == *actor
            && player.0 == statistics.player.0
            && predicate(statistics, match_statistics),
    )
}

pub(super) fn declaration(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&ShengjiDeclarationView) -> bool,
) -> u64 {
    let Some((player, event)) = event(trigger) else {
        return 0;
    };
    let (ShengjiEvent::DeclarationChanged { declaration }
    | ShengjiEvent::BottomCopied { declaration }) = event
    else {
        return 0;
    };
    u64::from(player == declaration.player && predicate(declaration))
}

pub(super) fn counter(trigger: &AchievementTrigger) -> u64 {
    declaration(trigger, |facts| {
        matches!(
            facts.kind,
            ShengjiBidKind::Counter | ShengjiBidKind::SelfCounter
        )
    })
}

pub(super) fn copy_bottom(trigger: &AchievementTrigger) -> u64 {
    u64::from(
        matches!(event(trigger), Some((player, ShengjiEvent::BottomCopied { declaration })) if player == declaration.player),
    )
}

pub(super) fn play(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&ShengjiPublicPlay, bool) -> bool,
) -> u64 {
    let Some((player, ShengjiEvent::CardsPlayed { play, is_lead })) = event(trigger) else {
        return 0;
    };
    u64::from(player == play.player && predicate(play, *is_lead))
}

pub(super) fn thrown_over(trigger: &AchievementTrigger, count: usize) -> u64 {
    play(trigger, |facts, is_lead| {
        is_lead && facts.play.is_throw() && facts.play.cards.len() > count
    })
}

pub(super) fn completed(trigger: &AchievementTrigger, deck_count: u8) -> u64 {
    hand(trigger, |facts| facts.deck_count == deck_count)
}

pub(super) fn attacking(facts: &ShengjiHandStatistics) -> bool {
    facts.player.team() == facts.result.collecting_team
}

pub(super) fn defending(facts: &ShengjiHandStatistics) -> bool {
    facts.player.team() == facts.result.dealer_team
}

pub(super) fn attack_promotes(facts: &ShengjiHandStatistics, steps: u8) -> bool {
    attacking(facts)
        && facts.result.promoted_team == facts.result.collecting_team
        && facts.result.promoted_steps >= steps
}

pub(super) fn shutout(facts: &ShengjiHandStatistics) -> bool {
    defending(facts) && facts.result.collecting_score == 0
}

pub(super) fn captured_with_structure(facts: &ShengjiHandStatistics) -> bool {
    attacking(facts)
        && facts.player == facts.last_trick_winner
        && facts.result.kitty_multiplier > 0
        && facts.result.kitty_points > 0
        && facts.last_winning_play.components.iter().any(|component| {
            matches!(
                component,
                Component::Quad { .. }
                    | Component::Tractor { .. }
                    | Component::Titanic { .. }
                    | Component::Spaceship { .. }
            )
        })
}

pub(super) fn exhausted_redeal(trigger: &AchievementTrigger) -> u64 {
    u64::from(matches!(
        event(trigger),
        Some((
            _,
            ShengjiEvent::RedealRequired {
                reason: ShengjiRedealReason::BottomFlipExhausted {
                    power_outage_used: true
                }
            }
        ))
    ))
}
