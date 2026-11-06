use crate::app::runtime::ClientResource;
use crate::app::shell::SocialUiState;

/// Reconcile accepted snapshots/rejections and clear game-owned state on phase changes.
pub(crate) trait GameUiLifecycle {
    fn reconcile(&mut self, client: Option<&ClientResource>, social: &mut SocialUiState);
}
