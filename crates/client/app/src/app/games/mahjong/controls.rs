use super::layout::MahjongLayout as Layout;
use super::{
    MAHJONG_KONG_STACK_LIFT, MahjongActionPrompt, MahjongAssets, MahjongChoiceMenu,
    MahjongTileHighlight, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual, MahjongUiAction,
    MahjongUiState, add_mahjong_tile_material,
};
use crate::app::presentation::mahjong::own_overlay;
use crate::app::presentation::{GinkgoButton, GinkgoButtonTone, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use bevy::ui_widgets::Button;
use leocard_mahjong::{MahjongClaim, MahjongClaimOption, MahjongMeldKind, MahjongTileKind};
use leocard_protocol::MahjongSnapshot;
use leocard_ui::{BackgroundButtonTint, ButtonTint, UiComposer, spawn_node};

struct TileChoice {
    action: MahjongUiAction,
    label: &'static str,
    tiles: [MahjongTileKind; 4],
    count: usize,
}

pub(super) fn render_action_bar(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    ui: &MahjongUiState,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let Some(prompt) = MahjongActionPrompt::from_snapshot(game, ui) else {
        return;
    };
    let overlay = own_overlay(commands, table);
    let stack_clearance = if game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
        .is_some_and(|player| {
            player
                .melds
                .iter()
                .any(|meld| matches!(meld.kind, MahjongMeldKind::Kong(_)))
        }) {
        MAHJONG_KONG_STACK_LIFT
    } else {
        0.0
    };
    let bar = commands.keyed_child(
        overlay,
        format!("mahjong-actions:{prompt:?}"),
        Node {
            position_type: PositionType::Absolute,
            left: px(if cfg!(target_os = "android") { 80 } else { 200 }),
            right: px(if cfg!(target_os = "android") { 80 } else { 200 }),
            bottom: px(Layout::OWN_BOTTOM
                + Layout::OWN_HAND_LIFT
                + Layout::OWN_HEIGHT
                + 12.0
                + stack_clearance),
            min_height: px(Layout::BUTTON_HEIGHT),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            overflow: Overflow::visible(),
            ..default()
        },
    );
    commands.entity(bar).insert(prompt);
    if let Some(pending) = &game.pending_claim {
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
                GinkgoButtonTone::Blue,
            ),
            (
                MahjongClaimOption::Kong,
                "杠",
                MahjongClaim::Kong,
                GinkgoButtonTone::Plum,
            ),
            (
                MahjongClaimOption::Win,
                "和",
                MahjongClaim::Win,
                GinkgoButtonTone::Cinnabar,
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
                );
            }
        }
        add_mahjong_button(
            commands,
            bar,
            "过",
            MahjongUiAction::Respond(MahjongClaim::Pass),
            GinkgoButtonTone::Ash,
            assets,
        );
        return;
    }
    if !game.ready_discards.is_empty() {
        if ui.declaring_ready {
            add_text(commands, bar, "选牌报听", 18.0, TEXT, assets);
        }
        add_mahjong_button(
            commands,
            bar,
            if ui.declaring_ready {
                "取消"
            } else {
                "报听"
            },
            MahjongUiAction::ToggleDeclareReady,
            if ui.declaring_ready {
                GinkgoButtonTone::Ash
            } else {
                GinkgoButtonTone::Amber
            },
            assets,
        );
        if ui.declaring_ready {
            return;
        }
    }
    if game.can_self_draw {
        add_mahjong_button(
            commands,
            bar,
            "自摸",
            MahjongUiAction::SelfDraw,
            GinkgoButtonTone::Cinnabar,
            assets,
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
    commands: &mut UiComposer,
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
    let tone = match menu {
        MahjongChoiceMenu::Chow { .. } => GinkgoButtonTone::Moss,
        MahjongChoiceMenu::Kong { .. } => GinkgoButtonTone::Plum,
    };
    let anchor = add_mahjong_button(commands, bar, label, action, tone, assets);
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
            left: percent(50),
            bottom: px(Layout::BUTTON_HEIGHT + 8.0),
            width: px(if cfg!(target_os = "android") {
                320
            } else {
                240
            }),
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
        },
        None,
    );
    commands.entity(popup).insert((
        image,
        GlobalZIndex(1500),
        UiTransform::from_translation(Val2::percent(-50.0, 0.0)),
    ));
    for choice in choices {
        let row = commands.child(
            popup,
            (
                Button,
                UiAction::Mahjong(choice.action),
                Node {
                    width: percent(100),
                    height: px(if cfg!(target_os = "android") { 76 } else { 57 }),
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
            ),
        );
        let tiles = spawn_node(
            commands,
            row,
            Node {
                height: px(if cfg!(target_os = "android") { 62 } else { 45 }),
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        commands.entity(tiles).insert(Pickable::IGNORE);
        for (index, &kind) in choice.tiles[..choice.count].iter().enumerate() {
            let tile = add_mahjong_tile_material(
                commands,
                tiles,
                MahjongTileVisual {
                    elevation: 0.0,
                    kind: Some(kind),
                    size: MahjongTileSize::GuideHand,
                    index,
                    highlight: MahjongTileHighlight::None,
                    deal: None,
                    relative: 0,
                },
                game_assets,
                materials,
            );
            commands.entity(tile).insert(Pickable::IGNORE);
        }
        let text = add_text(commands, row, choice.label, 15.0, TEXT, assets);
        commands.entity(text).insert(Pickable::IGNORE);
    }
}

fn add_mahjong_button(
    commands: &mut UiComposer,
    bar: Entity,
    label: &str,
    action: MahjongUiAction,
    tone: GinkgoButtonTone,
    assets: &UiAssets,
) -> Entity {
    let anchor = spawn_node(
        commands,
        bar,
        Node {
            min_width: px(Layout::BUTTON_WIDTH),
            height: px(Layout::BUTTON_HEIGHT),
            flex_shrink: 0.0,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    GinkgoButton {
        label,
        action: UiAction::Mahjong(action),
        tone,
        size: Vec2::new(Layout::BUTTON_WIDTH, Layout::BUTTON_HEIGHT),
        font_size: Layout::BUTTON_FONT,
    }
    .mount(commands, anchor, assets);
    anchor
}
