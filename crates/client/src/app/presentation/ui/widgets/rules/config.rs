use super::*;

pub trait EditableRuleSet: Copy {
    const ROW_HEIGHT: f32;
    const VALUE_WIDTH: f32;

    fn update_action(self) -> UiAction;
}

macro_rules! editable_rule_set {
    ($rules:ty, $action:path, $row_height:expr, $value_width:expr) => {
        impl EditableRuleSet for $rules {
            const ROW_HEIGHT: f32 = $row_height;
            const VALUE_WIDTH: f32 = $value_width;

            fn update_action(self) -> UiAction {
                $action(self)
            }
        }
    };
}

editable_rule_set!(QiGuiRuleSet, UiAction::UpdateRules, 34.0, 68.0);
editable_rule_set!(TexasHoldemRuleSet, UiAction::UpdateTexasRules, 38.0, 78.0);
editable_rule_set!(UnoRuleSet, UiAction::UpdateUnoRules, 38.0, 78.0);
editable_rule_set!(ShengjiRuleSet, UiAction::UpdateShengjiRules, 38.0, 92.0);
editable_rule_set!(MahjongRuleSet, UiAction::UpdateMahjongRules, 38.0, 92.0);

pub struct RuleConfigRow<'a, R> {
    pub label: &'a str,
    pub value: String,
    pub help: &'a str,
    pub editable: bool,
    pub previous: Option<R>,
    pub next: Option<R>,
}

pub type TexasRuleConfigRow<'a> = RuleConfigRow<'a, TexasHoldemRuleSet>;
pub type UnoRuleConfigRow<'a> = RuleConfigRow<'a, UnoRuleSet>;
pub type ShengjiRuleConfigRow<'a> = RuleConfigRow<'a, ShengjiRuleSet>;
pub type MahjongRuleConfigRow<'a> = RuleConfigRow<'a, MahjongRuleSet>;

pub fn add_rule_config_row<R: EditableRuleSet>(
    commands: &mut Commands,
    parent: Entity,
    spec: RuleConfigRow<'_, R>,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(R::ROW_HEIGHT),
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
        add_rule_step_button(commands, controls, "‹", spec.previous, assets);
    }
    let value_box = spawn_node(
        commands,
        controls,
        Node {
            min_width: px(R::VALUE_WIDTH),
            min_height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, value_box, spec.value, 14.0, TEXT, assets);
    if spec.editable {
        add_rule_step_button(commands, controls, "›", spec.next, assets);
    }
    add_rule_help(commands, controls, spec.help, assets);
}

fn add_rule_step_button<R: EditableRuleSet>(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    rules: Option<R>,
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
                rules.update_action(),
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.64, 0.76, 0.88),
                    pressed: Color::srgb(0.30, 0.44, 0.58),
                },
                node,
                ImageNode::new(assets.controls.secondary_button.clone())
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
