//! 设置窗口的本地标签状态。

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SettingsTab {
    #[default]
    Appearance,
    Sound,
    Notifications,
    Controls,
    #[cfg(not(target_arch = "wasm32"))]
    Ai,
    Room,
    About,
}

#[derive(Default)]
pub(crate) struct SettingsUiState {
    pub open: bool,
    pub tab: SettingsTab,
    #[cfg(not(target_arch = "wasm32"))]
    pub ai_device_menu_open: bool,
}
