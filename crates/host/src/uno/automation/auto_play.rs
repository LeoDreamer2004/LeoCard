use super::BotPolicy;
use super::action::AutomaticUnoAction;
use leocard_uno::GameState;

pub(super) struct Policy;

impl Policy {
    pub fn should_call_uno(game: &GameState, forced_skip: bool) -> bool {
        BotPolicy::should_call_uno(game, forced_skip)
    }

    pub fn choose(game: &GameState, forced_skip: bool) -> Option<AutomaticUnoAction> {
        BotPolicy::choose(game, forced_skip)
    }
}
