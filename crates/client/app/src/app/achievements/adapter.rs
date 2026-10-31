use crate::app::runtime::ServerNotification;
use leocard_achievements::{AchievementContext, AchievementTrigger};
use leocard_protocol::ServerEvent;

pub(super) struct AchievementInput {
    pub trigger: AchievementTrigger,
    pub context: Option<AchievementContext>,
}

impl AchievementInput {
    pub fn from_notification(notification: &ServerNotification) -> Option<Self> {
        let player = notification.player?;
        let trigger = match &notification.event {
            ServerEvent::GameEvent(event) if notification.game_context.is_some() => {
                AchievementTrigger::Game {
                    player,
                    event: event.into(),
                }
            }
            ServerEvent::PlayerInteraction(event) => AchievementTrigger::Interaction {
                player,
                event: event.into(),
            },
            ServerEvent::ChatMessage(message) => AchievementTrigger::Chat {
                player,
                message: message.into(),
            },
            ServerEvent::GameStarted { host, .. } if notification.game_context.is_some() => {
                AchievementTrigger::SessionStarted {
                    player,
                    host: *host,
                }
            }
            _ => return None,
        };
        let context = match trigger {
            AchievementTrigger::Game { .. } | AchievementTrigger::SessionStarted { .. } => {
                notification.game_context.map(|context| AchievementContext {
                    match_id: context.match_id.0,
                    hand_index: context.hand_index,
                    sequence: context.sequence,
                })
            }
            _ => None,
        };
        Some(Self { trigger, context })
    }
}
