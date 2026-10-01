use super::{
    MahjongAssets, MahjongChoiceMenu, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    MahjongUiAction, MahjongUiState, add_mahjong_tile_material,
};
use crate::app::presentation::ButtonHighlight;
use crate::app::presentation::{BackgroundButtonTint, ButtonTint, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_mahjong::{MahjongClaim, MahjongClaimOption, MahjongTileKind};
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

#[derive(Clone, Copy)]
enum ActionTone {
    Normal,
    Pass,
    Win,
}

struct TileChoice {
    action: MahjongUiAction,
    label: &'static str,
    tiles: [MahjongTileKind; 4],
    count: usize,
    claimed_index: Option<usize>,
}

pub(super) fn render_action_bar(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    ui: &MahjongUiState,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    if ui.no_claim
        && game
            .pending_claim
            .as_ref()
            .is_some_and(|pending| !pending.your_options.contains(&MahjongClaimOption::Win))
    {
        return;
    }
    let bar = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(240),
            right: px(240),
            bottom: px(104),
            min_height: px(54),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    if let Some(pending) = &game.pending_claim {
        if pending.your_response.is_some() {
            return;
        }
        if pending.your_options.is_empty() {
            return;
        }
        let chows = pending
            .your_options
            .iter()
            .filter_map(|option| {
                let MahjongClaimOption::Chow { start } = *option else {
                    return None;
                };
                let MahjongTileKind::Suited { suit, rank } = pending.tile.kind() else {
                    return None;
                };
                if !(start..=start + 2).contains(&rank) {
                    return None;
                }
                Some(TileChoice {
                    action: MahjongUiAction::Respond(MahjongClaim::Chow { start }),
                    label: "吃",
                    tiles: [
                        MahjongTileKind::suited(suit, start),
                        MahjongTileKind::suited(suit, start + 1),
                        MahjongTileKind::suited(suit, start + 2),
                        MahjongTileKind::suited(suit, start + 2),
                    ],
                    count: 3,
                    claimed_index: Some(usize::from(rank - start)),
                })
            })
            .collect::<Vec<_>>();
        if !ui.no_claim && !chows.is_empty() {
            add_choice_button(
                commands,
                bar,
                "吃",
                MahjongChoiceMenu::Chow {
                    match_id: game.match_id,
                    source: pending.source,
                    tile: pending.tile,
                },
                chows,
                ui,
                assets,
                game_assets,
                materials,
            );
        }
        for (option, label, claim, tone) in [
            (
                MahjongClaimOption::Pung,
                "碰",
                MahjongClaim::Pung,
                ActionTone::Normal,
            ),
            (
                MahjongClaimOption::Kong,
                "杠",
                MahjongClaim::Kong,
                ActionTone::Normal,
            ),
            (
                MahjongClaimOption::Win,
                "和",
                MahjongClaim::Win,
                ActionTone::Win,
            ),
        ] {
            if pending.your_options.contains(&option)
                && (!ui.no_claim || option == MahjongClaimOption::Win)
            {
                add_mahjong_button(
                    commands,
                    bar,
                    label,
                    MahjongUiAction::Respond(claim),
                    tone,
                    assets,
                    game_assets,
                );
            }
        }
        add_mahjong_button(
            commands,
            bar,
            "过",
            MahjongUiAction::Respond(MahjongClaim::Pass),
            ActionTone::Pass,
            assets,
            game_assets,
        );
        return;
    }
    if !matches!(game.phase, MahjongPhaseView::Playing) || game.current_player != game.you {
        return;
    }
    if game.can_self_draw {
        add_mahjong_button(
            commands,
            bar,
            "自摸",
            MahjongUiAction::SelfDraw,
            ActionTone::Win,
            assets,
            game_assets,
        );
    }
    let mut kongs = game
        .concealed_kong_options
        .iter()
        .copied()
        .map(|kind| TileChoice {
            action: MahjongUiAction::ConcealedKong(kind),
            label: "暗杠",
            tiles: [kind; 4],
            count: 4,
            claimed_index: None,
        })
        .collect::<Vec<_>>();
    if !ui.no_claim {
        kongs.extend(
            game.added_kong_options
                .iter()
                .copied()
                .map(|tile| TileChoice {
                    action: MahjongUiAction::AddedKong(tile),
                    label: "加杠",
                    tiles: [tile.kind(); 4],
                    count: 4,
                    claimed_index: None,
                }),
        );
    }
    if !kongs.is_empty() {
        add_choice_button(
            commands,
            bar,
            "杠",
            MahjongChoiceMenu::Kong {
                match_id: game.match_id,
                sequence_index: game.sequence_index,
                hand_len: game.your_hand.len(),
            },
            kongs,
            ui,
            assets,
            game_assets,
            materials,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the popup renders tile materials as well as controls"
)]
fn add_choice_button(
    commands: &mut Commands,
    bar: Entity,
    label: &str,
    menu: MahjongChoiceMenu,
    choices: Vec<TileChoice>,
    ui: &MahjongUiState,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let action = if choices.len() == 1 {
        choices[0].action.clone()
    } else {
        MahjongUiAction::ToggleChoiceMenu(menu)
    };
    let anchor = add_mahjong_button(
        commands,
        bar,
        label,
        action,
        ActionTone::Normal,
        assets,
        game_assets,
    );
    if choices.len() < 2 || ui.choice_menu != Some(menu) {
        return;
    }
    let mut image = ImageNode::new(assets.home.game_card.clone()).with_mode(NodeImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::all(22.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.55,
        },
    ));
    image.visual_box = VisualBox::BorderBox;
    let popup = spawn_node(
        commands,
        anchor,
        Node {
            position_type: PositionType::Absolute,
            left: px(-38),
            bottom: px(62),
            width: px(240),
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
        },
        None,
    );
    commands.entity(popup).insert((image, GlobalZIndex(1500)));
    for choice in choices {
        let row = commands
            .spawn((
                Button,
                UiAction::Mahjong(choice.action),
                Node {
                    width: percent(100),
                    height: px(57),
                    padding: UiRect::axes(px(9), px(5)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.17, 0.16, 0.22, 0.85)),
                BackgroundButtonTint,
                ButtonTint {
                    normal: Color::srgba(0.17, 0.16, 0.22, 0.85),
                    hovered: Color::srgba(0.31, 0.27, 0.43, 0.95),
                    pressed: Color::srgba(0.40, 0.35, 0.56, 0.95),
                },
            ))
            .id();
        commands.entity(popup).add_child(row);
        let tiles = spawn_node(
            commands,
            row,
            Node {
                height: px(45),
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        commands.entity(tiles).insert(FocusPolicy::Pass);
        for index in 0..choice.count {
            let tile = add_mahjong_tile_material(
                commands,
                tiles,
                MahjongTileVisual {
                    kind: Some(choice.tiles[index]),
                    size: MahjongTileSize::GuideHand,
                    index,
                    highlighted: choice.claimed_index == Some(index),
                    deal: None,
                    relative: 0,
                },
                game_assets,
                materials,
            );
            commands.entity(tile).insert(FocusPolicy::Pass);
        }
        let text = add_text(commands, row, choice.label, 15.0, TEXT, assets);
        commands.entity(text).insert(FocusPolicy::Pass);
    }
}

fn add_mahjong_button(
    commands: &mut Commands,
    bar: Entity,
    label: &str,
    action: MahjongUiAction,
    tone: ActionTone,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
) -> Entity {
    let anchor = spawn_node(
        commands,
        bar,
        Node {
            width: px(164),
            height: px(54),
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let (normal, hovered) = match tone {
        ActionTone::Normal => (&game_assets.action_button, &game_assets.action_button_hover),
        ActionTone::Pass => (&game_assets.action_pass, &game_assets.action_pass_hover),
        ActionTone::Win => (&game_assets.action_win, &game_assets.action_win_hover),
    };
    let button = commands
        .spawn((
            Button,
            UiAction::Mahjong(action),
            Node {
                width: px(164),
                height: px(54),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(normal.clone()).with_mode(NodeImageMode::Stretch),
        ))
        .id();
    commands.entity(anchor).add_child(button);
    let overlay = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            ImageNode::new(hovered.clone()).with_mode(NodeImageMode::Stretch),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(overlay);
    let text = add_text(commands, button, label, 20.0, TEXT, assets);
    commands.entity(text).insert(FocusPolicy::Pass);
    commands.entity(button).insert(ButtonHighlight::Button {
        overlay,
        arrows: None,
    });
    anchor
}
