//! 错误提示覆盖层的运行时状态与标记组件。

use super::*;

#[derive(Resource, Default)]
pub struct PlayErrorToast {
    pub seen_rejection_serial: u64,
    pub seen_notice_serial: u64,
    pub observed_form_error: Option<String>,
    pub observed_appearance_error: Option<String>,
    pub message: Option<String>,
    pub elapsed: f32,
    pub shake_elapsed: Option<f32>,
    pub entering: bool,
    pub active: bool,
}

#[derive(Component)]
pub struct PlayErrorPopup;

#[derive(Component)]
pub struct PlayErrorPopupText;
