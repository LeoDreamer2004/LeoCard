use super::{BlackjackTable, BlackjackUiAction};
use crate::app::presentation::{
    ACCENT, GameButtonImageMode, GameButtonSpec, GameButtonTone, PanelSkin, TEXT, UiComposer,
    add_text, add_textured_game_button, decorate_panel_skin, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{LobbyUiAction, UiAction, add_cozy_button};
use bevy::prelude::*;
use leocard_blackjack::{BlackjackAction, Phase};

impl BlackjackTable<'_> {
    pub(super) fn controls(&self, commands: &mut UiComposer, table: Entity) {
        if self.game.match_complete || self.playback.busy() {
            return;
        }
        if self.game.phase != Phase::Complete && self.game.current_player != Some(self.game.you) {
            return;
        }
        let controls = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(172),
                width: px(480),
                height: px(50),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(7),
                ..default()
            },
            None,
        );
        commands
            .entity(controls)
            .insert(UiTransform::from_translation(Val2::px(-240.0, 0.0)));
        let row = spawn_node(
            commands,
            controls,
            Node {
                width: percent(100),
                height: px(50),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            None,
        );
        if self.game.phase == Phase::Complete {
            let own = self
                .game
                .players
                .iter()
                .find(|view| view.player.id == self.game.you)
                .expect("recipient");
            add_text(
                commands,
                row,
                format!(
                    "本局 {:+}",
                    i64::from(own.stack) - i64::from(own.round_start_stack)
                ),
                21.0,
                ACCENT,
                self.assets,
            );
            action_button(
                commands,
                row,
                if own.player.ready {
                    "已准备"
                } else {
                    "准备下一局"
                },
                (!own.player.ready).then_some(UiAction::Lobby(LobbyUiAction::PlayAgain)),
                GameButtonTone::Play,
                134.0,
                self.assets,
            );
            if self.game.you == self.game.host {
                action_button(
                    commands,
                    row,
                    "返回房间",
                    Some(UiAction::Lobby(LobbyUiAction::ReturnToLobby)),
                    GameButtonTone::Hint,
                    120.0,
                    self.assets,
                );
            }
        } else if self.game.available.minimum_bet > 0 {
            self.bet_controls(commands, controls, row);
        } else {
            for (label, action, enabled, tone) in [
                (
                    "要牌",
                    BlackjackAction::Hit,
                    self.game.available.hit,
                    GameButtonTone::Play,
                ),
                (
                    "停牌",
                    BlackjackAction::Stand,
                    self.game.available.stand,
                    GameButtonTone::Pass,
                ),
                (
                    "加倍",
                    BlackjackAction::DoubleDown,
                    self.game.available.double_down,
                    GameButtonTone::Hint,
                ),
                (
                    "分牌",
                    BlackjackAction::Split,
                    self.game.available.split,
                    GameButtonTone::Hint,
                ),
            ] {
                action_button(
                    commands,
                    row,
                    label,
                    enabled.then_some(UiAction::Blackjack(BlackjackUiAction::Act(action))),
                    tone,
                    110.0,
                    self.assets,
                );
            }
        }
    }

    fn bet_controls(&self, commands: &mut UiComposer, controls: Entity, row: Entity) {
        let available = self.game.available;
        let amount = self.ui.bet;
        let step = (self.game.rules.minimum_bet / 2).max(1) * 2;
        for (increase, next) in [
            (
                false,
                amount.saturating_sub(step).max(available.minimum_bet),
            ),
            (true, amount.saturating_add(step).min(available.maximum_bet)),
        ] {
            let (normal, hovered) = if increase {
                (
                    &self.assets.controls.adjust_right,
                    &self.assets.controls.adjust_right_hover,
                )
            } else {
                (
                    &self.assets.controls.adjust_left,
                    &self.assets.controls.adjust_left_hover,
                )
            };
            add_textured_game_button(
                commands,
                row,
                self.assets,
                GameButtonSpec {
                    label: "",
                    action: (next != amount)
                        .then_some(UiAction::Blackjack(BlackjackUiAction::SetBet(next))),
                    normal,
                    hovered,
                    width: 44.0,
                    height: 50.0,
                    font_size: 17.0,
                    image_mode: GameButtonImageMode::Stretch,
                },
            );
            if !increase {
                action_button(
                    commands,
                    row,
                    &format!("下注 {amount}"),
                    Some(UiAction::Blackjack(BlackjackUiAction::Act(
                        BlackjackAction::Bet { amount },
                    ))),
                    GameButtonTone::Play,
                    170.0,
                    self.assets,
                );
            }
        }
        let shortcuts = spawn_node(
            commands,
            controls,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(57),
                height: px(34),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            },
            None,
        );
        let mut bets = vec![available.minimum_bet, 10, 20, 50, available.maximum_bet];
        bets.sort_unstable();
        bets.dedup();
        for bet in bets
            .into_iter()
            .filter(|bet| *bet >= available.minimum_bet && *bet <= available.maximum_bet)
        {
            let button = action_button(
                commands,
                shortcuts,
                &bet.to_string(),
                Some(UiAction::Blackjack(BlackjackUiAction::SetBet(bet))),
                GameButtonTone::Hint,
                64.0,
                self.assets,
            );
            commands
                .entity(button)
                .entry::<Node>()
                .and_modify(|mut node| node.height = px(34));
        }
    }

    pub(super) fn guide(&self, commands: &mut UiComposer, content: Entity) {
        let panel = spawn_node(
            commands,
            content,
            Node {
                position_type: PositionType::Absolute,
                right: px(70),
                bottom: px(90),
                width: px(440),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        commands.entity(panel).insert(GlobalZIndex(1900));
        decorate_panel_skin(commands, panel, PanelSkin::Window, self.assets);
        add_text(commands, panel, "21 点", 25.0, ACCENT, self.assets);
        for text in [
            "A 计为 1 或 11；J、Q、K 计为 10。超过 21 点即爆牌。",
            "每人分别与系统庄家比较；庄家不足 17 点要牌，软 17 停牌。同点退还下注。",
            "起手 A 加 10 点牌为黑杰克，净赢下注的 3:2；普通获胜净赢 1:1。",
            "两张牌时可以加倍下注，只再补一张牌后停牌。",
            "点数相同的两张牌可以分牌，最多四手，允许分牌后加倍。分 A 后每手只补一张，21 点按普通手牌比较。",
            "每小局重新洗六副牌；下注须为偶数，上限为初始筹码，且不能超过剩余筹码。",
        ] {
            add_text(commands, panel, text, 16.0, TEXT, self.assets);
        }
        add_cozy_button(
            commands,
            panel,
            "关闭",
            UiAction::Blackjack(BlackjackUiAction::ToggleGuide),
            self.assets,
            percent(100),
            42.0,
        );
    }
}

fn action_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: Option<UiAction>,
    tone: GameButtonTone,
    width: f32,
    assets: &UiAssets,
) -> Entity {
    let mut spec = GameButtonSpec::action(assets, label, action, tone);
    spec.width = width;
    spec.height = 50.0;
    spec.font_size = 17.0;
    spec.image_mode = GameButtonImageMode::Sliced {
        border: 30.0,
        corner_scale: 0.42,
    };
    add_textured_game_button(commands, parent, assets, spec).0
}
