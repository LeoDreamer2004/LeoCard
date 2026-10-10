//! Complete-match standings and continuation actions.

use super::*;
use bevy::picking::Pickable;

use crate::app::presentation::{
    ACCENT, DANGER, PanelSkin, READY, TEXT, add_avatar, add_text, decorate_panel_skin, spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{
    CozyButtonVariant, LobbyUiAction, UiAction, add_cozy_button, add_cozy_button_variant,
    add_cozy_disabled_button,
};
use bevy::prelude::*;
use leocard_mahjong::MahjongMatchLength;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot};

pub(super) struct FinalSummaryView<'a> {
    pub game: &'a MahjongSnapshot,
    pub result: &'a MahjongHandResultView,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
    pub opened_at: f32,
}

impl FinalSummaryView<'_> {
    pub(super) fn render(self, commands: &mut Commands, table: Entity) {
        let Self {
            game,
            result,
            assets,
            avatars,
            opened_at,
        } = self;
        let summary = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(270),
                right: px(270),
                top: px(114),
                padding: UiRect::axes(px(24), px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            None,
        );
        commands
            .entity(summary)
            .insert((GlobalZIndex(1201), Pickable::IGNORE));
        decorate_panel_skin(commands, summary, PanelSkin::Window, assets);
        add_text(commands, summary, "整局结算", 26.0, ACCENT, assets);
        spawn_node(
            commands,
            summary,
            Node {
                width: percent(100),
                height: px(2),
                ..default()
            },
            Some(ACCENT.with_alpha(0.55)),
        );
        let mut ranked = game.players.iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            result.match_scores[right.id.0 as usize]
                .cmp(&result.match_scores[left.id.0 as usize])
                .then_with(|| left.seat.0.cmp(&right.seat.0))
        });
        for (place, player) in ranked.iter().enumerate() {
            let row = spawn_node(
                commands,
                summary,
                Node {
                    width: percent(100),
                    height: px(61),
                    padding: UiRect::axes(px(13), px(7)),
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    ..default()
                },
                None,
            );
            commands.entity(row).insert((
                settlement_row_texture(assets),
                MahjongFinalRow {
                    opened_at,
                    index: place,
                },
                UiTransform::default(),
                Visibility::Hidden,
                Pickable::IGNORE,
            ));
            add_text(
                commands,
                row,
                format!("{}", place + 1),
                19.0,
                ACCENT,
                assets,
            );
            let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
            add_avatar(commands, row, &player.name, avatar, 39.0, assets);
            add_text(commands, row, &player.name, 17.0, TEXT, assets);
            spawn_node(
                commands,
                row,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            add_text(
                commands,
                row,
                format!("总分 {:+}", result.match_scores[usize::from(player.id.0)]),
                17.0,
                ACCENT,
                assets,
            );
            if let Some(change) = result
                .reference_changes
                .iter()
                .find(|change| change.player == player.id)
            {
                add_text(
                    commands,
                    row,
                    format!("积分 {:+}", change.delta),
                    17.0,
                    if change.delta >= 0 { READY } else { DANGER },
                    assets,
                );
            }
        }
        let actions = spawn_node(
            commands,
            summary,
            Node {
                width: percent(100),
                min_height: px(52),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            None,
        );
        commands.entity(actions).insert((
            MahjongFinalActions {
                opened_at,
                row_count: ranked.len(),
            },
            Visibility::Hidden,
        ));
        if game.rules.match_length == MahjongMatchLength::SingleHand {
            add_cozy_button_variant(
                commands,
                actions,
                "退出游戏",
                UiAction::Lobby(LobbyUiAction::LeaveRoom),
                assets,
                px(150),
                48.0,
                CozyButtonVariant::Danger,
            );
            if game.you == game.host {
                add_cozy_button(
                    commands,
                    actions,
                    "返回房间",
                    UiAction::Lobby(LobbyUiAction::ReturnToLobby),
                    assets,
                    px(150),
                    48.0,
                );
            }
            let ready = game
                .players
                .iter()
                .find(|player| player.id == game.you)
                .is_some_and(|player| player.ready);
            if ready {
                add_cozy_disabled_button(commands, actions, "已准备", assets, px(150), 48.0);
            } else {
                add_cozy_button_variant(
                    commands,
                    actions,
                    "再来一局",
                    UiAction::Lobby(LobbyUiAction::PlayAgain),
                    assets,
                    px(150),
                    48.0,
                    CozyButtonVariant::Cool,
                );
            }
        } else {
            add_cozy_button_variant(
                commands,
                actions,
                "返回大厅",
                UiAction::Lobby(LobbyUiAction::ReturnToLobby),
                assets,
                percent(100),
                48.0,
                CozyButtonVariant::Primary,
            );
        }
    }
}
