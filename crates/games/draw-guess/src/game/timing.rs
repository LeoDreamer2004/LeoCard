use super::{DrawGuessGame, DrawGuessPhase};
use crate::{CanvasDocument, DrawAction, DrawGuessRuleSet};

impl DrawGuessGame {
    pub fn advance_ms(&mut self, elapsed: u32) -> bool {
        self.elapsed_ms += u64::from(elapsed);
        let mut elapsed = elapsed;
        let mut changed = false;
        while self.phase != DrawGuessPhase::Finished && elapsed >= self.remaining_ms {
            elapsed -= self.remaining_ms;
            match self.phase {
                DrawGuessPhase::Choosing => {
                    self.choose_word(self.artist().unwrap(), 0)
                        .expect("first valid word");
                }
                DrawGuessPhase::Drawing => self.reveal(),
                DrawGuessPhase::Reveal => self.next_turn(),
                DrawGuessPhase::Finished => break,
            }
            changed = true;
        }
        if self.phase != DrawGuessPhase::Finished {
            self.remaining_ms -= elapsed;
        }
        changed
    }

    pub(super) fn reveal(&mut self) {
        if matches!(
            self.phase,
            DrawGuessPhase::Finished | DrawGuessPhase::Reveal
        ) {
            return;
        }
        if self.canvas.pending().is_some() {
            self.canvas
                .apply(&DrawAction::End)
                .expect("valid pending stroke");
        }
        self.phase = DrawGuessPhase::Reveal;
        self.remaining_ms = u32::from(DrawGuessRuleSet::REVEAL_SECONDS) * 1000;
    }

    fn next_turn(&mut self) {
        loop {
            self.turn += 1;
            if self.turn >= self.total_turns() {
                self.phase = DrawGuessPhase::Finished;
                self.remaining_ms = 0;
                return;
            }
            if self.players[self.turn as usize % self.players.len()].connected {
                break;
            }
        }
        self.phase = DrawGuessPhase::Choosing;
        self.remaining_ms = u32::from(DrawGuessRuleSet::CHOICE_SECONDS) * 1000;
        self.secret = None;
        self.canvas = CanvasDocument::default();
        self.history.clear();
        self.last_guess_ms.fill(None);
        for player in &mut self.players {
            player.guessed = false;
        }
    }
}
