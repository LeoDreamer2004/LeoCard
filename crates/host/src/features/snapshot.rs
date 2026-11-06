use leocard_game_common::PlayerId;
use leocard_protocol::GameSnapshot;

/// A recipient-specific game view; implementations own private-card and spectator policy.
pub(crate) trait GameSnapshotView {
    type Snapshot: Into<GameSnapshot>;

    fn game_view(&self, recipient: PlayerId) -> Self::Snapshot;
}
