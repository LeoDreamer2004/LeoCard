use crate::{GuandanGameError, GuandanGameResult};
use leocard_game_common::PlayerId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuandanMatchState {
    levels: [u8; 2],
    level_advance: u8,
    hand_number: u32,
    next_starter: PlayerId,
    dealer_team: u8,
    previous_result: Option<GuandanGameResult>,
    complete: bool,
    hand_finished: bool,
}

impl GuandanMatchState {
    pub fn new(starting_level: u8, first_starter: PlayerId) -> Result<Self, GuandanGameError> {
        if !(2..=14).contains(&starting_level) {
            return Err(GuandanGameError::InvalidRules);
        }
        if first_starter.index() >= 4 {
            return Err(GuandanGameError::InvalidPlayer(first_starter));
        }
        Ok(Self {
            levels: [starting_level; 2],
            level_advance: 0,
            hand_number: 1,
            next_starter: first_starter,
            dealer_team: first_starter.0 % 2,
            previous_result: None,
            complete: false,
            hand_finished: false,
        })
    }

    pub const fn levels(&self) -> [u8; 2] {
        self.levels
    }

    pub const fn level_advance(&self) -> u8 {
        self.level_advance
    }

    pub const fn hand_number(&self) -> u32 {
        self.hand_number
    }

    pub const fn complete(&self) -> bool {
        self.complete
    }

    pub const fn next_starter(&self) -> PlayerId {
        self.next_starter
    }

    pub const fn current_level(&self) -> u8 {
        self.levels[self.dealer_team as usize]
    }

    pub const fn previous_result(&self) -> Option<&GuandanGameResult> {
        self.previous_result.as_ref()
    }

    /// Records exactly one result for the current hand. A team passes ace only
    /// when its players finish first and second while it is already at ace.
    pub fn finish_hand(&mut self, result: &GuandanGameResult) -> bool {
        if self.complete || self.hand_finished || result.winning_team > 1 {
            return false;
        }
        let team = usize::from(result.winning_team);
        if !(1..=3).contains(&result.level_gain) {
            return false;
        }
        let was_at_ace = self.levels[team] == 14;
        let next = self.levels[team].saturating_add(result.level_gain).min(14);
        self.level_advance = next - self.levels[team];
        self.levels[team] = next;
        self.complete = was_at_ace && result.level_gain == 3;
        self.previous_result = Some(result.clone());
        self.next_starter = result.finishing_order[0];
        self.dealer_team = result.winning_team;
        self.hand_finished = true;
        true
    }

    pub fn next_hand(&mut self) -> bool {
        if !self.hand_finished || self.complete {
            return false;
        }
        self.hand_number = self.hand_number.saturating_add(1);
        self.level_advance = 0;
        self.hand_finished = false;
        true
    }

    pub const fn hand_finished(&self) -> bool {
        self.hand_finished
    }
}

#[cfg(test)]
mod tests {
    use super::GuandanMatchState;
    use crate::GuandanGameResult;
    use leocard_game_common::PlayerId;

    #[test]
    fn reaching_ace_requires_another_double_finish_to_complete() {
        let mut match_state = GuandanMatchState::new(13, PlayerId(0)).unwrap();
        let result = GuandanGameResult {
            known_positions: 2,
            finishing_order: [PlayerId(0), PlayerId(2), PlayerId(1), PlayerId(3)],
            winning_team: 0,
            level_gain: 3,
        };
        assert!(match_state.finish_hand(&result));
        assert_eq!(match_state.levels(), [14, 13]);
        assert_eq!(match_state.level_advance(), 1);
        assert!(!match_state.complete());
        assert!(!match_state.finish_hand(&result));
        assert!(match_state.next_hand());
        assert_eq!(match_state.hand_number(), 2);
        assert_eq!(match_state.current_level(), 14);
        assert!(match_state.finish_hand(&result));
        assert!(match_state.complete());
    }
}
