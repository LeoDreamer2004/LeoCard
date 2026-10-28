use super::DoudizhuTable;
use super::cards::{CardRowStyle, card_row_aligned};
use crate::app::presentation::{
    ACCENT, CardSize, MUTED, SeatSide, add_text, spawn_trick_play_area,
};

use bevy::prelude::*;
use leocard_doudizhu::{Action, Phase};
use leocard_protocol::PlayerId;
use leocard_ui::{UiComposer, spawn_node};

impl DoudizhuTable<'_> {
    pub(super) fn play_area(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        player: PlayerId,
        side: SeatSide,
        intro: bool,
    ) {
        let area = spawn_trick_play_area(commands, parent, side);
        if intro {
            return;
        }
        if self.game.phase == Phase::Bidding {
            if let Some(bid) = self.game.bids.get(usize::from(player.0)).copied().flatten() {
                add_text(
                    commands,
                    area,
                    if bid == 0 {
                        "不叫".to_owned()
                    } else {
                        format!("{bid} 分")
                    },
                    28.0,
                    if bid == 0 { MUTED } else { ACCENT },
                    self.assets,
                );
            }
            return;
        }
        let action = self
            .game
            .last_actions
            .get(usize::from(player.0))
            .and_then(Option::as_ref);
        let revealed = self
            .game
            .revealed_hands
            .get(usize::from(player.0))
            .filter(|hand| player != self.game.you && !hand.is_empty());
        let cards = revealed
            .map(|hand| (hand.as_slice(), CardSize::FinishedHand))
            .or(match action {
                Some(Action::Play(cards)) => Some((cards.as_slice(), CardSize::TrickSeat)),
                _ => None,
            });
        if let Some((cards, size)) = cards {
            let alignment = match side {
                SeatSide::Left => AlignItems::FlexStart,
                SeatSide::Top => AlignItems::Center,
                SeatSide::Right => AlignItems::FlexEnd,
            };
            // Absolute contents keep a long combination from moving the player's portrait.
            let holder = spawn_node(
                commands,
                area,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: percent(50),
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    align_items: alignment,
                    ..default()
                },
                None,
            );
            commands
                .entity(holder)
                .insert(UiTransform::from_translation(Val2::new(
                    px(0),
                    percent(-50),
                )));
            let effect = self
                .play_effect
                .active
                .as_ref()
                .filter(|effect| {
                    revealed.is_none()
                        && effect.match_id == self.game.match_id
                        && effect.player == player
                        && effect.card_count == cards.len()
                        && effect.kind.is_sequence()
                })
                .map(|effect| effect.kind);
            card_row_aligned(
                commands,
                holder,
                cards,
                CardRowStyle {
                    size,
                    alignment,
                    effect,
                },
                self.assets,
            );
        } else if matches!(action, Some(Action::Pass)) {
            add_text(commands, area, "不出", 28.0, MUTED, self.assets);
        }
    }
}
