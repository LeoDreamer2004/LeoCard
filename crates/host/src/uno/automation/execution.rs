use super::super::{
    PendingDrawReveal, UnoSession, analysis_events, append_finished_event, events_for_outcome,
    from_core_player, pending_draw_reveal_for_outcome,
};
use super::{BotPolicy, auto_play};
use leocard_protocol::UnoEvent;
use leocard_uno::UnoActionContext;

impl UnoSession {
    pub(in super::super) fn play_automatic_action(
        &mut self,
    ) -> Option<(Vec<UnoEvent>, Option<PendingDrawReveal>)> {
        let forced_skip = self.automatically_resolves_skip();
        let player = self.game.as_ref()?.turn()?.current_player;
        let is_bot = self
            .room
            .players
            .get(usize::from(from_core_player(player).0))?
            .is_bot;
        let game = self.game.as_mut()?;
        let call_uno = if is_bot {
            BotPolicy::should_call_uno(game, forced_skip)
        } else {
            auto_play::Policy::should_call_uno(game, forced_skip)
        };
        let uno_before = UnoActionContext::capture(game, player);
        let uno_outcome = call_uno.then(|| game.call_uno(player).ok()).flatten();
        let mut events = uno_outcome
            .as_ref()
            .map(|outcome| events_for_outcome(outcome, &[]))
            .unwrap_or_default();
        if let (Some(before), Some(outcome)) = (uno_before, uno_outcome.as_ref()) {
            events.extend(analysis_events(
                self.statistics
                    .as_mut()
                    .expect("active UNO games have a recorder")
                    .observe(before, outcome, &[], false, game),
            ));
        }
        let before = UnoActionContext::capture(game, player)?;
        let action = if is_bot {
            BotPolicy::choose(game, forced_skip)?
        } else {
            auto_play::Policy::choose(game, forced_skip)?
        };
        let played = action.played_cards();
        let outcome = action.apply(game, player).ok()?;
        events.extend(events_for_outcome(&outcome, &played));
        append_finished_event(&mut events, game);
        events.extend(analysis_events(
            self.statistics
                .as_mut()
                .expect("active UNO games have a recorder")
                .observe(before, &outcome, &played, false, game),
        ));
        let draw_reveal = pending_draw_reveal_for_outcome(&outcome, game);
        Some((events, draw_reveal))
    }
}
