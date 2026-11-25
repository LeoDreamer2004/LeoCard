use super::section::add_settings_separator;
use crate::app::presentation::{DANGER, MUTED, TEXT, add_text, add_title, rule_help_style};
use crate::app::runtime::{AiModels, AppearancePreferences, ModelStatus, UiAssets};
use crate::app::shell::settings::ai::AiProxyPortInput;
use crate::app::shell::{
    CozyButtonVariant, CozyDropdown, CozyDropdownOption, SettingsUiAction, ShellOverlayLayer,
    UiAction, add_cozy_button_variant, add_cozy_button_with_icon, add_cozy_disabled_button,
    add_cozy_switch,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::Underline;
use bevy::ui_widgets::Button;
use leocard_ai::{DevicePreference, ModelId, model_spec};
use leocard_ui::{TextInput, UiComposer, attach_help_tooltip, spawn_node};

pub(super) struct AiSettings<'a> {
    form: &'a AppearancePreferences,
    models: &'a AiModels,
    device_menu_open: bool,
    assets: &'a UiAssets,
}

impl<'a> AiSettings<'a> {
    pub(super) fn new(
        form: &'a AppearancePreferences,
        models: &'a AiModels,
        device_menu_open: bool,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            form,
            models,
            device_menu_open,
            assets,
        }
    }

    pub(super) fn render(&self, commands: &mut UiComposer, root: Entity, parent: Entity) {
        self.basic_settings(commands, root, parent);
        for (id, title) in [
            (ModelId::Douzero, "三人斗地主"),
            (ModelId::Kdens3, "国标麻将"),
            (ModelId::GuandanActor, "掼蛋"),
        ] {
            self.model_settings(commands, parent, id, title);
        }
    }

    fn basic_settings(&self, commands: &mut UiComposer, root: Entity, parent: Entity) {
        add_title(commands, parent, "基础设置", 20.0, TEXT, self.assets);
        let row = settings_row(commands, parent);
        add_text(commands, row, "推理设备", 16.0, TEXT, self.assets);
        CozyDropdown {
            label: self.form.ai.device.label(),
            toggle_action: UiAction::Settings(SettingsUiAction::ToggleAiDeviceMenu),
            close_action: UiAction::Settings(SettingsUiAction::CloseAiDeviceMenu),
            options: [
                DevicePreference::Auto,
                DevicePreference::Cpu,
                DevicePreference::Gpu,
            ]
            .into_iter()
            .map(|device| CozyDropdownOption {
                label: device.label(),
                selected: self.form.ai.device == device,
                action: (device != DevicePreference::Gpu || self.models.gpu_available())
                    .then_some(UiAction::Settings(SettingsUiAction::SetAiDevice(device))),
            })
            .collect(),
        }
        .render_in_dialog(commands, root, row, self.device_menu_open, self.assets);
        let row = settings_row(commands, parent);
        add_text(commands, row, "下载代理端口", 16.0, TEXT, self.assets);
        let mut input = TextInput::new(
            "settings.ai.download-proxy-port",
            &self.form.ai.download_proxy_port,
        );
        input.placeholder = "留空使用环境代理";
        input.max_characters = 5;
        input.filter = |character| character.is_ascii_digit();
        let editor = input.spawn(
            commands,
            row,
            Node {
                width: px(190),
                height: px(40),
                padding: UiRect::axes(px(13), px(8)),
                ..default()
            },
            self.assets.text_input_style(),
        );
        commands.entity(editor).insert(AiProxyPortInput);
    }

    fn model_settings(&self, commands: &mut UiComposer, parent: Entity, id: ModelId, title: &str) {
        add_settings_separator(commands, parent);
        let spec = model_spec(id);
        let bytes: u64 = spec.files.iter().map(|file| file.bytes).sum();
        let status = self.models.status(id);
        let heading = settings_row(commands, parent);
        add_title(commands, heading, title, 20.0, TEXT, self.assets);
        let controls = spawn_node(
            commands,
            heading,
            Node {
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(24),
                ..default()
            },
            None,
        );
        if let ModelStatus::Failed(error) = status {
            self.model_error(commands, controls, error);
        }
        let enabled = status.ready()
            && (self.form.ai.device != DevicePreference::Gpu || self.models.gpu_available());
        add_cozy_switch(
            commands,
            controls,
            self.form.ai.enabled.contains(&id),
            enabled.then_some(UiAction::Settings(SettingsUiAction::ToggleStrongAi(id))),
            self.assets,
        );
        let row = settings_row(commands, parent);
        commands
            .entity(row)
            .entry::<Node>()
            .and_modify(|mut node| node.column_gap = px(24));
        let details = format!("{}，{:.0}MB", spec.name, bytes as f64 / 1_000_000.0);
        let label = add_text(commands, row, details, 16.0, TEXT, self.assets);
        commands.entity(label).insert(Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            ..default()
        });
        let buttons = spawn_node(
            commands,
            row,
            Node {
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        let github = spec.source.starts_with("https://github.com/");
        add_cozy_button_with_icon(
            commands,
            buttons,
            if github { "GitHub" } else { "模型主页" },
            UiAction::Settings(SettingsUiAction::OpenAiModelSource(id)),
            self.assets,
            px(124),
            44.0,
            Some(if github {
                self.assets.controls.github_mark.clone()
            } else {
                self.assets.controls.hugging_face_mark.clone()
            }),
        );
        if status.busy() {
            add_cozy_disabled_button(commands, buttons, "下载模型", self.assets, px(104), 44.0);
        } else {
            let (label, action) = if status.ready() {
                ("删除模型", SettingsUiAction::RemoveAiModel(id))
            } else {
                ("下载模型", SettingsUiAction::DownloadAiModel(id))
            };
            add_cozy_button_variant(
                commands,
                buttons,
                label,
                UiAction::Settings(action),
                self.assets,
                px(104),
                44.0,
                if status.ready() {
                    CozyButtonVariant::Danger
                } else {
                    CozyButtonVariant::Cool
                },
            );
        }
        self.model_status(commands, row, status);
    }

    fn model_status(&self, commands: &mut UiComposer, row: Entity, status: &ModelStatus) {
        let label = match status {
            ModelStatus::Missing | ModelStatus::Ready | ModelStatus::Failed(_) => return,
            ModelStatus::Loading => "读取中".to_owned(),
            ModelStatus::Downloading { completed, total } => {
                format!("下载中 {:.0}%", *completed as f64 / *total as f64 * 100.0)
            }
            ModelStatus::Converting => "转换中".to_owned(),
        };
        add_text(commands, row, label, 14.0, MUTED, self.assets);
    }

    fn model_error(&self, commands: &mut UiComposer, parent: Entity, error: &str) {
        let label = add_text(commands, parent, "出现错误", 14.0, DANGER, self.assets);
        commands
            .entity(label)
            .insert((Underline, Button, Pickable::default()));
        let tooltip = attach_help_tooltip(commands, label, error, rule_help_style(self.assets));
        commands
            .entity(tooltip)
            .insert(GlobalZIndex(ShellOverlayLayer::Dialog.z_index().0 + 31));
    }
}

fn settings_row(commands: &mut UiComposer, parent: Entity) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(40),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(12),
            row_gap: px(6),
            ..default()
        },
        None,
    )
}
