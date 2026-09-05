//! Reusable cards, buttons, panels, labels, avatars, and display formatting.

use super::*;

pub(super) fn add_card_image(
    commands: &mut Commands,
    parent: Entity,
    card: Card,
    size: CardSize,
    index: usize,
    is_last: bool,
    initially_hidden: bool,
    assets: &UiAssets,
) -> Entity {
    let (width, height) = size.dimensions();
    let reveal = match size {
        CardSize::Score => SCORE_CARD_REVEAL,
        CardSize::TableScore => TABLE_SCORE_CARD_REVEAL,
        CardSize::FinishedHand => FINISHED_HAND_CARD_REVEAL,
        CardSize::Hand | CardSize::Seat => TABLE_CARD_REVEAL,
    };
    let image = assets
        .cards
        .get(&(card.rank(), card.suit()))
        .expect("all valid card faces are preloaded")
        .clone();
    let entity = commands
        .spawn((
            Node {
                width: px(width),
                height: px(height),
                margin: UiRect::right(px(if is_last { 0.0 } else { reveal - width })),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            ImageNode::new(image).with_color(if initially_hidden {
                Color::NONE
            } else {
                Color::WHITE
            }),
            ZIndex(index as i32),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub(super) fn add_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    kind: ButtonKind,
    assets: &UiAssets,
) -> Entity {
    add_action_button_with_label(commands, parent, label, action, kind, assets).0
}

pub(super) fn add_action_button_with_label(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    kind: ButtonKind,
    assets: &UiAssets,
) -> (Entity, Entity) {
    let (image, normal, hovered, pressed) = match kind {
        ButtonKind::Primary => (
            assets.primary_button.clone(),
            Color::WHITE,
            Color::srgb(1.0, 1.0, 0.82),
            Color::srgb(0.78, 0.90, 0.78),
        ),
        ButtonKind::Secondary => (
            assets.secondary_button.clone(),
            Color::srgb(0.48, 0.62, 0.76),
            Color::srgb(0.64, 0.76, 0.88),
            Color::srgb(0.32, 0.46, 0.60),
        ),
        ButtonKind::Warning => (
            assets.warning_button.clone(),
            Color::srgb(0.88, 0.68, 0.24),
            Color::srgb(0.98, 0.82, 0.48),
            Color::srgb(0.72, 0.54, 0.18),
        ),
        ButtonKind::Pass => (
            assets.danger_button.clone(),
            Color::srgb(0.58, 0.42, 0.42),
            Color::srgb(0.76, 0.58, 0.56),
            Color::srgb(0.42, 0.28, 0.27),
        ),
    };
    let entity = commands
        .spawn((
            Button,
            action,
            ButtonTint {
                normal,
                hovered,
                pressed,
            },
            Node {
                min_width: px(150),
                height: px(48),
                padding: UiRect::axes(px(20), px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(image)
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    let label = add_text(commands, entity, label, 16.0, Color::WHITE, assets);
    (entity, label)
}

/// 托管时覆盖整条手牌与操作区。蒙版本身是唯一可点击目标，因此其后的牌、
/// 操作按钮和聊天抽屉在这个区域内都不会收到指针事件。
pub(super) fn add_auto_play_overlay(commands: &mut Commands, parent: Entity, assets: &UiAssets) {
    let overlay = commands
        .spawn((
            Button,
            UiAction::ToggleAutoPlay,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(0),
                height: px(190),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(5),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.82)),
            GlobalZIndex(1900),
            FocusPolicy::Block,
            AutoPlayOverlay,
        ))
        .id();
    commands.entity(parent).add_child(overlay);
    let title = add_text(commands, overlay, "您已托管", 27.0, ACCENT, assets);
    commands.entity(title).insert((
        FocusPolicy::Pass,
        TextShadow {
            offset: Vec2::new(1.2, 1.5),
            color: Color::BLACK.with_alpha(0.92),
        },
    ));
    let detail = add_text(commands, overlay, "点击此处取消", 14.0, TEXT, assets);
    commands.entity(detail).insert(FocusPolicy::Pass);
}

pub(super) fn add_disabled_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
) -> Entity {
    let entity = commands
        .spawn((
            Node {
                min_width: px(150),
                height: px(48),
                padding: UiRect::axes(px(20), px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.disabled_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgb(0.56, 0.58, 0.57)),
            FocusPolicy::Block,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    add_text(commands, entity, label, 16.0, Color::WHITE, assets);
    entity
}

pub(super) fn add_header_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
) {
    let normal = Color::srgb(0.42, 0.56, 0.70);
    let entity = commands
        .spawn((
            Button,
            action,
            ButtonTint {
                normal,
                hovered: Color::srgb(0.60, 0.72, 0.84),
                pressed: Color::srgb(0.28, 0.40, 0.54),
            },
            Node {
                min_width: px(108),
                height: px(36),
                padding: UiRect::axes(px(14), px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    add_text(commands, entity, label, 14.0, Color::WHITE, assets);
}

pub(super) fn add_header_exit_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
) {
    let normal = Color::WHITE;
    let entity = commands
        .spawn((
            Button,
            UiAction::LeaveRoom,
            ButtonTint {
                normal,
                hovered: Color::srgb(1.0, 0.88, 0.84),
                pressed: Color::srgb(0.74, 0.66, 0.64),
            },
            Node {
                min_width: px(108),
                height: px(36),
                padding: UiRect::axes(px(14), px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.danger_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    add_text(commands, entity, label, 14.0, Color::WHITE, assets);
}

pub(super) fn add_panel(
    commands: &mut Commands,
    parent: Entity,
    mut node: Node,
    color: Color,
    skin: PanelSkin,
    assets: &UiAssets,
) -> Entity {
    node.padding = UiRect::all(px(match skin {
        PanelSkin::Window => 28.0,
        PanelSkin::Section => 24.0,
        PanelSkin::Popup => 22.0,
    }));
    node.border = UiRect::all(px(1));
    node.border_radius = BorderRadius::all(px(8));
    let entity = spawn_node(commands, parent, node, Some(color));
    decorate_panel_skin(commands, entity, skin, assets);
    entity
}

/// 给任意布局节点叠加独立的九宫格面板皮肤。玩家框仍使用专用贴图；这里仅
/// 服务于主窗口、内容分区和小型提示框，避免随尺寸拉伸边角与描边。
pub(super) fn decorate_panel_skin(
    commands: &mut Commands,
    panel: Entity,
    skin: PanelSkin,
    assets: &UiAssets,
) -> Entity {
    let (image, border, alpha) = match skin {
        PanelSkin::Window => (assets.panel_window.clone(), 40.0, 0.98),
        PanelSkin::Section => (assets.panel_section.clone(), 28.0, 0.96),
        PanelSkin::Popup => (assets.panel_popup.clone(), 36.0, 0.98),
    };
    commands
        .entity(panel)
        .insert((BackgroundColor(Color::NONE), BorderColor::all(Color::NONE)));
    let texture = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            ImageNode::new(image)
                .with_mode(NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(border),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }))
                .with_color(Color::WHITE.with_alpha(alpha)),
            ZIndex(-1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(panel).add_child(texture);
    texture
}

pub(super) fn spawn_node(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    background: Option<Color>,
) -> Entity {
    let mut entity = commands.spawn(node);
    if let Some(color) = background {
        entity.insert(BackgroundColor(color));
    }
    let entity = entity.id();
    commands.entity(parent).add_child(entity);
    entity
}

pub(super) fn add_section_title(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    assets: &UiAssets,
) {
    add_text(commands, parent, text, 21.0, ACCENT, assets);
}

pub(super) struct RuleConfigRow<'a> {
    pub(super) label: &'a str,
    pub(super) value: String,
    pub(super) help: &'a str,
    pub(super) editable: bool,
    pub(super) previous: Option<RuleSet>,
    pub(super) next: Option<RuleSet>,
}

pub(super) fn add_rule_config_row(
    commands: &mut Commands,
    parent: Entity,
    spec: RuleConfigRow,
    assets: &UiAssets,
) {
    let RuleConfigRow {
        label,
        value,
        help,
        editable,
        previous,
        next,
    } = spec;
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(34),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(commands, row, label, 14.0, MUTED, assets);
    let controls = spawn_node(
        commands,
        row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    if editable {
        add_rule_step_button(commands, controls, "‹", previous, assets);
    }
    let value_box = spawn_node(
        commands,
        controls,
        Node {
            min_width: px(68),
            min_height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, value_box, value, 14.0, TEXT, assets);
    if editable {
        add_rule_step_button(commands, controls, "›", next, assets);
    }
    add_rule_help(commands, controls, help, assets);
}

pub(super) struct TexasRuleConfigRow<'a> {
    pub(super) label: &'a str,
    pub(super) value: String,
    pub(super) help: &'a str,
    pub(super) editable: bool,
    pub(super) previous: Option<TexasHoldemRuleSet>,
    pub(super) next: Option<TexasHoldemRuleSet>,
}

pub(super) fn add_texas_rule_config_row(
    commands: &mut Commands,
    parent: Entity,
    spec: TexasRuleConfigRow,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(38),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(commands, row, spec.label, 14.0, MUTED, assets);
    let controls = spawn_node(
        commands,
        row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    if spec.editable {
        add_texas_rule_step_button(commands, controls, "‹", spec.previous, assets);
    }
    let value_box = spawn_node(
        commands,
        controls,
        Node {
            min_width: px(78),
            min_height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, value_box, spec.value, 14.0, TEXT, assets);
    if spec.editable {
        add_texas_rule_step_button(commands, controls, "›", spec.next, assets);
    }
    add_rule_help(commands, controls, spec.help, assets);
}

pub(super) struct UnoRuleConfigRow<'a> {
    pub(super) label: &'a str,
    pub(super) value: String,
    pub(super) help: &'a str,
    pub(super) editable: bool,
    pub(super) previous: Option<UnoRuleSet>,
    pub(super) next: Option<UnoRuleSet>,
}

pub(super) fn add_uno_rule_config_row(
    commands: &mut Commands,
    parent: Entity,
    spec: UnoRuleConfigRow,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(38),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(commands, row, spec.label, 14.0, MUTED, assets);
    let controls = spawn_node(
        commands,
        row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    if spec.editable {
        add_uno_rule_step_button(commands, controls, "‹", spec.previous, assets);
    }
    let value_box = spawn_node(
        commands,
        controls,
        Node {
            min_width: px(78),
            min_height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, value_box, spec.value, 14.0, TEXT, assets);
    if spec.editable {
        add_uno_rule_step_button(commands, controls, "›", spec.next, assets);
    }
    add_rule_help(commands, controls, spec.help, assets);
}

pub(super) struct ShengjiRuleConfigRow<'a> {
    pub(super) label: &'a str,
    pub(super) value: String,
    pub(super) help: &'a str,
    pub(super) editable: bool,
    pub(super) previous: Option<ShengjiRuleSet>,
    pub(super) next: Option<ShengjiRuleSet>,
}

pub(super) fn add_shengji_rule_config_row(
    commands: &mut Commands,
    parent: Entity,
    spec: ShengjiRuleConfigRow,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(38),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(commands, row, spec.label, 14.0, MUTED, assets);
    let controls = spawn_node(
        commands,
        row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    if spec.editable {
        add_shengji_rule_step_button(commands, controls, "‹", spec.previous, assets);
    }
    let value_box = spawn_node(
        commands,
        controls,
        Node {
            min_width: px(92),
            min_height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, value_box, spec.value, 14.0, TEXT, assets);
    if spec.editable {
        add_shengji_rule_step_button(commands, controls, "›", spec.next, assets);
    }
    add_rule_help(commands, controls, spec.help, assets);
}

pub(super) struct MahjongRuleConfigRow<'a> {
    pub(super) label: &'a str,
    pub(super) value: String,
    pub(super) help: &'a str,
    pub(super) editable: bool,
    pub(super) previous: Option<MahjongRuleSet>,
    pub(super) next: Option<MahjongRuleSet>,
}

pub(super) fn add_mahjong_rule_config_row(
    commands: &mut Commands,
    parent: Entity,
    spec: MahjongRuleConfigRow,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(38),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(commands, row, spec.label, 14.0, MUTED, assets);
    let controls = spawn_node(
        commands,
        row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    if spec.editable {
        add_mahjong_rule_step_button(commands, controls, "‹", spec.previous, assets);
    }
    let value_box = spawn_node(
        commands,
        controls,
        Node {
            min_width: px(92),
            min_height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, value_box, spec.value, 14.0, TEXT, assets);
    if spec.editable {
        add_mahjong_rule_step_button(commands, controls, "›", spec.next, assets);
    }
    add_rule_help(commands, controls, spec.help, assets);
}

fn add_shengji_rule_step_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    rules: Option<ShengjiRuleSet>,
    assets: &UiAssets,
) {
    let normal = Color::srgb(0.46, 0.60, 0.74);
    let node = Node {
        width: px(28),
        height: px(28),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(5)),
        ..default()
    };
    let entity = if let Some(rules) = rules {
        commands
            .spawn((
                Button,
                UiAction::UpdateShengjiRules(rules),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.64, 0.76, 0.88),
                    pressed: Color::srgb(0.30, 0.44, 0.58),
                },
                node,
                ImageNode::new(assets.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id()
    } else {
        commands
            .spawn((node, BackgroundColor(HEADER_BG.with_alpha(0.55))))
            .id()
    };
    commands.entity(parent).add_child(entity);
    add_text(
        commands,
        entity,
        label,
        18.0,
        if rules.is_some() {
            TEXT
        } else {
            MUTED.with_alpha(0.35)
        },
        assets,
    );
}

fn add_texas_rule_step_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    rules: Option<TexasHoldemRuleSet>,
    assets: &UiAssets,
) {
    let normal = Color::srgb(0.46, 0.60, 0.74);
    let node = Node {
        width: px(28),
        height: px(28),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(5)),
        ..default()
    };
    let entity = if let Some(rules) = rules {
        commands
            .spawn((
                Button,
                UiAction::UpdateTexasRules(rules),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.64, 0.76, 0.88),
                    pressed: Color::srgb(0.30, 0.44, 0.58),
                },
                node,
                ImageNode::new(assets.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id()
    } else {
        commands
            .spawn((node, BackgroundColor(HEADER_BG.with_alpha(0.55))))
            .id()
    };
    commands.entity(parent).add_child(entity);
    add_text(
        commands,
        entity,
        label,
        18.0,
        if rules.is_some() {
            TEXT
        } else {
            MUTED.with_alpha(0.35)
        },
        assets,
    );
}

fn add_mahjong_rule_step_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    rules: Option<MahjongRuleSet>,
    assets: &UiAssets,
) {
    let normal = Color::srgb(0.46, 0.60, 0.74);
    let node = Node {
        width: px(28),
        height: px(28),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(5)),
        ..default()
    };
    let entity = if let Some(rules) = rules {
        commands
            .spawn((
                Button,
                UiAction::UpdateMahjongRules(rules),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.64, 0.76, 0.88),
                    pressed: Color::srgb(0.30, 0.44, 0.58),
                },
                node,
                ImageNode::new(assets.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id()
    } else {
        commands
            .spawn((node, BackgroundColor(HEADER_BG.with_alpha(0.55))))
            .id()
    };
    commands.entity(parent).add_child(entity);
    add_text(
        commands,
        entity,
        label,
        18.0,
        if rules.is_some() {
            TEXT
        } else {
            MUTED.with_alpha(0.35)
        },
        assets,
    );
}

fn add_uno_rule_step_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    rules: Option<UnoRuleSet>,
    assets: &UiAssets,
) {
    let normal = Color::srgb(0.46, 0.60, 0.74);
    let node = Node {
        width: px(28),
        height: px(28),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(5)),
        ..default()
    };
    let entity = if let Some(rules) = rules {
        commands
            .spawn((
                Button,
                UiAction::UpdateUnoRules(rules),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.64, 0.76, 0.88),
                    pressed: Color::srgb(0.30, 0.44, 0.58),
                },
                node,
                ImageNode::new(assets.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id()
    } else {
        commands
            .spawn((node, BackgroundColor(HEADER_BG.with_alpha(0.55))))
            .id()
    };
    commands.entity(parent).add_child(entity);
    add_text(
        commands,
        entity,
        label,
        18.0,
        if rules.is_some() {
            TEXT
        } else {
            MUTED.with_alpha(0.35)
        },
        assets,
    );
}

fn add_rule_step_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    rules: Option<RuleSet>,
    assets: &UiAssets,
) {
    let normal = Color::srgb(0.46, 0.60, 0.74);
    let node = Node {
        width: px(28),
        height: px(28),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(5)),
        ..default()
    };
    let entity = if let Some(rules) = rules {
        commands
            .spawn((
                Button,
                UiAction::UpdateRules(rules),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.64, 0.76, 0.88),
                    pressed: Color::srgb(0.30, 0.44, 0.58),
                },
                node,
                ImageNode::new(assets.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id()
    } else {
        commands
            .spawn((node, BackgroundColor(HEADER_BG.with_alpha(0.55))))
            .id()
    };
    commands.entity(parent).add_child(entity);
    add_text(
        commands,
        entity,
        label,
        18.0,
        if rules.is_some() {
            TEXT
        } else {
            MUTED.with_alpha(0.35)
        },
        assets,
    );
}

pub(in crate::app) fn add_rule_help(
    commands: &mut Commands,
    parent: Entity,
    help: &str,
    assets: &UiAssets,
) {
    let question = commands
        .spawn((
            Button,
            Node {
                width: px(22),
                height: px(22),
                position_type: PositionType::Relative,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor::all(MUTED.with_alpha(0.8)),
        ))
        .id();
    commands.entity(parent).add_child(question);
    add_text(commands, question, "?", 14.0, MUTED, assets);

    let tooltip = spawn_node(
        commands,
        question,
        Node {
            position_type: PositionType::Absolute,
            right: px(28),
            top: px(-7),
            width: px(270),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(HEADER_BG),
    );
    commands.entity(tooltip).insert((
        Visibility::Hidden,
        BorderColor::all(ACCENT.with_alpha(0.7)),
        GlobalZIndex(1000),
    ));
    add_text(commands, tooltip, help, 12.0, TEXT, assets);
    commands.entity(question).insert(RuleHelp { tooltip });
}

pub(super) fn add_text(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> Entity {
    let entity = commands
        .spawn((
            Text::new(text),
            TextFont::from_font_size(size).with_font(assets.font.clone()),
            TextColor(color),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub(super) fn decorate_player_panel(
    commands: &mut Commands,
    panel: Entity,
    assets: &UiAssets,
    scale: f32,
) {
    let image = if scale < 1.0 {
        assets.player_panel_compact.clone()
    } else {
        assets.player_panel_wide.clone()
    };
    let texture = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            ImageNode::new(image)
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgba(1.0, 1.0, 1.0, 0.82)),
            ZIndex(-1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(panel).add_child(texture);
}

/// 人物框靠牌桌内侧的大号数值区域。七鬼五二三用于本局得分，德州用于剩余筹码。
pub(super) fn add_player_panel_primary_value(
    commands: &mut Commands,
    badge: Entity,
    side: SeatSide,
    value: impl ToString,
    assets: &UiAssets,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        top: px(0),
        bottom: px(0),
        width: px(66),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    match side {
        SeatSide::Left | SeatSide::Top => node.right = px(2),
        SeatSide::Right => node.left = px(2),
    }
    let area = spawn_node(commands, badge, node, None);
    commands.entity(area).insert((ZIndex(2), FocusPolicy::Pass));
    let text = add_text(commands, area, value.to_string(), 28.0, ACCENT, assets);
    commands.entity(text).insert(TextShadow {
        offset: Vec2::new(1.5, 2.0),
        color: Color::BLACK.with_alpha(0.82),
    });
    text
}

pub(super) fn add_avatar(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    image: Option<&Handle<Image>>,
    size: f32,
    assets: &UiAssets,
) -> Entity {
    let mut entity = commands.spawn(Node {
        width: px(size),
        height: px(size),
        min_width: px(size),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(percent(50)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
    entity.insert(BorderColor::all(BORDER));
    if let Some(image) = image {
        entity.insert(ImageNode::new(image.clone()));
    } else {
        entity.insert(BackgroundColor(avatar_color(name)));
    }
    let entity = entity.id();
    commands.entity(parent).add_child(entity);
    if image.is_none() {
        let initial = name.chars().next().unwrap_or('玩').to_string();
        add_text(commands, entity, initial, size * 0.42, Color::WHITE, assets);
    }
    entity
}

/// 结算窗口中的下一局准备状态。保持德州扑克原有的绿色头像环和右下角对钩，
/// 让所有游戏都能在玩家点击“再来一局”后直接看到彼此的准备进度。
pub(super) fn add_ready_avatar(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    image: Option<&Handle<Image>>,
    avatar_size: f32,
    ready: bool,
    assets: &UiAssets,
) -> Entity {
    let frame_size = avatar_size + 6.0;
    let frame = spawn_node(
        commands,
        parent,
        Node {
            width: px(frame_size),
            height: px(frame_size),
            min_width: px(frame_size),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        None,
    );
    commands.entity(frame).insert(BorderColor::all(if ready {
        READY
    } else {
        MUTED.with_alpha(0.42)
    }));
    add_avatar(commands, frame, name, image, avatar_size, assets);
    if ready {
        let check_size = (avatar_size * 0.47).max(13.0);
        let check = spawn_node(
            commands,
            frame,
            Node {
                position_type: PositionType::Absolute,
                right: px(-3),
                bottom: px(-2),
                width: px(check_size),
                height: px(check_size),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(READY),
        );
        add_text(
            commands,
            check,
            "✓",
            check_size * 2.0 / 3.0,
            Color::WHITE,
            assets,
        );
    }
    frame
}

pub(super) fn add_host_crown(commands: &mut Commands, avatar: Entity, assets: &UiAssets) -> Entity {
    let crown = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(-9),
                top: px(-11),
                width: px(19),
                height: px(17),
                ..default()
            },
            ImageNode::new(assets.host_crown.clone()),
            UiTransform::from_rotation(Rot2::radians(-1.08)),
            ZIndex(40),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(avatar).add_child(crown);
    crown
}

pub(super) fn avatar_color(name: &str) -> Color {
    const COLORS: [Color; 6] = [
        Color::srgb(0.20, 0.48, 0.76),
        Color::srgb(0.65, 0.29, 0.68),
        Color::srgb(0.82, 0.35, 0.25),
        Color::srgb(0.18, 0.62, 0.48),
        Color::srgb(0.76, 0.53, 0.16),
        Color::srgb(0.35, 0.42, 0.72),
    ];
    let hash = name
        .bytes()
        .fold(0_usize, |hash, byte| hash.wrapping_mul(31) + byte as usize);
    COLORS[hash % COLORS.len()]
}

pub(super) fn suit_comparison_label(comparison: SuitComparison) -> &'static str {
    match comparison {
        SuitComparison::HighestCard => "极大法",
        SuitComparison::Lexicographic => "逐项法",
        SuitComparison::SumPoints => "记点法",
    }
}

pub(super) fn time_control_label(control: TimeControl) -> &'static str {
    match control {
        TimeControl::Unlimited => "不限时",
        TimeControl::FivePlusTen => "5 + 10",
        TimeControl::FivePlusThirty => "5 + 30",
        TimeControl::FifteenPlusThirty => "15 + 30",
        TimeControl::ThirtyPlusSixty => "30 + 60",
    }
}

pub(super) fn previous_time_control(control: TimeControl) -> Option<TimeControl> {
    match control {
        TimeControl::FivePlusTen => None,
        TimeControl::FivePlusThirty => Some(TimeControl::FivePlusTen),
        TimeControl::FifteenPlusThirty => Some(TimeControl::FivePlusThirty),
        TimeControl::ThirtyPlusSixty => Some(TimeControl::FifteenPlusThirty),
        TimeControl::Unlimited => Some(TimeControl::ThirtyPlusSixty),
    }
}

pub(super) fn next_time_control(control: TimeControl) -> Option<TimeControl> {
    match control {
        TimeControl::FivePlusTen => Some(TimeControl::FivePlusThirty),
        TimeControl::FivePlusThirty => Some(TimeControl::FifteenPlusThirty),
        TimeControl::FifteenPlusThirty => Some(TimeControl::ThirtyPlusSixty),
        TimeControl::ThirtyPlusSixty => Some(TimeControl::Unlimited),
        TimeControl::Unlimited => None,
    }
}

pub(super) fn previous_suit_comparison(comparison: SuitComparison) -> SuitComparison {
    match comparison {
        SuitComparison::HighestCard => SuitComparison::SumPoints,
        SuitComparison::Lexicographic => SuitComparison::HighestCard,
        SuitComparison::SumPoints => SuitComparison::Lexicographic,
    }
}

pub(super) fn next_suit_comparison(comparison: SuitComparison) -> SuitComparison {
    match comparison {
        SuitComparison::HighestCard => SuitComparison::Lexicographic,
        SuitComparison::Lexicographic => SuitComparison::SumPoints,
        SuitComparison::SumPoints => SuitComparison::HighestCard,
    }
}

pub(super) fn reference_level(points: i32) -> &'static str {
    if points > 1_000 {
        "下界合金"
    } else if points >= 500 {
        "钻石"
    } else if points >= 200 {
        "金"
    } else if points >= 100 {
        "红石"
    } else if points >= 50 {
        "铁"
    } else if points >= 10 {
        "铜"
    } else if points >= 0 {
        "圆石"
    } else if points >= -10 {
        "木头"
    } else if points >= -50 {
        "泥土"
    } else {
        "堆肥桶"
    }
}

pub(super) fn reference_points_label(points: i32) -> String {
    format!("等级:{}  分数:{}", reference_level(points), points)
}

pub(super) fn rejection_label(reason: &RejectReason) -> Option<String> {
    if matches!(
        reason,
        RejectReason::GameViolation(GameViolation::QiGui523(RuleViolation::NotPlayersTurn,))
    ) {
        return None;
    }
    if let RejectReason::NameTooLong { max_chars } = reason {
        return Some(format!("提示：玩家名称不能超过 {max_chars} 个字符"));
    }
    let detail = match reason {
        RejectReason::NameEmpty => "玩家名称不能为空",
        RejectReason::InvalidIdentityProof => "玩家身份签名无效，请重新生成或恢复玩家档案",
        RejectReason::InvalidAvatar => "头像数据无效",
        RejectReason::AvatarAlreadySet => "本次连接已经上传过头像",
        RejectReason::InvalidSeat => "座位编号无效",
        RejectReason::SeatTaken => "这个座位已经有人了",
        RejectReason::MustSelectSeat => "必须先选择座位",
        RejectReason::OnlyHostCanConfigure => "只有房主可以修改游戏配置",
        RejectReason::OnlyHostCanStart => "只有房主可以开始游戏",
        RejectReason::OnlyHostCanReturnToLobby => "只有房主可以返回大厅",
        RejectReason::OnlyHostCanCloseRoom => "只有房主可以关闭房间",
        RejectReason::NotEnoughPlayers { .. } => "至少需要两名玩家才能开始",
        RejectReason::WaitingForPlayers { .. } => "人数尚未到齐",
        RejectReason::PlayersNotReady { .. } => "仍有玩家没有准备",
        RejectReason::InvalidRuleConfiguration => "这组配置无法满足最多六名玩家的初始发牌",
        RejectReason::DeveloperFeatureUnavailable => "房主程序没有启用开发者功能",
        RejectReason::InvalidDeveloperHand => "开发者手牌无效",
        RejectReason::InvalidChatMessage => "聊天消息为空、过长或快捷语音无效",
        RejectReason::GameAlreadyStarted => "游戏已经开始",
        RejectReason::GameNotFinished => "游戏尚未结束",
        RejectReason::GameViolation(GameViolation::QiGui523(violation)) => match violation {
            RuleViolation::NotPlayersTurn => unreachable!("filtered above"),
            RuleViolation::MustLeadWithCards => "领出时必须出牌",
            RuleViolation::CardNotInHand => "选择的牌不在手中",
            RuleViolation::InvalidPattern => "所选牌不能组成合法牌型",
            RuleViolation::PlayDoesNotBeatCurrent => "这手牌无法压过当前牌",
            RuleViolation::GameAlreadyFinished => "游戏已经结束",
            RuleViolation::InvalidPlayer => "玩家身份无效",
        },
        RejectReason::GameViolation(GameViolation::TexasHoldem(violation)) => match violation {
            TexasHoldemViolation::InvalidPlayer => "玩家身份无效",
            TexasHoldemViolation::NotPlayersTurn => "还没有轮到你行动",
            TexasHoldemViolation::HandAlreadyComplete => "这一手已经结束",
            TexasHoldemViolation::PlayerCannotAct => "弃牌、全下或离线玩家不能继续行动",
            TexasHoldemViolation::MustPostBlind => "当前只能下盲注",
            TexasHoldemViolation::NoBlindToPost => "当前没有需要下的盲注",
            TexasHoldemViolation::CannotCheckWhileFacingBet { .. } => "面对下注时不能过牌",
            TexasHoldemViolation::NothingToCall => "当前没有需要跟注的筹码",
            TexasHoldemViolation::RaiseMustExceedCurrentBet { .. } => "加注必须超过当前最高注",
            TexasHoldemViolation::RaiseBelowMinimum { .. } => "加注没有达到本轮最小额度",
            TexasHoldemViolation::RaiseExceedsStack { .. } => "加注额度超过了你的可用筹码",
            TexasHoldemViolation::RaiseNotReopened => "本轮下注尚未重新开放加注",
        },
        RejectReason::GameViolation(GameViolation::Shengji(violation)) => match violation {
            ShengjiViolation::InvalidPlayer => "玩家身份无效",
            ShengjiViolation::WrongPhase => "当前阶段不能执行这个操作",
            ShengjiViolation::InvalidDeclaration => "这些牌不能用于亮主或反主",
            ShengjiViolation::DeclarationCardsNotOwned => "亮出的牌不全在你的手中",
            ShengjiViolation::CounterRequiresPair => "反主必须亮出规则要求的同张牌",
            ShengjiViolation::CounterNotStronger => "只能用更强的主牌反主",
            ShengjiViolation::ProtectedSuitCanOnlyBeCounteredByNoTrump => {
                "自保后只能用无主或更多级牌反主"
            }
            ShengjiViolation::DeclarationRequiresJoker => "带王亮必须同时亮出对应颜色的王",
            ShengjiViolation::NoTrumpCannotOpen => "带王亮时无主只能用于反主",
            ShengjiViolation::NotDealer => "只有庄家可以埋底",
            ShengjiViolation::WrongBuryCount { expected, .. } => {
                return Some(format!("提示：必须埋下 {expected} 张底牌"));
            }
            ShengjiViolation::CardsNotOwned => "选择的牌不全在你的手中",
            ShengjiViolation::CrossingNotEligible => "你本局不符合五主过江条件",
            ShengjiViolation::CrossingAlreadyDecided => "你已经完成过江选择",
            ShengjiViolation::WrongCrossingCount { .. } => "五主过江和归还都必须正好选择五张牌",
            ShengjiViolation::CrossingMustIncludeAllTrumps => "过江牌必须包含你当前的全部主牌",
            ShengjiViolation::CrossingReturnNotRequired => "当前不需要你归还过江牌",
            ShengjiViolation::CrossingAlreadyReturned => "你已经归还过江牌",
            ShengjiViolation::NotBottomCopyPlayer => "当前没有轮到你抄底或重新埋底",
            ShengjiViolation::NotPlayersTurn => "还没有轮到你出牌",
            ShengjiViolation::MustLeadWithCards => "领出时必须出牌",
            ShengjiViolation::ThrowDisabled => "本房间不允许甩牌",
            ShengjiViolation::InvalidPattern => "所选牌不能组成合法牌型",
            ShengjiViolation::WrongCardCount { .. } => "跟牌张数必须与首家相同",
            ShengjiViolation::MustFollowCategory => "手中有该门牌时必须先跟该门",
            ShengjiViolation::MustFollowStructure => "必须优先跟泰坦尼克、拖拉机、三同张或对子结构",
        },
        RejectReason::GameViolation(GameViolation::Uno(violation)) => match violation {
            UnoViolation::InvalidPlayer => "玩家身份无效",
            UnoViolation::PlayerEliminated => "你已经被淘汰，不能继续操作",
            UnoViolation::NotPlayersTurn => "还没有轮到你行动",
            UnoViolation::GameAlreadyFinished => "游戏已经结束",
            UnoViolation::InitialColorChoiceRequired => "请先为起始万能牌选择颜色",
            UnoViolation::InitialColorAlreadyChosen => "当前不需要选择起始颜色",
            UnoViolation::CardNotInHand => "这张牌不在你的手中",
            UnoViolation::CardDoesNotMatch => "这张牌与当前颜色、数字或符号不匹配",
            UnoViolation::ColorRequired => "万能牌必须选择后续颜色",
            UnoViolation::UnexpectedColor => "普通牌不能指定后续颜色",
            UnoViolation::MustPlayDrawnCard => "摸牌后只能打出刚摸到的牌",
            UnoViolation::MustResolveDrawPenalty => "请先叠加、质疑或接受累计罚牌",
            UnoViolation::NoDrawPenalty => "当前没有待结算的罚牌",
            UnoViolation::CannotStack => "这张牌不能叠加到当前罚牌上",
            UnoViolation::CannotChallenge => "当前没有可质疑的万能摸四",
            UnoViolation::MustDrawBeforePassing => "必须先摸牌，才能结束回合",
            UnoViolation::MustResolveSkip => "请先叠加禁手或接受累计禁手",
            UnoViolation::NoSkipToResolve => "当前没有待结算的禁手",
            UnoViolation::UnoCalloutDisabled => "本房间没有启用 UNO 宣告与检举",
            UnoViolation::CannotCallUno => "你当前不能宣告 UNO",
            UnoViolation::MustPlayAfterUno => "喊出 UNO 后，本回合必须出牌至只剩一张",
            UnoViolation::CannotReportSelf => "不能检举自己",
            UnoViolation::PlayerNotReportable => "该玩家当前不可被检举",
            UnoViolation::CannotPlayTogether => "这些牌当前不能一次打出",
            UnoViolation::CannotJumpIn => "抢出窗口已经关闭",
            UnoViolation::DrawPileExhausted => "摸牌堆已经耗尽",
            UnoViolation::MustResolveSwapEffect => "请先完成当前换牌效果",
            UnoViolation::NoSwapEffect => "当前没有待处理的换牌效果",
            UnoViolation::InvalidSwapTargets => "请选择符合要求且互不重复的玩家",
        },
        RejectReason::GameViolation(GameViolation::Mahjong(violation)) => match violation {
            leocard_protocol::MahjongViolation::InvalidPlayer => "玩家身份无效",
            leocard_protocol::MahjongViolation::NotPlayersTurn => "还没有轮到你出牌",
            leocard_protocol::MahjongViolation::WrongPhase => "当前阶段不能执行这个操作",
            leocard_protocol::MahjongViolation::TileNotInHand => "这张牌不在你的手中",
            leocard_protocol::MahjongViolation::InvalidClaim => "当前不能这样吃、碰、杠或和",
            leocard_protocol::MahjongViolation::AlreadyResponded => "你已经响应过这张牌",
            leocard_protocol::MahjongViolation::CannotWin => "当前手牌不能和牌",
            leocard_protocol::MahjongViolation::CannotKong => "当前不能开杠",
        },
        RejectReason::WrongGame { .. } => "该命令不属于当前房间游戏",
        _ => "请求被房主拒绝",
    };
    Some(format!("提示：{detail}"))
}

pub(super) fn card_asset_path(rank: Rank, suit: Suit) -> String {
    let base = "vendor/kenney/boardgame/PNG/Cards";
    if rank == Rank::Joker {
        return if suit == Suit::Spade {
            format!("{base}/cardJokerBig.png")
        } else {
            format!("{base}/cardJoker.png")
        };
    }
    let suit = match suit {
        Suit::Spade => "Spades",
        Suit::Heart => "Hearts",
        Suit::Club => "Clubs",
        Suit::Diamond => "Diamonds",
    };
    let rank = match rank {
        Rank::Ace => "A",
        Rank::King => "K",
        Rank::Queen => "Q",
        Rank::Jack => "J",
        Rank::Ten => "10",
        Rank::Nine => "9",
        Rank::Eight => "8",
        Rank::Seven => "7",
        Rank::Six => "6",
        Rank::Five => "5",
        Rank::Four => "4",
        Rank::Three => "3",
        Rank::Two => "2",
        Rank::Joker => unreachable!(),
    };
    format!("{base}/card{suit}{rank}.png")
}
