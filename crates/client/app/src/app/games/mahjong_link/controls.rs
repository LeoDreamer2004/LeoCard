use super::{MahjongLinkUiAction, state::MahjongLinkUiState, table::can_act};
use crate::app::presentation::{
    ACCENT, GAME_SIDE_CONTROL_SCALE, GameButtonSpec, GameButtonTone, MUTED, PanelSkin, TEXT,
    add_text, add_textured_game_button, add_title, decorate_panel_skin,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{UiAction, add_cozy_button};
use bevy::prelude::*;
use leocard_mahjong_link::{BOARD_COLUMNS, LinkCell, MahjongLinkAction, PushSide};
use leocard_protocol::MahjongLinkSnapshot;
use leocard_ui::{UiComposer, spawn_node};

fn action(action: MahjongLinkUiAction) -> UiAction {
    UiAction::MahjongLink(action)
}

fn button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: Option<MahjongLinkUiAction>,
    width: f32,
    tone: GameButtonTone,
    assets: &UiAssets,
) {
    let mut spec = GameButtonSpec::compact(assets, label, action.map(self::action), tone)
        .scaled(GAME_SIDE_CONTROL_SCALE);
    spec.width = width * GAME_SIDE_CONTROL_SCALE;
    spec.height = 42.0 * GAME_SIDE_CONTROL_SCALE;
    add_textured_game_button(commands, parent, assets, spec);
}

pub(super) fn render(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongLinkSnapshot,
    ui: &MahjongLinkUiState,
    assets: &UiAssets,
) {
    if game.match_complete {
        return;
    }
    let enabled = can_act(game);
    let can_push = enabled && !game.push_used;
    let selected = ui.selected;
    let distance = ui.distance.max(1);
    let controls = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(260),
            right: px(35),
            top: px(532),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    let row = spawn_node(
        commands,
        controls,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    button(
        commands,
        row,
        "−",
        (can_push && distance > 1).then_some(MahjongLinkUiAction::SetDistance(distance - 1)),
        34.0,
        GameButtonTone::Hint,
        assets,
    );
    add_text(
        commands,
        row,
        format!("{distance} 格"),
        18.0 * GAME_SIDE_CONTROL_SCALE,
        TEXT,
        assets,
    );
    button(
        commands,
        row,
        "+",
        (can_push && usize::from(distance) < BOARD_COLUMNS - 1)
            .then_some(MahjongLinkUiAction::SetDistance(distance + 1)),
        34.0,
        GameButtonTone::Hint,
        assets,
    );
    for (from, label) in [(PushSide::Left, "向右推"), (PushSide::Right, "向左推")] {
        let push = selected.and_then(|cell| {
            let columns = (0..BOARD_COLUMNS as u8)
                .filter(|column| {
                    game.board
                        .tile(LinkCell {
                            row: cell.row,
                            column: *column,
                        })
                        .is_some()
                })
                .collect::<Vec<_>>();
            let endpoint = match from {
                PushSide::Left => columns.first(),
                PushSide::Right => columns.last(),
            };
            (can_push
                && endpoint == Some(&cell.column)
                && distance <= game.board.maximum_push(cell.row, from))
            .then_some(MahjongLinkUiAction::Act(MahjongLinkAction::Push {
                row: cell.row,
                from,
                distance,
            }))
        });
        button(
            commands,
            row,
            label,
            push,
            104.0,
            GameButtonTone::Play,
            assets,
        );
    }
    button(
        commands,
        row,
        "取消",
        selected.map(|_| MahjongLinkUiAction::Clear),
        68.0,
        GameButtonTone::Hint,
        assets,
    );
    button(
        commands,
        row,
        "跳过",
        enabled.then_some(MahjongLinkUiAction::Act(MahjongLinkAction::Pass)),
        76.0,
        GameButtonTone::Pass,
        assets,
    );
    let message = if game.push_used {
        "本回合已推动一次  可继续选择两张相同牌消除，或跳过结束回合".to_owned()
    } else {
        selected.map_or_else(
            || "点两张相同牌尝试消除；推牌先点该行最左端或最右端的牌".to_owned(),
            |cell| {
                format!(
                    "第 {} 排 {}  已选中  推动距离 {} 格",
                    cell.row + 1,
                    game.board
                        .tile(cell)
                        .map(|tile| tile.to_string())
                        .unwrap_or_default(),
                    distance
                )
            },
        )
    };
    add_text(
        commands,
        controls,
        message,
        if cfg!(target_os = "android") {
            19.0
        } else {
            16.0
        },
        MUTED,
        assets,
    );
    if game.consecutive_passes > 0 {
        add_text(
            commands,
            controls,
            format!(
                "已有 {} 人连续跳过  全员连续跳过将结算",
                game.consecutive_passes
            ),
            16.0,
            ACCENT,
            assets,
        );
    }
}

pub(super) fn guide(commands: &mut UiComposer, content: Entity, assets: &UiAssets) {
    let panel = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            right: px(75),
            bottom: px(70),
            width: px(if cfg!(target_os = "android") {
                620
            } else {
                500
            }),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert(GlobalZIndex(1900));
    decorate_panel_skin(commands, panel, PanelSkin::Window, assets);
    add_title(commands, panel, "麻将连连看", 26.0, ACCENT, assets);
    for text in [
        "136 张麻将铺满 8 行 17 列。按座位轮流，每回合可直接消一对，或先推一次再消一对；消除或跳过后换人。",
        "两张相同牌以最多两次转弯的连线相连即可消除，连线可绕外沿，不能穿牌。分别向上、向下露出的相同牌也可直接配对。",
        "推牌从该行最左端向右，或最右端向左。先选择端点牌，再调格数，最后按方向按钮。碰上其他牌会连带推动，不能越界。",
        "每对只记一次牌面分：字牌 10 分，其他牌按数字得 1 至 9 分。",
        "牌消完或全员连续跳过后结算，分数最高者胜出，同分并列。",
    ] {
        add_text(
            commands,
            panel,
            text,
            if cfg!(target_os = "android") {
                22.0
            } else {
                18.0
            },
            TEXT,
            assets,
        );
    }
    add_cozy_button(
        commands,
        panel,
        "关闭",
        action(MahjongLinkUiAction::ToggleGuide),
        assets,
        percent(100),
        46.0 * GAME_SIDE_CONTROL_SCALE,
    );
}
