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

/// 仅在开始等待响应时检查能否提醒，之后不补发。
/// 冷却跨回合、对局和房间保留。
#[derive(Default)]
pub(super) struct ActionReminder {
    context: Option<ActionContext>,
    last_attempt: Option<Instant>,
}

impl ActionReminder {
    pub(super) fn observe(
        &mut self,
        context: Option<ActionContext>,
        suppressed: bool,
        now: Instant,
    ) -> bool {
        if self.context == context {
            return false;
        }
        self.context = context;
        if context.is_none()
            || suppressed
            || self
                .last_attempt
                .is_some_and(|last| now.duration_since(last) < REMINDER_COOLDOWN)
        {
            return false;
        }
        self.last_attempt = Some(now);
        true
    }
}
