use super::*;

pub fn initial_chip_denominations(total: u32) -> Vec<u16> {
    let counts = match total {
        5 => Some((0, 0, 5)),
        10 => Some((0, 1, 5)),
        20 => Some((0, 3, 5)),
        30 => Some((1, 3, 5)),
        40 => Some((2, 3, 5)),
        50 => Some((3, 3, 5)),
        _ => None,
    };
    if let Some((tens, fives, ones)) = counts {
        return std::iter::repeat_n(10, tens)
            .chain(std::iter::repeat_n(5, fives))
            .chain(std::iter::repeat_n(1, ones))
            .collect();
    }
    canonical_chip_denominations(total)
}

pub fn visual_pots(game: &TexasHoldemSnapshot) -> Vec<VisualPot> {
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

fn canonical_chip_denominations(mut total: u32) -> Vec<u16> {
    let mut result = Vec::new();
    for denomination in DENOMINATIONS {
        while total >= u32::from(denomination) {
            result.push(denomination);
            total -= u32::from(denomination);
        }
    }
    result
}

pub fn change_for(denomination: u16) -> Vec<u16> {
    match denomination {
        100 => vec![25; 4],
        25 => vec![10, 10, 5],
        10 => vec![5, 5],
        5 => vec![1; 5],
        _ => Vec::new(),
    }
}

impl TexasChipTableState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn observe(&mut self, game: &TexasHoldemSnapshot, events: Vec<TexasHoldemEvent>) {
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
                } => {
                    self.audio_cues
                        .extend(texas_action_sound_plan(action, amount, sound_seed));
                    self.apply_action(player, action, amount);
                }
                TexasHoldemEvent::StreetAdvanced { dealt, .. } => {
                    self.audio_cues
                        .extend(texas_street_sound_plan(dealt.len(), sound_seed));
                    self.sweep_bets_to_pot();
                    swept = true;
                }
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
                    font_size: 15.0,
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

    pub fn initialize(&mut self, game: &TexasHoldemSnapshot) {
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
                        font_size: 15.0,
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

    fn apply_action(&mut self, player: PlayerId, action: TexasHoldemAction, amount: u32) {
        let label = match action {
            TexasHoldemAction::PostBlind => ActionLabel {
                text: "下盲注",
                font_size: 15.0,
                color: ACCENT,
                folded: false,
                kind: ActionFeedbackKind::Blind,
                elapsed: 0.0,
            },
            TexasHoldemAction::Fold => ActionLabel {
                text: "弃牌",
                font_size: 15.0,
                color: Color::srgb(0.58, 0.62, 0.60),
                folded: true,
                kind: ActionFeedbackKind::Fold,
                elapsed: 0.0,
            },
            TexasHoldemAction::Check => ActionLabel {
                text: "过牌",
                font_size: 15.0,
                color: TEXT,
                folded: false,
                kind: ActionFeedbackKind::Check,
                elapsed: 0.0,
            },
            TexasHoldemAction::Call => ActionLabel {
                text: "跟注",
                font_size: 15.0,
                color: READY,
                folded: false,
                kind: ActionFeedbackKind::Call,
                elapsed: 0.0,
            },
            TexasHoldemAction::RaiseTo(_) => ActionLabel {
                text: "加注",
                font_size: 15.0,
                color: ACCENT,
                folded: false,
                kind: ActionFeedbackKind::Raise,
                elapsed: 0.0,
            },
            TexasHoldemAction::AllIn => ActionLabel {
                text: "全下",
                font_size: 21.0,
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

    fn take_exact_from_stack(&mut self, player: PlayerId, amount: u32) -> Vec<u64> {
        self.take_exact_with_change(ChipZone::Stack(player), amount, Some(player))
    }

    fn take_exact_from_zone(&mut self, zone: ChipZone, amount: u32) -> Vec<u64> {
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

    fn spawn_value_in_zone(&mut self, value: u32, zone: ChipZone) {
        for denomination in canonical_chip_denominations(value) {
            let id = self.next_id.saturating_add(1).max(1);
            let position = self.scatter_target(zone, id);
            self.spawn_chip(denomination, zone, position);
        }
    }

    fn spawn_chip(&mut self, denomination: u16, zone: ChipZone, position: Vec2) -> u64 {
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

    fn move_chip(&mut self, id: u64, zone: ChipZone, target: Vec2, delay: f32, duration: f32) {
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

    pub fn relative_seat(&self, player: PlayerId) -> u8 {
        let Some(you) = self.you.and_then(|you| self.seats.get(&you)).copied() else {
            return 0;
        };
        self.seats.get(&player).map_or(0, |seat| {
            (seat.0 + TABLE_SEAT_COUNT - you.0) % TABLE_SEAT_COUNT
        })
    }

    fn stack_anchor(&self, player: PlayerId) -> Vec2 {
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

    fn scatter_target(&self, zone: ChipZone, id: u64) -> Vec2 {
        let area = self.zone_layout(zone);
        let horizontal = random_unit(id, self.event_serial.wrapping_add(17));
        let vertical = random_unit(id, self.event_serial.wrapping_add(43));
        Vec2::new(
            area.left + 9.0 + horizontal * (area.width - CHIP_SIZE - 18.0).max(1.0),
            area.top + 30.0 + vertical * (area.height - CHIP_SIZE - 35.0).max(1.0),
        )
    }

    pub fn stack_counts(&self, player: PlayerId) -> Vec<(u16, usize)> {
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
