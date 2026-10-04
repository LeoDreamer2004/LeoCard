pub(super) use leocard_achievements::{
    AchievementBook, AchievementDefinition, AchievementTrigger, PersonalEvent, achievement_by_id,
};
pub(super) use leocard_protocol::{
    ChatContent, ChatMessage, PlayerId, PlayerInteraction, PlayerInteractionKind,
};

pub(super) fn definition(id: &str) -> &'static AchievementDefinition {
    achievement_by_id(&format!("leocard:personal/{id}")).unwrap()
}

pub(super) fn amount(id: &str, trigger: &AchievementTrigger) -> u64 {
    (definition(id).criteria[0].amount)(trigger)
}

pub(super) fn interaction(
    kind: PlayerInteractionKind,
    source: u8,
    target: u8,
    elapsed_millis: u64,
) -> PlayerInteraction {
    PlayerInteraction {
        source: PlayerId(source),
        target: PlayerId(target),
        kind,
        seed: 1,
        elapsed_millis,
    }
}

pub(super) fn social(kind: PlayerInteractionKind, source: u8, target: u8) -> AchievementTrigger {
    AchievementTrigger::Interaction {
        player: PlayerId(0),
        event: interaction(kind, source, target, 0),
    }
}

pub(super) fn chat(source: u8, content: ChatContent) -> AchievementTrigger {
    AchievementTrigger::Chat {
        player: PlayerId(0),
        message: ChatMessage {
            source: PlayerId(source),
            content,
        },
    }
}
