//! Chip selection, exchanges and physical zone transfers.

use super::denominations::{canonical_chip_denominations, change_for};
use super::{
    CHIP_MOVE_DURATION, CHIP_SIZE, ChipMotion, ChipZone, ChipZoneLayout, DENOMINATIONS, TableChip,
    TexasChipTableState, random_signed, random_unit, texas_player_chip_zone, texas_pot_chip_zone,
    texas_pot_partition_zone,
};
use bevy::prelude::*;
use leocard_protocol::{PlayerId, TABLE_SEAT_COUNT};

impl TexasChipTableState {
    pub(super) fn take_exact_from_stack(&mut self, player: PlayerId, amount: u32) -> Vec<u64> {
        self.take_exact_with_change(ChipZone::Stack(player), amount, Some(player))
    }

    pub(super) fn take_exact_from_zone(&mut self, zone: ChipZone, amount: u32) -> Vec<u64> {
        self.take_exact_with_change(zone, amount, None)
    }

    fn take_exact_with_change(
        &mut self,
        zone: ChipZone,
        amount: u32,
        player: Option<PlayerId>,
    ) -> Vec<u64> {
        loop {
            if let Some(ids) = self.select_exact(zone, amount, None) {
                return ids;
            }
            let Some(chip_id) = self
                .chips
                .iter()
                .filter(|chip| chip.zone == zone && chip.denomination > 1)
                .min_by_key(|chip| chip.denomination)
                .map(|chip| chip.id)
            else {
                return Vec::new();
            };
            self.exchange_chip(chip_id, player);
        }
    }

    fn exchange_chip(&mut self, chip_id: u64, player: Option<PlayerId>) {
        let Some(index) = self.chips.iter().position(|chip| chip.id == chip_id) else {
            return;
        };
        let denomination = self.chips[index].denomination;
        let source_zone = self.chips[index].zone;
        let source_position = self.chips[index].position;

        // 优先与中央底池或其他玩家下注区交换同价值的小面额筹码。
        let provider_zones = (0..self.pots.len().max(1))
            .map(ChipZone::Pot)
            .chain(self.seats.keys().copied().map(ChipZone::Bet))
            .chain(self.seats.keys().copied().map(ChipZone::Stack))
            .filter(|zone| *zone != source_zone)
            .collect::<Vec<_>>();
        for provider in provider_zones {
            if let Some(change_ids) =
                self.select_exact(provider, u32::from(denomination), Some(denomination))
            {
                let provider_center = self.zone_layout(provider).center();
                self.move_chip(chip_id, provider, provider_center, 0.0, CHIP_MOVE_DURATION);
                for (offset, id) in change_ids.into_iter().enumerate() {
                    let destination = player.map_or(source_zone, ChipZone::Stack);
                    let target = player.map_or(source_position, |owner| self.stack_anchor(owner));
                    self.move_chip(
                        id,
                        destination,
                        target,
                        0.08 + offset as f32 * 0.025,
                        CHIP_MOVE_DURATION,
                    );
                }
                return;
            }
        }

        // 没有可换筹码时由桌面钱箱拆分；旧筹码退场，小筹码从同一点散开。
        let exchange_point = player
            .map(|owner| self.zone_layout(ChipZone::Bet(owner)).center())
            .unwrap_or_else(|| self.zone_layout(ChipZone::Pot(0)).center());
        self.move_chip(
            chip_id,
            ChipZone::Retired,
            exchange_point,
            0.0,
            CHIP_MOVE_DURATION * 0.65,
        );
        for (offset, replacement) in change_for(denomination).into_iter().enumerate() {
            let id = self.spawn_chip(replacement, source_zone, exchange_point);
            let target = player.map_or(source_position, |owner| self.stack_anchor(owner));
            self.move_chip(
                id,
                source_zone,
                target,
                0.16 + offset as f32 * 0.025,
                CHIP_MOVE_DURATION,
            );
        }
    }

    fn select_exact(
        &self,
        zone: ChipZone,
        amount: u32,
        strictly_below: Option<u16>,
    ) -> Option<Vec<u64>> {
        if amount == 0 {
            return Some(Vec::new());
        }
        let mut candidates = self
            .chips
            .iter()
            .filter(|chip| {
                chip.zone == zone && strictly_below.is_none_or(|limit| chip.denomination < limit)
            })
            .map(|chip| (chip.id, chip.denomination))
            .collect::<Vec<_>>();
        candidates.sort_unstable_by_key(|(_, denomination)| std::cmp::Reverse(*denomination));
        let mut reachable = vec![None::<Vec<u64>>; amount as usize + 1];
        reachable[0] = Some(Vec::new());
        for (id, denomination) in candidates {
            let value = usize::from(denomination);
            for sum in (value..=amount as usize).rev() {
                if reachable[sum].is_none()
                    && let Some(previous) = reachable[sum - value].clone()
                {
                    let mut selected = previous;
                    selected.push(id);
                    reachable[sum] = Some(selected);
                }
            }
        }
        reachable[amount as usize].clone()
    }

    pub(super) fn spawn_value_in_zone(&mut self, value: u32, zone: ChipZone) {
        for denomination in canonical_chip_denominations(value) {
            let id = self.next_id.saturating_add(1).max(1);
            let position = self.scatter_target(zone, id);
            self.spawn_chip(denomination, zone, position);
        }
    }

    pub(super) fn spawn_chip(&mut self, denomination: u16, zone: ChipZone, position: Vec2) -> u64 {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        self.chips.push(TableChip {
            id,
            denomination,
            zone,
            position,
            rotation: random_signed(id, 91) * 0.16,
            motion: None,
        });
        id
    }

    pub(super) fn move_chip(
        &mut self,
        id: u64,
        zone: ChipZone,
        target: Vec2,
        delay: f32,
        duration: f32,
    ) {
        let Some(chip) = self.chips.iter_mut().find(|chip| chip.id == id) else {
            return;
        };
        let target_rotation = random_signed(id, self.event_serial) * 0.22;
        chip.zone = zone;
        chip.motion = Some(ChipMotion {
            start: chip.position,
            target,
            start_rotation: chip.rotation,
            target_rotation,
            elapsed: 0.0,
            delay,
            duration,
        });
    }

    pub(crate) fn relative_seat(&self, player: PlayerId) -> u8 {
        let Some(you) = self.you.and_then(|you| self.seats.get(&you)).copied() else {
            return 0;
        };
        self.seats.get(&player).map_or(0, |seat| {
            (seat.0 + TABLE_SEAT_COUNT - you.0) % TABLE_SEAT_COUNT
        })
    }

    pub(super) fn stack_anchor(&self, player: PlayerId) -> Vec2 {
        match self.relative_seat(player) {
            0 => Vec2::new(386.0, 570.0),
            1 => Vec2::new(145.0, 415.0),
            2 => Vec2::new(145.0, 165.0),
            3 => Vec2::new(640.0, 65.0),
            4 => Vec2::new(1135.0, 165.0),
            5 => Vec2::new(1135.0, 415.0),
            _ => Vec2::new(640.0, 330.0),
        }
    }

    fn zone_layout(&self, zone: ChipZone) -> ChipZoneLayout {
        match zone {
            ChipZone::Bet(player) => texas_player_chip_zone(self.relative_seat(player)),
            ChipZone::Pot(index) => texas_pot_partition_zone(index, self.pots.len().max(1)),
            ChipZone::Retired => texas_pot_chip_zone(),
            ChipZone::Stack(player) => {
                let center = self.stack_anchor(player);
                ChipZoneLayout {
                    left: center.x - 20.0,
                    top: center.y - 20.0,
                    width: 40.0,
                    height: 40.0,
                }
            }
        }
    }

    pub(super) fn scatter_target(&self, zone: ChipZone, id: u64) -> Vec2 {
        let area = self.zone_layout(zone);
        let horizontal = random_unit(id, self.event_serial.wrapping_add(17));
        let vertical = random_unit(id, self.event_serial.wrapping_add(43));
        Vec2::new(
            area.left + 9.0 + horizontal * (area.width - CHIP_SIZE - 18.0).max(1.0),
            area.top + 30.0 + vertical * (area.height - CHIP_SIZE - 35.0).max(1.0),
        )
    }

    pub(crate) fn stack_counts(&self, player: PlayerId) -> Vec<(u16, usize)> {
        DENOMINATIONS
            .into_iter()
            .filter_map(|denomination| {
                let count = self
                    .chips
                    .iter()
                    .filter(|chip| {
                        chip.zone == ChipZone::Stack(player) && chip.denomination == denomination
                    })
                    .count();
                (count > 0).then_some((denomination, count))
            })
            .collect()
    }
}
