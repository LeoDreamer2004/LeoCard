//! 跨页面共用的 UI 标记、皮肤与桌面外观控件类型。

use bevy::prelude::*;

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct AutoPlayOverlay;

#[derive(Component)]
pub struct RuleHelp {
    pub tooltip: Entity,
}

#[derive(Component)]
pub struct TableBackground;

#[derive(Component)]
pub struct TableAppearanceSlider(pub TableAppearanceSetting);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableAppearanceSetting {
    Brightness,
    Vignette,
    Volume,
}

#[derive(Component)]
pub struct TableAppearanceIndicator {
    pub setting: TableAppearanceSetting,
    pub part: TableAppearanceIndicatorPart,
}

#[derive(Clone, Copy)]
pub enum TableAppearanceIndicatorPart {
    Fill,
    Knob,
}

#[derive(Component)]
pub struct TableAppearanceLabel(pub TableAppearanceSetting);

#[derive(Component)]
pub struct ButtonTint {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

#[derive(Component)]
pub struct BackgroundButtonTint;

#[derive(Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Warning,
    Pass,
}

#[derive(Clone, Copy)]
pub enum PanelSkin {
    Window,
    Section,
    Popup,
}
