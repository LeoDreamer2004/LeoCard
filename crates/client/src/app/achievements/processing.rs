use super::{
    AchievementRecipient, AchievementSession, AchievementUnlocked, LocalAchievementTrigger,
};
use crate::app::runtime::ServerNotification;
use bevy::prelude::*;
use leocard_achievements::{
    AchievementContext, AchievementTrigger, PersonalActivity, achievement_by_id,
};
use leocard_client::{LocalPlayerProfile, PlayerAchievements};
use leocard_protocol::ServerEvent;

#[derive(Resource, Default)]
pub(super) struct ActivityObserver {
    epoch: u64,
    activity: PersonalActivity,
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct TriggerResources<'w> {
    profile: Res<'w, LocalPlayerProfile>,
    book: ResMut<'w, PlayerAchievements>,
    session: ResMut<'w, AchievementSession>,
    observer: ResMut<'w, ActivityObserver>,
}

pub(super) fn process_triggers(
    mut notifications: MessageReader<ServerNotification>,
    mut local: MessageReader<LocalAchievementTrigger>,
    mut state: TriggerResources,
    mut unlocked: MessageWriter<AchievementUnlocked>,
) {
    let state = &mut state;
    if state.observer.epoch != state.session.connection_epoch {
        state.observer.epoch = state.session.connection_epoch;
        state.observer.activity = PersonalActivity::default();
    }
    for notification in notifications.read() {
        let trigger = match (&notification.event, notification.player) {
            (ServerEvent::GameEvent(event), Some(player)) => Some(AchievementTrigger::Game {
                player,
                event: event.clone(),
            }),
            (ServerEvent::PlayerInteraction(event), Some(player)) => {
                Some(AchievementTrigger::Interaction {
                    player,
                    event: *event,
                })
            }
            (ServerEvent::ChatMessage(message), Some(player)) => Some(AchievementTrigger::Chat {
                player,
                message: message.clone(),
            }),
            (ServerEvent::GameStarted { host, .. }, Some(player)) => {
                Some(AchievementTrigger::SessionStarted {
                    player,
                    host: *host,
                })
            }
            _ => None,
        };
        if state.session.enabled
            && let Some(trigger) = trigger
        {
            let context = notification.game_context.map(|context| AchievementContext {
                match_id: context.match_id,
                hand_index: context.hand_index,
                sequence: context.sequence,
            });
            let source = state
                .session
                .connected
                .then_some(state.session.connection_epoch);
            state
                .session
                .record(state.book.trigger_with_context(&trigger, context), source);
            if let AchievementTrigger::Interaction { player, event } = trigger
                && let Some(event) = state.observer.activity.observe_interaction(player, &event)
            {
                state.session.record(
                    state.book.trigger(&AchievementTrigger::Personal(event)),
                    source,
                );
            }
        }
        if let ServerEvent::AchievementUnlocked(announcement) = &notification.event
            && announcement.profile_id != state.profile.identity.profile_id()
            && let Some(definition) = achievement_by_id(&announcement.achievement_id)
        {
            unlocked.write(AchievementUnlocked {
                definition,
                recipient: AchievementRecipient::TablePlayer {
                    name: announcement.name.clone(),
                },
            });
        }
    }
    for event in local.read() {
        if state.session.enabled {
            state.session.record(state.book.trigger(&event.0), None);
        }
    }
}
