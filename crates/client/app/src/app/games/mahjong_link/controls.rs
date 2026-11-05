use super::{MahjongLinkUiAction, state::MahjongLinkUiState, table::can_act};
use crate::app::presentation::{
    GAME_SIDE_CONTROL_SCALE, GameButtonSpec, GameButtonTone, MUTED, TEXT, add_text,
    add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
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
            left: px(220),
            right: px(220),
            bottom: px(18),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    commands.entity(controls).insert(ZIndex(100));
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
}
