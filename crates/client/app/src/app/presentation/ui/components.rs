//! 跨页面共用的 UI 标记、皮肤与桌面外观控件类型。

use bevy::prelude::*;
use leocard_ui::CustomButtonMotion;

#[derive(Component)]
pub(crate) struct UiRoot;

#[derive(Component)]
pub(crate) struct AutoPlayOverlay;

#[derive(Component)]
pub(crate) struct TableBackground;

#[derive(Component)]
#[require(CustomButtonMotion)]
pub(crate) struct TableAppearanceSlider(pub TableAppearanceSetting);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TableAppearanceSetting {
    Brightness,
    Vignette,
    Volume,
}

#[derive(Component)]
pub(crate) struct TableAppearanceIndicator(pub TableAppearanceSetting);

#[derive(Component)]
pub(crate) struct TableAppearanceLabel(pub TableAppearanceSetting);

#[derive(Clone, Copy)]
pub(crate) enum ButtonKind {
    Primary,
    Secondary,
    Warning,
    Pass,
}

#[derive(Clone, Copy)]
pub(crate) enum PanelSkin {
    Window,
    Section,
    Popup,
}
