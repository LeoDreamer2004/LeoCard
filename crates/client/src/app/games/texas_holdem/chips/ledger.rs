//! Snapshot reconciliation, action feedback and pot transfers.

use super::super::{
    queue_texas_turn_sound, texas_action_sound_plan, texas_hand_finish_sound_plan,
    texas_side_pot_sound_plan, texas_street_sound_plan,
};

use super::denominations::initial_chip_denominations;
use super::{
    ActionFeedbackKind, ActionLabel, CHIP_MOVE_DURATION, ChipZone, PotDivisionTransition,
    TexasChipTableState, VisualPot,
};
use crate::app::presentation::{ACCENT, DANGER, READY, TEXT};
use bevy::prelude::*;
use leocard_protocol::{PlayerId, TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemSnapshot};
use leocard_texas_holdem::TexasHoldemAction;

pub(super) fn visual_pots(game: &TexasHoldemSnapshot) -> Vec<VisualPot> {
    let maximum = game
        .players
        .iter()
        .map(|player| player.committed_total)
        .max()
        .unwrap_or(0);
    if maximum == 0 {
        return Vec::new();
    }

    // A temporary unmatched raise is not a side pot. Only a funded all-in cap
    // below the current maximum creates a stable boundary during betting.
    let mut levels = game
        .players
        .iter()
        .filter(|player| {
            player.all_in && player.committed_total > 0 && player.committed_total < maximum
        })
        .map(|player| player.committed_total)
        .collect::<Vec<_>>();
    levels.push(maximum);
    levels.sort_unstable();
    levels.dedup();

    let mut previous = 0;
    levels
        .into_iter()
        .filter_map(|level| {
            let lower = previous;
            let amount = game
                .players
                .iter()
                .map(|player| {
                    player
                        .committed_total
                        .saturating_sub(lower)
                        .min(level.saturating_sub(lower))
                })
                .sum::<u32>();
            previous = level;
            (amount > 0).then(|| VisualPot {
                amount,
                eligible: game
                    .players
                    .iter()
                    .filter(|player| !player.folded && player.committed_total > lower)
                    .map(|player| player.id)
                    .collect(),
            })
        })
        .collect()
}

impl TexasChipTableState {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn observe(&mut self, game: &TexasHoldemSnapshot, events: Vec<TexasHoldemEvent>) {
        let new_hand = self.match_id != Some(game.match_id)
            || self.hand_number != game.hand_number
            || self.you != Some(game.you);
        if new_hand {
            self.initialize(game);
            return;
        }
        self.seats = game
            .players
            .iter()
            .map(|player| (player.id, player.seat))
            .collect();
        let previous_player = self.current_player;
        let mut swept = false;
        let mut hand_finished = false;
        for event in events {
            self.event_serial = self.event_serial.saturating_add(1);
            let sound_seed = self.event_serial.wrapping_mul(17);
            match event {
                TexasHoldemEvent::ActionApplied {
                    player,
                    action,
                    amount,
                    ..
                } => {
                    self.audio_cues
                        .extend(texas_action_sound_plan(action, amount, sound_seed));
                    self.apply_action(game, player, action, amount);
                }
                TexasHoldemEvent::StreetAdvanced { dealt, .. } => {
                    self.audio_cues
                        .extend(texas_street_sound_plan(dealt.len(), sound_seed));
                    self.sweep_bets_to_pot();
                    swept = true;
                }
                TexasHoldemEvent::HandAnalyzed { .. } => {}
                TexasHoldemEvent::HandFinished { showdown } => {
                    self.audio_cues
                        .extend(texas_hand_finish_sound_plan(showdown, sound_seed));
                    self.sweep_bets_to_pot();
                    swept = true;
                    hand_finished = true;
                }
            }
        }
        let next_pots = visual_pots(game);
        let old_count = self.pots.len().max(1);
        let new_count = next_pots.len().max(1);
        let divisions_changed = old_count != new_count;
        if divisions_changed && (old_count > 1 || new_count > 1) {
            self.division = Some(PotDivisionTransition {
                old_count,
                new_count,
                elapsed: 0.0,
            });
            self.audio_cues.extend(texas_side_pot_sound_plan(
                self.event_serial.wrapping_mul(31),
            ));
        }
        self.pots = next_pots;
        if swept || divisions_changed {
            self.repartition_pot_chips(if divisions_changed { 0.24 } else { 0.0 });
        }
        if hand_finished {
            self.award_finished_pots(game);
        }
        for player in &game.players {
            if player.folded {
                self.actions.entry(player.id).or_insert(ActionLabel {
                    text: "弃牌",
                    font_size: 18.0,
                    color: Color::srgb(0.58, 0.62, 0.60),
                    folded: true,
                    kind: ActionFeedbackKind::Fold,
                    elapsed: 0.0,
                });
            }
        }
        if previous_player != game.current_player && game.current_player == Some(game.you) {
            queue_texas_turn_sound(&mut self.audio_cues, self.event_serial.wrapping_mul(47));
        }
        self.current_player = game.current_player;
    }

    pub(super) fn initialize(&mut self, game: &TexasHoldemSnapshot) {
        self.reset();
        self.match_id = Some(game.match_id);
        self.hand_number = game.hand_number;
        self.you = Some(game.you);
        self.current_player = game.current_player;
        self.seats = game
            .players
            .iter()
            .map(|player| (player.id, player.seat))
            .collect();
        self.pots = visual_pots(game);
        let complete = matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. });
        for player in &game.players {
            for denomination in initial_chip_denominations(player.stack) {
                self.spawn_chip(
                    denomination,
                    ChipZone::Stack(player.id),
                    self.stack_anchor(player.id),
                );
            }
            if !complete {
                let in_pot = player
                    .committed_total
                    .saturating_sub(player.committed_street);
                self.spawn_value_in_zone(in_pot, ChipZone::Pot(0));
                self.spawn_value_in_zone(player.committed_street, ChipZone::Bet(player.id));
            }
            if player.folded {
                self.actions.insert(
                    player.id,
                    ActionLabel {
                        text: "弃牌",
                        font_size: 18.0,
                        color: Color::srgb(0.58, 0.62, 0.60),
                        folded: true,
                        kind: ActionFeedbackKind::Fold,
                        elapsed: 1.0,
                    },
                );
            }
        }
        if !complete {
            self.repartition_pot_chips(0.0);
        }
    }

    fn apply_action(
        &mut self,
        game: &TexasHoldemSnapshot,
        player: PlayerId,
        action: TexasHoldemAction,
        amount: u32,
    ) {
        let label = match action {
            TexasHoldemAction::PostBlind => ActionLabel {
                text: if player == game.small_blind {
                    "下小盲注"
                } else {
                    "下大盲注"
                },
                font_size: 18.0,
                color: ACCENT,
                folded: false,
                kind: ActionFeedbackKind::Blind,
                elapsed: 0.0,
            },
            TexasHoldemAction::Fold => ActionLabel {
                text: "弃牌",
                font_size: 18.0,
                color: Color::srgb(0.58, 0.62, 0.60),
                folded: true,
                kind: ActionFeedbackKind::Fold,
                elapsed: 0.0,
            },
            TexasHoldemAction::Check => ActionLabel {
                text: "过牌",
                font_size: 18.0,
                color: TEXT,
                folded: false,
                kind: ActionFeedbackKind::Check,
                elapsed: 0.0,
            },
            TexasHoldemAction::Call => ActionLabel {
                text: "跟注",
                font_size: 18.0,
                color: READY,
                folded: false,
                kind: ActionFeedbackKind::Call,
                elapsed: 0.0,
            },
            TexasHoldemAction::RaiseTo(_) => ActionLabel {
                text: "加注",
                font_size: 18.0,
                color: ACCENT,
                folded: false,
                kind: ActionFeedbackKind::Raise,
                elapsed: 0.0,
            },
            TexasHoldemAction::AllIn => ActionLabel {
                text: "全下",
                font_size: 24.0,
                color: DANGER,
                folded: false,
                kind: ActionFeedbackKind::AllIn,
                elapsed: 0.0,
            },
        };
        self.actions.insert(player, label);
        if amount > 0 {
            let ids = self.take_exact_from_stack(player, amount);
            for (index, id) in ids.into_iter().enumerate() {
                let target = self.scatter_target(ChipZone::Bet(player), id);
                self.move_chip(
                    id,
                    ChipZone::Bet(player),
                    target,
                    index as f32 * 0.045,
                    CHIP_MOVE_DURATION,
                );
            }
        }
    }

    fn sweep_bets_to_pot(&mut self) {
        self.actions.retain(|_, label| label.folded);
        let ids = self
            .chips
            .iter()
            .filter_map(|chip| matches!(chip.zone, ChipZone::Bet(_)).then_some(chip.id))
            .collect::<Vec<_>>();
        for (index, id) in ids.into_iter().enumerate() {
            let target = self.scatter_target(ChipZone::Pot(0), id);
            self.move_chip(id, ChipZone::Pot(0), target, index as f32 * 0.025, 0.48);
        }
    }

    fn repartition_pot_chips(&mut self, base_delay: f32) {
        let pot_count = self.pots.len().max(1);
        let mut chips = self
            .chips
            .iter()
            .filter(|chip| matches!(chip.zone, ChipZone::Pot(_)))
            .map(|chip| (chip.id, chip.denomination))
            .collect::<Vec<_>>();
        if chips.is_empty() {
            return;
        }
        chips.sort_unstable_by_key(|(_, denomination)| std::cmp::Reverse(*denomination));
        let central_total = chips
            .iter()
            .map(|(_, denomination)| f32::from(*denomination))
            .sum::<f32>();
        let declared_total = self.pots.iter().map(|pot| pot.amount).sum::<u32>().max(1) as f32;
        let mut remaining = if self.pots.is_empty() {
            vec![central_total]
        } else {
            self.pots
                .iter()
                .map(|pot| pot.amount as f32 / declared_total * central_total)
                .collect::<Vec<_>>()
        };

        for (order, (id, denomination)) in chips.into_iter().enumerate() {
            let target_pot = remaining
                .iter()
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(right.1))
                .map_or(0, |(index, _)| index.min(pot_count - 1));
            remaining[target_pot] = (remaining[target_pot] - f32::from(denomination)).max(0.0);
            let zone = ChipZone::Pot(target_pot);
            let target = self.scatter_target(zone, id);
            self.move_chip(id, zone, target, base_delay + order as f32 * 0.018, 0.46);
        }
    }

    fn award_finished_pots(&mut self, game: &TexasHoldemSnapshot) {
        let TexasHoldemPhaseView::HandComplete { awards, .. } = &game.phase else {
            return;
        };
        // Exact settlement still comes from the authority. Consolidating the
        // visual zones only affects chip selection; positions remain where the
        // side-pot animation placed them until they fly to each winner.
        for chip in &mut self.chips {
            if matches!(chip.zone, ChipZone::Pot(_)) {
                chip.zone = ChipZone::Pot(0);
            }
        }
        let mut delay = 0.86;
        for award in awards {
            if award.winners.is_empty() {
                continue;
            }
            let share = award.amount / award.winners.len() as u32;
            let remainder = award.amount % award.winners.len() as u32;
            for (winner_index, winner) in award.winners.iter().copied().enumerate() {
                let amount = share + u32::from((winner_index as u32) < remainder);
                let ids = self.take_exact_from_zone(ChipZone::Pot(0), amount);
                for (chip_index, id) in ids.into_iter().enumerate() {
                    let target = self.stack_anchor(winner);
                    self.move_chip(
                        id,
                        ChipZone::Stack(winner),
                        target,
                        delay + chip_index as f32 * 0.025,
                        0.56,
                    );
                }
                delay += 0.10;
            }
        }
    }
}
