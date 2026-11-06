use super::GameScreen;

/// Required local decisions only; excludes optional actions and spectator controls.
pub(crate) trait GameActionPrompt: GameScreen {
    fn requires_action(&self, game: &Self::Snapshot) -> bool;
}
