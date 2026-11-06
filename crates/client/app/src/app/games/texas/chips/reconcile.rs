//! Repair missed authoritative changes without rebuilding matching animated zones.

use super::denominations::initial_chip_denominations;
use super::{ChipZone, TexasChipTableState};
use leocard_protocol::{TexasPhaseView, TexasSnapshot};
use std::collections::{HashMap, HashSet};

impl TexasChipTableState {
    pub(super) fn reconcile_balances(&mut self, game: &TexasSnapshot) {
        let complete = matches!(game.phase, TexasPhaseView::HandComplete { .. });
        let betting = matches!(game.phase, TexasPhaseView::Betting { .. });
        let mut expected = HashMap::new();
        let mut bets = 0;
        for player in &game.players {
            expected.insert(ChipZone::Stack(player.player.id), player.stack);
            let bet = if betting { player.committed_street } else { 0 };
            expected.insert(ChipZone::Bet(player.player.id), bet);
            bets += bet;
        }
        expected.insert(
            ChipZone::Pot(0),
            if complete {
                0
            } else {
                game.pot.saturating_sub(bets)
            },
        );

        // Pot divisions are visual, approximate denomination partitions. Compare
        // their combined value, not each side pot's nominal award amount.
        let normalize = |zone| match zone {
            ChipZone::Pot(_) => ChipZone::Pot(0),
            other => other,
        };
        let mut actual = HashMap::<_, u32>::new();
        for chip in &self.chips {
            if chip.zone != ChipZone::Retired {
                *actual.entry(normalize(chip.zone)).or_default() += u32::from(chip.denomination);
            }
        }
        let changed = expected
            .keys()
            .chain(actual.keys())
            .copied()
            .filter(|zone| {
                expected.get(zone).copied().unwrap_or(0) != actual.get(zone).copied().unwrap_or(0)
            })
            .collect::<HashSet<_>>();
        if changed.is_empty() {
            return;
        }
        self.chips
            .retain(|chip| !changed.contains(&normalize(chip.zone)));
        for zone in &changed {
            let amount = expected.get(zone).copied().unwrap_or(0);
            if let ChipZone::Stack(player) = *zone {
                for denomination in initial_chip_denominations(amount) {
                    self.spawn_chip(denomination, *zone, self.stack_anchor(player));
                }
            } else {
                self.spawn_value_in_zone(amount, *zone);
            }
        }
        if changed.contains(&ChipZone::Pot(0)) {
            self.repartition_pot_chips(0.0);
            // A reconnect repairs missed history; it does not replay that history.
            for chip in &mut self.chips {
                if matches!(chip.zone, ChipZone::Pot(_))
                    && let Some(motion) = chip.motion.take()
                {
                    chip.position = motion.target;
                    chip.rotation = motion.target_rotation;
                }
            }
        }
    }
}
