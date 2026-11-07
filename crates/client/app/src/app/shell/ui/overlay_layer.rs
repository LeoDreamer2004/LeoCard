use bevy::prelude::GlobalZIndex;

/// Global ordering for shell overlays. Dialogs in the same flow share one layer.
#[derive(Clone, Copy)]
#[repr(i32)]
pub(crate) enum ShellOverlayLayer {
    Dialog = 2000,
    #[cfg(not(target_os = "android"))]
    ConnectionProgress = 2200,
    Update = 2300,
    Confirmation = 2400,
    /// Reserved for error feedback above every dialog and transition.
    Feedback = i32::MAX,
}

impl ShellOverlayLayer {
    pub(crate) fn z_index(self) -> GlobalZIndex {
        GlobalZIndex(self as i32)
    }
}
