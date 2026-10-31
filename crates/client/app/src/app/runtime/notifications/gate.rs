use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, MatchId, RoomId};
use std::time::{Duration, Instant};

pub(super) const REMINDER_COOLDOWN: Duration = Duration::from_secs(45);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ActionContext {
    pub room: RoomId,
    pub game: GameKind,
    pub match_id: MatchId,
    pub player: PlayerId,
}

/// 冷却跨回合、对局和房间保留；同一个持续等待的操作只通知一次。
#[derive(Default)]
pub(super) struct ActionReminder {
    context: Option<ActionContext>,
    notified: bool,
    last_attempt: Option<Instant>,
}

impl ActionReminder {
    pub(super) fn observe(
        &mut self,
        context: Option<ActionContext>,
        focused: bool,
        now: Instant,
    ) -> bool {
        if self.context != context {
            self.context = context;
            self.notified = false;
        }
        if context.is_none()
            || focused
            || self.notified
            || self
                .last_attempt
                .is_some_and(|last| now.duration_since(last) < REMINDER_COOLDOWN)
        {
            return false;
        }
        self.notified = true;
        self.last_attempt = Some(now);
        true
    }
}
