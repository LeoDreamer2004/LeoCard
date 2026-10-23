use super::super::MinecraftGuessUiAction;
use super::super::feedback::GuessReadyControls;
use super::super::state::GuessInput;
use super::super::turn_timer::add_guess_clock;
use super::super::view::GuessTable;
use super::buttons::{GuessButtonSpec, GuessSubmit, add_guess_button};
use super::candidates::GuessCandidates;
use crate::app::presentation::{ACCENT, TextInput, UiComposer, add_text, spawn_node};
use bevy::prelude::*;
use leocard_minecraft_guess::{GuessMode, MAX_GUESS_CHARACTERS};

impl GuessTable<'_> {
    pub(in super::super) fn controls(&self, commands: &mut UiComposer, page: Entity) {
        let controls = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                height: px(92),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                ..default()
            },
            None,
        );
        if self.transition.is_active_for(self.game.match_id) {
            commands.entity(controls).insert(Visibility::Hidden);
        }
        let row = spawn_node(
            commands,
            controls,
            Node {
                width: if self.game.graph.is_some() {
                    percent(100)
                } else {
                    Val::Auto
                },
                height: px(46),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        if self.game.round.finished() && self.game.progress.is_last_game() {
            return;
        }
        if self.game.round.finished() {
            commands.entity(row).insert((
                GuessReadyControls,
                if self.feedback.busy() || self.scores.busy() {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
            ));
            let winner = self
                .game
                .round
                .winner
                .and_then(|id| self.game.players.iter().find(|player| player.id.0 == id));
            add_text(
                commands,
                row,
                winner
                    .map(|player| format!("{} 猜中了！", player.name))
                    .unwrap_or_else(|| {
                        if self.game.graph.is_some() {
                            "本局谜图已完成".to_owned()
                        } else if self.game.rules.mode == GuessMode::Race {
                            "时间到，本局结束".to_owned()
                        } else {
                            "全员跳过，本轮结束".to_owned()
                        }
                    }),
                20.0,
                ACCENT,
                self.assets,
            );
            let ready = self
                .game
                .players
                .iter()
                .find(|player| player.id == self.game.you)
                .is_some_and(|player| player.ready);
            self.button(
                commands,
                row,
                if ready {
                    "已准备"
                } else if self.game.progress.is_last_game() {
                    "再玩一场"
                } else {
                    "准备下一局"
                },
                (!ready).then_some(MinecraftGuessUiAction::Again),
            );
            if self.game.you == self.game.host {
                self.button(
                    commands,
                    row,
                    "返回房间",
                    Some(MinecraftGuessUiAction::Lobby),
                );
            }
            return;
        }
        let active = self.state.can_guess(self.game);
        let mut input = TextInput::new("minecraft-guess", &self.state.input);
        input.placeholder = "输入条目名称";
        input.max_characters = MAX_GUESS_CHARACTERS;
        input.font_size = 18.0;
        input.skin = Some(self.game_assets.input());
        let editor = input.spawn(
            commands,
            row,
            Node {
                width: if self.game.graph.is_some() {
                    px(0)
                } else {
                    px(310)
                },
                min_width: px(120),
                flex_grow: f32::from(self.game.graph.is_some()),
                height: px(46),
                padding: UiRect::horizontal(px(12)),
                ..default()
            },
            self.assets.text_input_style(),
        );
        commands.entity(editor).insert(GuessInput);
        let submit = self.button(
            commands,
            row,
            "猜测",
            (active && !self.state.input.trim().is_empty())
                .then_some(MinecraftGuessUiAction::Guess(None)),
        );
        commands.entity(submit).insert(GuessSubmit);
        if self.game.rules.mode != GuessMode::Race {
            self.button(
                commands,
                row,
                "跳过",
                (self.game.round.current_player == Some(self.game.you.0))
                    .then_some(MinecraftGuessUiAction::Skip),
            );
        }
        add_guess_clock(
            commands,
            row,
            self.game.guess_seconds_remaining,
            self.assets,
        );
        let candidates = spawn_node(
            commands,
            controls,
            Node {
                width: percent(100),
                height: px(40),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(6),
                overflow: Overflow::clip(),
                ..default()
            },
            None,
        );
        commands
            .entity(candidates)
            // 输入同步统一持有候选按钮；页面重建不重置缓存，也不再创建第二套。
            .init(GuessCandidates::default());
    }

    fn button(
        &self,
        commands: &mut UiComposer,
        row: Entity,
        label: &str,
        action: Option<MinecraftGuessUiAction>,
    ) -> Entity {
        add_guess_button(
            commands,
            row,
            GuessButtonSpec {
                label,
                action,
                width: px(if self.game.graph.is_some() {
                    100.0
                } else {
                    125.0
                }),
                height: 46.0,
                icon: None,
            },
            self.assets,
            self.game_assets,
        )
    }
}
