//! 设置窗口的本地标签状态。

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SettingsTab {
    #[default]
    Appearance,
    Sound,
    About,
}

#[derive(Default)]
pub(crate) struct SettingsUiState {
    pub open: bool,
    pub tab: SettingsTab,
}
