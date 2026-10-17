use super::{
    BlackjackTable,
    cards::{BlackjackCardSpec, add_card},
    chips::{BlackjackWagerSpec, add_blackjack_wager},
    playback::BlackjackCardSlot,
};
use crate::app::presentation::{
    ACCENT, DANGER, MUTED, TEXT, TableStatusEffect, TableStatusSpec, UiComposer, add_table_status,
    spawn_node,
};
use bevy::prelude::*;
use leocard_blackjack::{HandState, HandStatus, Phase};
use leocard_protocol::PlayerId;

pub(super) struct BlackjackHandSpec<'a> {
    pub hand: &'a HandState,
    pub player: PlayerId,
    pub index: usize,
    pub own: bool,
    pub width: f32,
    pub active: bool,
    pub wager_height: f32,
    pub pile: Entity,
    pub stack: Entity,
}

impl BlackjackTable<'_> {
    pub(super) fn waiting_hand(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        insufficient: bool,
    ) {
        add_table_status(
            commands,
            parent,
            TableStatusSpec {
                text: if insufficient {
                    "筹码不足"
                } else {
                    "等待下注"
                },
                font_size: 18.0,
                color: MUTED,
                effect: TableStatusEffect::Appear,
                elapsed: None,
            },
            self.assets,
        );
    }

    pub(super) fn hand(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        spec: BlackjackHandSpec<'_>,
    ) {
        let hand = spec.hand;
        let column = commands.keyed_child(
            parent,
            format!("hand:{}", spec.index),
            (Node {
                width: px(spec.width),
                min_width: px(0),
                height: percent(100),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(3),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },),
        );
        let ready = self.playback.hand_ready(spec.player, spec.index);
        let value = hand.value();
        let label = if !ready {
            "发牌中".to_owned()
        } else if hand.status == HandStatus::Blackjack {
            "黑杰克".to_owned()
        } else if hand.status == HandStatus::Bust {
            "爆牌".to_owned()
        } else if hand.cards.is_empty() {
            "已下注".to_owned()
        } else {
            format!("{}{} 点", if value.soft { "软 " } else { "" }, value.total)
        };
        let title = spawn_node(
            commands,
            column,
            Node {
                width: percent(100),
                height: px(24),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        add_table_status(
            commands,
            title,
            TableStatusSpec {
                text: &label,
                font_size: (spec.width * 0.25).clamp(12.0, if spec.own { 20.0 } else { 18.0 }),
                color: if ready && hand.status == HandStatus::Bust {
                    DANGER
                } else if spec.active {
                    ACCENT
                } else {
                    TEXT
                },
                effect: if ready && hand.status == HandStatus::Bust {
                    TableStatusEffect::Impact
                } else if ready && hand.status == HandStatus::Blackjack {
                    TableStatusEffect::Bounce
                } else {
                    TableStatusEffect::Pulse
                },
                elapsed: None,
            },
            self.assets,
        );
        let size = (spec.width - 12.0).clamp(30.0, if spec.own { 84.0 } else { 44.0 });
        let cards = spawn_node(
            commands,
            column,
            Node {
                position_type: PositionType::Relative,
                width: px(spec.width - 8.0),
                height: px(size * 1.4),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        let span = (spec.width - 8.0 - size)
            .min(if spec.own { 92.0 } else { 27.0 } * hand.cards.len().saturating_sub(1) as f32);
        let step = span / hand.cards.len().saturating_sub(1).max(1) as f32;
        for (index, &card) in hand.cards.iter().enumerate() {
            add_card(
                commands,
                cards,
                BlackjackCardSpec {
                    slot: BlackjackCardSlot::Hand(spec.player, spec.index, index),
                    face: Some(card),
                    node: Node {
                        position_type: PositionType::Absolute,
                        left: px((spec.width - 8.0 - size - span) * 0.5 + index as f32 * step),
                        width: px(size),
                        height: px(size * 1.4),
                        ..default()
                    },
                },
                spec.pile,
                self.playback,
                self.assets,
            );
        }
        if !spec.own {
            add_blackjack_wager(
                commands,
                column,
                BlackjackWagerSpec {
                    wager: hand.wager,
                    net: hand.result.map(|_| hand.net_chips),
                    settled: self.game.phase == Phase::Complete && !self.playback.busy(),
                    width: spec.width,
                    height: spec.wager_height,
                    stack: spec.stack,
                    house: spec.pile,
                },
                self.assets,
            );
        }
    }
}
