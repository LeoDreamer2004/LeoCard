use bevy::prelude::GlobalZIndex;

/// Global ordering for shell overlays. Dialogs in the same flow share one layer.
#[derive(Clone, Copy)]
pub(crate) enum ShellOverlayLayer {
    Dialog = 2000,
    Feedback = 2150,
    #[cfg(not(target_os = "android"))]
    ConnectionProgress = 2200,
    Update = 2300,
    Confirmation = 2400,
}

impl ShellOverlayLayer {
    pub(crate) fn z_index(self) -> GlobalZIndex {
        GlobalZIndex(self as i32)
    }
}
