use super::super::{BlackjackTable, layout::BlackjackSeatLayout};
use super::{BlackjackWagerSpec, add_blackjack_wager};
use crate::app::presentation::add_table_play_area;
use bevy::prelude::*;
use leocard_protocol::BlackjackPlayerView;
use leocard_ui::UiComposer;

impl BlackjackTable<'_> {
    pub(in super::super) fn own_wagers(
        &self,
        commands: &mut UiComposer,
        table: Entity,
        view: &BlackjackPlayerView,
        stack: Entity,
        house: Entity,
    ) {
        let area_width = BlackjackSeatLayout::OPPONENT_AREA_WIDTH;
        let area = commands.keyed_child(
            table,
            "blackjack-own-wagers",
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    bottom: px(240),
                    width: px(area_width),
                    height: px(80),
                    padding: UiRect::all(px(6)),
                    justify_content: JustifyContent::Center,
                    column_gap: px(6),
                    ..default()
                },
                UiTransform::from_translation(Val2::px(-area_width * 0.5, 0.0)),
            ),
        );
        let background = add_table_play_area(
            commands,
            area,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                left: px(0),
                top: px(0),
                ..default()
            },
        );
        commands.entity(background).insert(ZIndex(-10));
        let width = (area_width - 12.0 - 6.0 * view.hands.len().saturating_sub(1) as f32)
            / view.hands.len().max(1) as f32;
        for (index, hand) in view.hands.iter().enumerate() {
            let group = commands.keyed_child(
                area,
                format!("wager-hand:{index}"),
                (Node {
                    width: px(width),
                    flex_shrink: 0.0,
                    ..default()
                },),
            );
            add_blackjack_wager(
                commands,
                group,
                BlackjackWagerSpec {
                    wager: hand.wager,
                    net: hand.result.map(|_| hand.net_chips),
                    settled: self.playback.settlement_started(),
                    width,
                    height: 68.0,
                    stack,
                    house,
                },
                self.assets,
            );
        }
    }
}
