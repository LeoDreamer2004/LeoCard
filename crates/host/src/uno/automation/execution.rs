use super::super::{
    PendingDrawReveal, UnoSession, analysis_events, append_finished_event, events_for_outcome,
    pending_draw_reveal_for_outcome,
};
use fastrand::Rng;
use leocard_protocol::UnoEvent;
use leocard_uno::{UnoActionContext, UnoBot, UnoBotObservation};

impl UnoSession {
    pub(in super::super) fn play_automatic_action(
        &mut self,
    ) -> Option<(Vec<UnoEvent>, Option<PendingDrawReveal>)> {
        let forced_skip = self.automatically_resolves_skip();
        let player = self.game.as_ref()?.turn()?.current_player;
        let game = self.game.as_mut()?;
        let observation = UnoBotObservation::from_game(game, forced_skip)?;
        let call_uno = observation.should_call_uno();
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
        let action = UnoBot::choose(&observation, &mut Rng::new())?;
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
