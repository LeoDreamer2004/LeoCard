use crate::facts::{ChatActivity, InteractionKind};
use crate::{AchievementTrigger, PersonalEvent};

pub(super) fn personal(
    trigger: &AchievementTrigger,
    predicate: impl FnOnce(&PersonalEvent) -> bool,
) -> u64 {
    match trigger {
        AchievementTrigger::Personal(event) => u64::from(predicate(event)),
        _ => 0,
    }
}

pub(super) fn received(trigger: &AchievementTrigger, kind: InteractionKind) -> u64 {
    let AchievementTrigger::Interaction { player, event } = trigger else {
        return 0;
    };
    if *player != event.target {
        return 0;
    }
    u64::from(match kind {
        InteractionKind::Flower => event.kind.flower_count(),
        InteractionKind::Egg => event.kind.egg_count(),
        InteractionKind::Wine => u32::from(event.kind == kind),
        InteractionKind::Shoe => u32::from(event.kind == kind),
    })
}

pub(super) fn purchased(trigger: &AchievementTrigger) -> u64 {
    personal(trigger, |event| {
        matches!(event, PersonalEvent::ItemPurchased { .. })
    })
}

pub(super) fn flowers_sent(trigger: &AchievementTrigger) -> u64 {
    match trigger {
        AchievementTrigger::Interaction { player, event } if *player == event.source => {
            u64::from(event.kind.flower_count())
        }
        _ => 0,
    }
}

pub(super) fn quick_voice(trigger: &AchievementTrigger) -> u64 {
    u64::from(
        matches!(trigger, AchievementTrigger::Chat { player, message } if *player == message.source && matches!(message.content, ChatActivity::QuickVoice)),
    )
}

pub(super) fn text_characters(trigger: &AchievementTrigger) -> u64 {
    let AchievementTrigger::Chat { player, message } = trigger else {
        return 0;
    };
    if *player != message.source {
        return 0;
    }
    match &message.content {
        ChatActivity::Text(text) => text.chars().count() as u64,
        _ => 0,
    }
}

pub(super) fn hosted(trigger: &AchievementTrigger) -> u64 {
    u64::from(
        matches!(trigger, AchievementTrigger::SessionStarted { player, host } if player == host),
    )
}
