use super::cards::{play_area_width, render_card_row};
use super::table::GuandanTable;
use crate::app::presentation::{MUTED, SeatSide, add_text, spawn_trick_play_area};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::GuandanPublicAction;
use leocard_ui::UiComposer;

impl GuandanTable<'_> {
    pub(super) fn play_area(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        player: PlayerId,
        side: SeatSide,
    ) {
        let area = spawn_trick_play_area(commands, parent, side);
        commands
            .entity(area)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.width = px(play_area_width(side));
                node.min_width = node.width;
            });
        let previous = self.ui.previous_reveal > 0.0;
        let actions = if previous {
            self.game
                .previous_trick
                .as_ref()
                .unwrap_or(&self.game.last_actions)
        } else {
            &self.game.last_actions
        };
        match actions.get(player.index()).and_then(Option::as_ref) {
            Some(GuandanPublicAction::Play(cards)) => {
                let winning = self
                    .game
                    .table
                    .as_ref()
                    .is_some_and(|(leader, _)| !previous && *leader == player);
                let effect = self
                    .play_effect
                    .active
                    .as_ref()
                    .filter(|effect| {
                        winning
                            && effect.match_id == self.game.match_id
                            && effect.player == player
                            && effect.card_count == cards.len()
                            && effect.kind.is_sequence()
                    })
                    .map(|effect| effect.kind);
                let column = commands.child(
                    area,
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: percent(50),
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            align_items: match side {
                                SeatSide::Left => AlignItems::FlexStart,
                                SeatSide::Top => AlignItems::Center,
                                SeatSide::Right => AlignItems::FlexEnd,
                            },
                            row_gap: px(3),
                            ..default()
                        },
                        UiTransform::from_translation(Val2::new(px(0), percent(-50))),
                        Pickable::IGNORE,
                    ),
                );
                render_card_row(
                    commands,
                    column,
                    cards,
                    self.game.level,
                    side,
                    effect,
                    self.assets,
                );
            }
            Some(GuandanPublicAction::Pass) => {
                add_text(commands, area, "不出", 28.0, MUTED, self.assets);
            }
            None => {}
        }
    }
}
