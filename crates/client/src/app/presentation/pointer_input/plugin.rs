//! 指针命中规则和交互控件的单次按下事件。

use bevy::{
    picking::{Pickable, events::PointerPress, hover::PickingInteraction, pointer::PointerButton},
    prelude::*,
    ui::{
        InteractionDisabled,
        picking_backend::{UiPickingCamera, UiPickingSettings},
    },
    ui_widgets::{Button, TextInput},
};

#[derive(Message)]
pub(crate) struct UiPress(pub Entity);

/// 控件参与通用点击反馈，实际功能仍由各控件处理。
#[derive(Component, Default)]
pub(crate) struct UiPressTarget;

pub(crate) struct PointerInputPlugin;

impl Plugin for PointerInputPlugin {
    fn build(&self, app: &mut App) {
        // 只拾取明确声明的命中区域，不给布局节点预置不可点击状态。
        // 这样先创建 Node、随后添加 Button 的控件也能获得正常拾取规则。
        app.insert_resource(UiPickingSettings {
            require_markers: true,
        })
        .register_required_components::<Camera2d, UiPickingCamera>()
        .register_required_components::<Button, Node>()
        .register_required_components::<UiPressTarget, Pickable>()
        .register_required_components::<Button, UiPressTarget>()
        .register_required_components::<Button, PickingInteraction>()
        .register_required_components::<TextInput, UiPressTarget>()
        .add_message::<UiPress>()
        .add_observer(collect_ui_press);
    }
}

fn collect_ui_press(
    mut press: On<PointerPress>,
    controls: Query<Has<InteractionDisabled>, With<UiPressTarget>>,
    mut presses: MessageWriter<UiPress>,
) {
    let Ok(disabled) = controls.get(press.entity) else {
        return;
    };
    press.propagate(false);
    if !disabled && press.button == PointerButton::Primary {
        presses.write(UiPress(press.entity));
    }
}
