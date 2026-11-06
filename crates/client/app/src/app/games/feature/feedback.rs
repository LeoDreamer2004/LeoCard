/// Convert a game fact into typed feedback. Scheduling, deduplication and assets remain game-owned.
pub(crate) trait GameFeedback<Context = ()> {
    type Cue;

    fn feedback(&self, context: Context) -> Self::Cue;
}
