//! 德州扑克的客户端筹码账本、找零算法和牌桌筹码动画。
//!
//! 规则核心仍以整数结算；这里把整数映射为有独立身份的实体筹码。每枚筹码在
//! 玩家余额区、玩家下注区和中央底池之间转移，UI 重建时也能从资源恢复位置。

use super::*;

const DENOMINATIONS: [u16; 5] = [100, 25, 10, 5, 1];
const CHIP_SIZE: f32 = 30.0;
const CHIP_MOVE_DURATION: f32 = 0.42;
const PLAYER_CHIP_ZONE_WIDTH: f32 = 160.0;
const PLAYER_CHIP_ZONE_HEIGHT: f32 = 82.0;
const TEXAS_CHIP_ZONE_SHADER: &str = "shaders/texas_chip_zone.wgsl";

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(in crate::app) struct TexasChipZoneMaterial {
    /// Column 0: appearance; column 1: table placement; column 2: mapping mode.
    #[uniform(0)]
    params: Mat4,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl UiMaterial for TexasChipZoneMaterial {
    fn fragment_shader() -> ShaderRef {
        TEXAS_CHIP_ZONE_SHADER.into()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ChipZone {
    Stack(PlayerId),
    Bet(PlayerId),
    Pot(usize),
    Retired,
}

#[derive(Clone, Copy, Debug)]
struct ChipMotion {
    start: Vec2,
    target: Vec2,
    start_rotation: f32,
    target_rotation: f32,
    elapsed: f32,
    delay: f32,
    duration: f32,
}

#[derive(Clone, Debug)]
struct TableChip {
    id: u64,
    denomination: u16,
    zone: ChipZone,
    position: Vec2,
    rotation: f32,
    motion: Option<ChipMotion>,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::app) struct ActionLabel {
    text: &'static str,
    font_size: f32,
    color: Color,
    folded: bool,
    pub(in crate::app) kind: ActionFeedbackKind,
    pub(in crate::app) elapsed: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::app) enum ActionFeedbackKind {
    Blind,
    Fold,
    Check,
    Call,
    Raise,
    AllIn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisualPot {
    amount: u32,
    eligible: Vec<PlayerId>,
}

#[derive(Clone, Copy, Debug)]
struct PotDivisionTransition {
    old_count: usize,
    new_count: usize,
    elapsed: f32,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::app) struct ChipZoneLayout {
    pub(in crate::app) left: f32,
    pub(in crate::app) top: f32,
    pub(in crate::app) width: f32,
    pub(in crate::app) height: f32,
}

impl ChipZoneLayout {
    fn center(self) -> Vec2 {
        Vec2::new(self.left + self.width * 0.5, self.top + self.height * 0.5)
    }
}

#[derive(Resource, Default)]
pub(in crate::app) struct TexasChipTableState {
    match_id: Option<MatchId>,
    hand_number: u32,
    you: Option<PlayerId>,
    seats: HashMap<PlayerId, SeatId>,
    chips: Vec<TableChip>,
    pub(in crate::app) actions: HashMap<PlayerId, ActionLabel>,
    pub(in crate::app) audio_cues: Vec<TexasAudioCue>,
    current_player: Option<PlayerId>,
    pots: Vec<VisualPot>,
    division: Option<PotDivisionTransition>,
    next_id: u64,
    event_serial: u64,
}

#[derive(Component)]
pub(in crate::app) struct TexasChipSprite(u64);

pub(in crate::app) fn initial_chip_denominations(total: u32) -> Vec<u16> {
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

fn visual_pots(game: &TexasHoldemSnapshot) -> Vec<VisualPot> {
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

fn change_for(denomination: u16) -> Vec<u16> {
    match denomination {
        100 => vec![25; 4],
        25 => vec![10, 10, 5],
        10 => vec![5, 5],
        5 => vec![1; 5],
        _ => Vec::new(),
    }
}

impl TexasChipTableState {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn observe(&mut self, game: &TexasHoldemSnapshot, events: Vec<TexasHoldemEvent>) {
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

    fn initialize(&mut self, game: &TexasHoldemSnapshot) {
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
        chips.sort_unstable_by(|left, right| right.1.cmp(&left.1));
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
        candidates.sort_unstable_by(|left, right| right.1.cmp(&left.1));
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

    fn relative_seat(&self, player: PlayerId) -> u8 {
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

    pub(in crate::app) fn stack_counts(&self, player: PlayerId) -> Vec<(u16, usize)> {
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

pub(in crate::app) fn texas_player_chip_zone(relative: u8) -> ChipZoneLayout {
    let (center_x, top) = match relative {
        0 => {
            // 自己的下注区位于底牌正上方，操作按钮紧接在其下方。
            (640.0, 404.0)
        }
        1 => (342.0, 382.0),
        2 => (342.0, 159.0),
        3 => (640.0, 105.0),
        4 => (938.0, 159.0),
        5 => (938.0, 382.0),
        _ => return texas_pot_chip_zone(),
    };
    ChipZoneLayout {
        left: center_x - PLAYER_CHIP_ZONE_WIDTH * 0.5,
        top,
        width: PLAYER_CHIP_ZONE_WIDTH,
        height: PLAYER_CHIP_ZONE_HEIGHT,
    }
}

pub(in crate::app) fn texas_pot_chip_zone() -> ChipZoneLayout {
    ChipZoneLayout {
        left: 460.0,
        // Keep pot chips below the board cards, but reclaim the space that used
        // to be reserved for the "底池" title.
        top: 296.0,
        width: 360.0,
        height: 108.0,
    }
}

fn texas_pot_partition_zone(index: usize, count: usize) -> ChipZoneLayout {
    let whole = texas_pot_chip_zone();
    let count = count.max(1);
    let gap = if count > 1 { 8.0 } else { 0.0 };
    let width = (whole.width - gap * count.saturating_sub(1) as f32) / count as f32;
    ChipZoneLayout {
        left: whole.left + index.min(count - 1) as f32 * (width + gap),
        top: whole.top,
        width,
        height: whole.height,
    }
}

/// The visual centre zone contains both the board cards and the physical pot.
/// Chip scattering still uses `texas_pot_chip_zone`, so chips cannot cover cards.
fn texas_center_zone_panel() -> ChipZoneLayout {
    ChipZoneLayout {
        // The visible card row occupies roughly x=452..828. Keep the centre
        // frame close to that content and well clear of both side chip zones.
        left: 439.0,
        top: 194.0,
        width: 402.0,
        height: 210.0,
    }
}

fn random_unit(id: u64, salt: u64) -> f32 {
    let mut value = id
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    (value as u32) as f32 / u32::MAX as f32
}

fn random_signed(id: u64, salt: u64) -> f32 {
    random_unit(id, salt) * 2.0 - 1.0
}

pub(in crate::app) fn sync_texas_chip_state(
    mut client: Option<ResMut<ClientResource>>,
    mut state: ResMut<TexasChipTableState>,
) {
    let Some(client) = client.as_deref_mut() else {
        state.reset();
        return;
    };
    let events = client.0.take_texas_holdem_events();
    let snapshot = client.0.model().texas_holdem_game().cloned();
    let Some(snapshot) = snapshot else {
        if state.match_id.is_some() {
            state.reset();
        }
        return;
    };
    state.observe(&snapshot, events);
}

pub(in crate::app) fn animate_texas_chip_sprites(
    time: Res<Time>,
    mut state: ResMut<TexasChipTableState>,
    mut sprites: Query<(
        Entity,
        &TexasChipSprite,
        &mut Node,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();
    if let Some(division) = state.division.as_mut() {
        division.elapsed += delta;
        if division.elapsed >= 0.72 {
            state.division = None;
        }
    }
    for label in state.actions.values_mut() {
        label.elapsed += delta;
    }
    for chip in &mut state.chips {
        let Some(mut motion) = chip.motion else {
            continue;
        };
        motion.elapsed += delta;
        let progress = ((motion.elapsed - motion.delay) / motion.duration).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        chip.position = motion.start.lerp(motion.target, eased);
        chip.rotation =
            motion.start_rotation + (motion.target_rotation - motion.start_rotation) * eased;
        if progress >= 1.0 {
            chip.motion = None;
        } else {
            chip.motion = Some(motion);
        }
    }
    state
        .chips
        .retain(|chip| chip.zone != ChipZone::Retired || chip.motion.is_some());

    for (entity, marker, mut node, mut transform, mut visibility) in &mut sprites {
        let Some(chip) = state.chips.iter().find(|chip| chip.id == marker.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let visible = matches!(
            chip.zone,
            ChipZone::Bet(_) | ChipZone::Pot(_) | ChipZone::Retired
        ) || chip.motion.is_some();
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.left = px(chip.position.x);
        node.top = px(chip.position.y);
        transform.rotation = Rot2::radians(chip.rotation);
    }
}

pub(in crate::app) fn add_texas_chip_areas(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    state: &TexasChipTableState,
    assets: &UiAssets,
    materials: &mut Assets<TexasChipZoneMaterial>,
    felt: &Handle<Image>,
    tiled: bool,
    brightness: f32,
) {
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(SeatId(0), |player| player.seat);
    for relative in 0..TABLE_SEAT_COUNT {
        let physical_seat = SeatId((own_seat.0 + relative) % TABLE_SEAT_COUNT);
        let player = game
            .players
            .iter()
            .find(|player| player.seat == physical_seat);
        let layout = texas_player_chip_zone(relative);
        let zone = add_chip_zone_panel(commands, table, layout, materials, felt, tiled, brightness);
        if let Some(player) = player {
            if let Some(label) = state.actions.get(&player.id) {
                add_chip_zone_title(commands, zone, label, assets);
                if label.kind == ActionFeedbackKind::Fold {
                    let own_cards =
                        (player.id == game.you).then_some(game.your_hole_cards.as_slice());
                    add_fold_card_feedback(commands, table, zone, label.elapsed, own_cards, assets);
                }
            }
            if matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. })
                && let Some(hand) = game
                    .revealed_hands
                    .iter()
                    .find(|hand| hand.player == player.id)
            {
                add_revealed_hole_cards(commands, zone, hand.cards, assets);
            }
        }
    }

    // One continuous felt panel frames the complete centre: draw pile, five
    // community cards, and the chips below them. There is no separate pot title.
    add_chip_zone_panel(
        commands,
        table,
        texas_center_zone_panel(),
        materials,
        felt,
        tiled,
        brightness,
    );
    add_pot_divisions(commands, table, state);
    add_pot_hover_regions(commands, table, state);

    let layer = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    commands
        .entity(layer)
        .insert((ZIndex(35), FocusPolicy::Pass));
    for chip in state.chips.iter().filter(|chip| {
        matches!(
            chip.zone,
            ChipZone::Bet(_) | ChipZone::Pot(_) | ChipZone::Retired
        ) || chip.motion.is_some()
    }) {
        add_chip_sprite(commands, layer, chip, assets);
    }
}

fn add_pot_divisions(commands: &mut Commands, table: Entity, state: &TexasChipTableState) {
    if let Some(transition) = state.division {
        spawn_pot_divider_set(
            commands,
            table,
            transition.old_count,
            true,
            transition.elapsed,
        );
        spawn_pot_divider_set(
            commands,
            table,
            transition.new_count,
            false,
            transition.elapsed,
        );
    } else {
        spawn_pot_divider_set(commands, table, state.pots.len().max(1), false, 1.0);
    }
}

fn spawn_pot_divider_set(
    commands: &mut Commands,
    table: Entity,
    count: usize,
    old_layout: bool,
    elapsed: f32,
) {
    if count <= 1 {
        return;
    }
    let whole = texas_pot_chip_zone();
    let (alpha, scale) = pot_divider_visual(old_layout, elapsed);
    for index in 1..count {
        let divider = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(whole.left + whole.width * index as f32 / count as f32 - 0.5),
                    top: px(whole.top + 8.0),
                    width: px(1),
                    height: px(whole.height - 16.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.68, 0.72, 0.70).with_alpha(alpha * 0.42)),
                UiTransform {
                    scale: Vec2::new(1.0, scale),
                    ..UiTransform::IDENTITY
                },
                TexasPotDivider {
                    old_layout,
                    elapsed,
                },
                ZIndex(28),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(divider);
    }
}

fn add_pot_hover_regions(commands: &mut Commands, table: Entity, state: &TexasChipTableState) {
    if state.pots.len() <= 1 {
        return;
    }
    for (index, pot) in state.pots.iter().enumerate() {
        let layout = texas_pot_partition_zone(index, state.pots.len());
        let region = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(layout.left),
                    top: px(layout.top),
                    width: px(layout.width),
                    height: px(layout.height),
                    ..default()
                },
                RelativeCursorPosition::default(),
                TexasPotHover {
                    eligible: pot.eligible.clone(),
                },
                ZIndex(34),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(region);
    }
}

fn pot_divider_visual(old_layout: bool, elapsed: f32) -> (f32, f32) {
    if old_layout {
        (1.0 - (elapsed / 0.16).clamp(0.0, 1.0), 1.0)
    } else {
        let progress = ((elapsed - 0.18) / 0.30).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        (eased, eased)
    }
}

pub(in crate::app) fn animate_texas_pot_dividers(
    time: Res<Time>,
    mut dividers: Query<(&mut TexasPotDivider, &mut UiTransform, &mut BackgroundColor)>,
) {
    for (mut divider, mut transform, mut background) in &mut dividers {
        divider.elapsed += time.delta_secs();
        let (alpha, scale) = pot_divider_visual(divider.old_layout, divider.elapsed);
        transform.scale.y = scale;
        background.0 = Color::srgb(0.68, 0.72, 0.70).with_alpha(alpha * 0.42);
    }
}

pub(in crate::app) fn highlight_texas_pot_eligible_players(
    time: Res<Time>,
    hovers: Query<(&RelativeCursorPosition, &TexasPotHover)>,
    mut panels: Query<(&TexasPlayerPanel, &mut BorderColor, &mut BoxShadow)>,
) {
    let hovered = hovers
        .iter()
        .find(|(cursor, _)| cursor.cursor_over())
        .map(|(_, pot)| pot);
    let breath = pot_eligibility_breath(time.elapsed_secs());
    for (panel, mut border, mut shadow) in &mut panels {
        if hovered.is_some_and(|pot| pot.eligible.contains(&panel.player)) {
            let green = Color::srgb(
                0.28 + breath * 0.12,
                0.82 + breath * 0.18,
                0.48 + breath * 0.16,
            );
            border.set_all(green.with_alpha(0.76 + breath * 0.24));
            *shadow = BoxShadow::new(
                green.with_alpha(0.20 + breath * 0.34),
                px(0),
                px(0),
                px(1.0 + breath * 2.0),
                px(6.0 + breath * 10.0),
            );
        } else {
            border.set_all(panel.base_border);
            *shadow = BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0));
        }
    }
}

fn pot_eligibility_breath(elapsed: f32) -> f32 {
    let phase = elapsed.rem_euclid(1.7) / 1.7;
    let wave = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    wave * wave * (3.0 - 2.0 * wave)
}

fn add_chip_zone_panel(
    commands: &mut Commands,
    table: Entity,
    layout: ChipZoneLayout,
    materials: &mut Assets<TexasChipZoneMaterial>,
    felt: &Handle<Image>,
    tiled: bool,
    brightness: f32,
) -> Entity {
    let material =
        create_texas_chip_zone_material(materials, felt.clone(), layout, tiled, brightness);
    let zone = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(layout.left),
            top: px(layout.top),
            width: px(layout.width),
            height: px(layout.height),
            padding: UiRect::new(px(9), px(42), px(6), px(6)),
            border_radius: BorderRadius::all(px(11)),
            overflow: Overflow::clip(),
            ..default()
        },
        None,
    );
    commands.entity(zone).insert((
        MaterialNode(material),
        // 区域框属于桌面底层；实际筹码在独立的高层中移动。
        ZIndex(-10),
        FocusPolicy::Pass,
    ));
    zone
}

fn create_texas_chip_zone_material(
    materials: &mut Assets<TexasChipZoneMaterial>,
    felt: Handle<Image>,
    layout: ChipZoneLayout,
    tiled: bool,
    brightness: f32,
) -> Handle<TexasChipZoneMaterial> {
    let appearance = Vec4::new(brightness * 0.58, brightness * 1.12, 3.0, 11.0);
    let placement = Vec4::new(layout.left, layout.top, DESIGN_WIDTH, DESIGN_HEIGHT - 64.0);
    materials.add(TexasChipZoneMaterial {
        params: Mat4::from_cols(
            appearance,
            placement,
            Vec4::new(if tiled { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0),
            Vec4::ZERO,
        ),
        texture: felt,
    })
}

fn add_chip_zone_title(
    commands: &mut Commands,
    zone: Entity,
    label: &ActionLabel,
    assets: &UiAssets,
) {
    let title = spawn_node(
        commands,
        zone,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(5),
            height: px((label.font_size + 5.0).max(22.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(title).insert((
        ZIndex(22),
        FocusPolicy::Pass,
        TexasActionFeedback {
            kind: label.kind,
            elapsed: label.elapsed,
        },
        action_feedback_transform(label.kind, label.elapsed),
    ));
    let text = add_text(
        commands,
        title,
        label.text,
        label.font_size,
        action_feedback_text_color(label.kind, label.color, label.elapsed),
        assets,
    );
    commands.entity(text).insert(TexasActionFeedbackText {
        color: label.color,
        kind: label.kind,
        elapsed: label.elapsed,
    });
}

fn add_fold_card_feedback(
    commands: &mut Commands,
    table: Entity,
    zone: Entity,
    elapsed: f32,
    own_cards: Option<&[TexasHoldemCard]>,
    assets: &UiAssets,
) {
    let own = own_cards.is_some();
    let tooltip = own_cards.and_then(|cards| {
        (cards.len() == 2).then(|| add_own_fold_tooltip(commands, table, cards, assets))
    });
    let layout = texas_player_chip_zone(0);
    for index in 0..2 {
        let face = own_cards
            .and_then(|cards| cards.get(index))
            .copied()
            .map(|card| texas_card_face(card, assets));
        let visual = fold_card_visual(index, elapsed, own);
        let initial_image = if visual.face_visible {
            face.clone().unwrap_or_else(|| assets.card_back.clone())
        } else {
            assets.card_back.clone()
        };
        let card = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: if own {
                        px(layout.left + layout.width * 0.5)
                    } else {
                        percent(50)
                    },
                    top: if own { px(layout.top + 28.0) } else { px(28) },
                    width: px(34),
                    height: px(48),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                ImageNode::new(initial_image),
                visual.transform,
                TexasFoldCard {
                    index,
                    elapsed,
                    own,
                    face,
                    back: assets.card_back.clone(),
                },
                ZIndex(if own { 44 } else { 24 } + index as i32),
                FocusPolicy::Pass,
            ))
            .id();
        if let Some(tooltip) = tooltip {
            commands.entity(card).insert((
                RelativeCursorPosition::default(),
                TexasOwnFoldCardHover { tooltip },
            ));
        }
        commands
            .entity(if own { table } else { zone })
            .add_child(card);
    }
}

fn add_own_fold_tooltip(
    commands: &mut Commands,
    table: Entity,
    cards: &[TexasHoldemCard],
    assets: &UiAssets,
) -> Entity {
    let layout = texas_player_chip_zone(0);
    let tooltip = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(layout.left + layout.width * 0.5 - 53.0),
            top: px(layout.top - 79.0),
            width: px(106),
            height: px(72),
            padding: UiRect::all(px(7)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(5),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(9)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.96)),
    );
    commands.entity(tooltip).insert((
        Visibility::Hidden,
        TexasOwnFoldTooltip,
        BorderColor::all(MUTED.with_alpha(0.65)),
        ZIndex(80),
        FocusPolicy::Pass,
    ));
    for card in cards.iter().copied() {
        let face = commands
            .spawn((
                Node {
                    width: px(40),
                    height: px(54),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(texas_card_face(card, assets)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(tooltip).add_child(face);
    }
    tooltip
}

fn add_revealed_hole_cards(
    commands: &mut Commands,
    zone: Entity,
    cards: [TexasHoldemCard; 2],
    assets: &UiAssets,
) {
    let hand = spawn_node(
        commands,
        zone,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(4),
            width: px(70),
            height: px(50),
            flex_direction: FlexDirection::Row,
            column_gap: px(-7),
            ..default()
        },
        None,
    );
    commands.entity(hand).insert((
        UiTransform::from_translation(Val2::px(-35.0, 0.0)),
        ZIndex(20),
    ));
    for card in cards {
        let image = texas_card_face(card, assets);
        let entity = commands
            .spawn((
                Node {
                    width: px(37),
                    height: px(49),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                ImageNode::new(image),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(hand).add_child(entity);
    }
}

fn add_chip_sprite(commands: &mut Commands, layer: Entity, chip: &TableChip, assets: &UiAssets) {
    let Some(image) = assets.poker_chips.get(&chip.denomination) else {
        return;
    };
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(chip.position.x),
                top: px(chip.position.y),
                width: px(CHIP_SIZE),
                height: px(CHIP_SIZE),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(image.clone()),
            UiTransform::from_rotation(Rot2::radians(chip.rotation)),
            TexasChipSprite(chip.id),
            ZIndex((chip.id % 200) as i32),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(entity);
    let color = if chip.denomination == 1 {
        HEADER_BG
    } else {
        TEXT
    };
    add_text(
        commands,
        entity,
        chip.denomination.to_string(),
        8.0,
        color,
        assets,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pot_test_snapshot(committed: &[(u32, bool, bool)]) -> TexasHoldemSnapshot {
        let players = committed
            .iter()
            .enumerate()
            .map(
                |(index, &(amount, all_in, folded))| TexasHoldemPlayerState {
                    id: PlayerId(index as u8),
                    profile_id: leocard_protocol::ProfileId([index as u8; 32]),
                    name: format!("P{index}"),
                    avatar: None,
                    seat: SeatId(index as u8),
                    stack: if all_in {
                        0
                    } else {
                        20_u32.saturating_sub(amount)
                    },
                    hand_start_stack: 20,
                    committed_street: amount,
                    committed_total: amount,
                    folded,
                    all_in,
                    connected: true,
                    auto_play: false,
                    ready: false,
                    reference_points: 0,
                    completed_games: 0,
                },
            )
            .collect::<Vec<_>>();
        TexasHoldemSnapshot {
            match_id: MatchId([1; 16]),
            hand_number: 1,
            host_port: 5230,
            you: PlayerId(0),
            host: PlayerId(0),
            players,
            your_hole_cards: Vec::new(),
            revealed_hands: Vec::new(),
            community: Vec::new(),
            draw_pile_len: 52,
            dealer: PlayerId(0),
            small_blind: PlayerId(1),
            big_blind: PlayerId(2),
            current_player: Some(PlayerId(0)),
            blind_to_post: None,
            current_bet: committed.iter().map(|entry| entry.0).max().unwrap_or(0),
            minimum_raise_to: 1,
            amount_to_call: 0,
            raise_allowed: true,
            pot: committed.iter().map(|entry| entry.0).sum(),
            phase: TexasHoldemPhaseView::Betting {
                street: TexasStreet::PreFlop,
            },
        }
    }

    #[test]
    fn initial_distributions_keep_five_single_chips_and_exact_value() {
        let expected = [
            (5, (0, 0, 5)),
            (10, (0, 1, 5)),
            (20, (0, 3, 5)),
            (30, (1, 3, 5)),
            (40, (2, 3, 5)),
            (50, (3, 3, 5)),
        ];
        for (total, (tens, fives, ones)) in expected {
            let chips = initial_chip_denominations(total);
            assert_eq!(
                chips.iter().map(|chip| u32::from(*chip)).sum::<u32>(),
                total
            );
            assert_eq!(chips.iter().filter(|chip| **chip == 10).count(), tens);
            assert_eq!(chips.iter().filter(|chip| **chip == 5).count(), fives);
            assert_eq!(chips.iter().filter(|chip| **chip == 1).count(), ones);
        }
    }

    #[test]
    fn every_supported_chip_can_be_changed_exactly() {
        for denomination in [5, 10, 25, 100] {
            let change = change_for(denomination);
            assert_eq!(
                change.iter().map(|chip| u32::from(*chip)).sum::<u32>(),
                u32::from(denomination)
            );
            assert!(change.iter().all(|chip| *chip < denomination));
        }
    }

    #[test]
    fn side_pots_only_split_at_all_in_caps_and_keep_correct_eligibility() {
        let no_all_in = visual_pots(&pot_test_snapshot(&[
            (5, false, false),
            (10, false, false),
            (20, false, false),
        ]));
        assert_eq!(no_all_in.len(), 1);
        assert_eq!(no_all_in[0].amount, 35);

        let split = visual_pots(&pot_test_snapshot(&[
            (5, true, false),
            (10, true, false),
            (20, false, false),
        ]));
        assert_eq!(
            split.iter().map(|pot| pot.amount).collect::<Vec<_>>(),
            vec![15, 10, 10]
        );
        assert_eq!(
            split[0].eligible,
            vec![PlayerId(0), PlayerId(1), PlayerId(2)]
        );
        assert_eq!(split[1].eligible, vec![PlayerId(1), PlayerId(2)]);
        assert_eq!(split[2].eligible, vec![PlayerId(2)]);
    }

    #[test]
    fn pot_dividers_remove_old_layout_before_drawing_new_layout() {
        assert_eq!(pot_divider_visual(true, 0.0), (1.0, 1.0));
        assert_eq!(pot_divider_visual(true, 0.18).0, 0.0);
        assert_eq!(pot_divider_visual(false, 0.16), (0.0, 0.0));
        let appeared = pot_divider_visual(false, 0.48);
        assert!((appeared.0 - 1.0).abs() < f32::EPSILON);
        assert!((appeared.1 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn eligible_player_glow_uses_a_smooth_breathing_cycle() {
        assert!(pot_eligibility_breath(0.0).abs() < 0.000_001);
        assert!((pot_eligibility_breath(0.85) - 1.0).abs() < 0.000_001);
        assert!(pot_eligibility_breath(1.7).abs() < 0.000_001);
    }

    #[test]
    fn authoritative_action_audio_is_not_requeued_by_an_empty_ui_refresh() {
        let snapshot =
            pot_test_snapshot(&[(0, false, false), (0, false, false), (0, false, false)]);
        let mut state = TexasChipTableState::default();
        state.initialize(&snapshot);
        state.observe(
            &snapshot,
            vec![TexasHoldemEvent::ActionApplied {
                player: PlayerId(1),
                action: TexasHoldemAction::Check,
                amount: 0,
            }],
        );
        assert_eq!(state.audio_cues.len(), 1);
        assert_eq!(state.audio_cues[0].kind, TexasSoundKind::Check);

        state.audio_cues.clear();
        state.observe(&snapshot, Vec::new());
        assert!(state.audio_cues.is_empty());
    }

    #[test]
    fn action_feedback_uses_distinct_motion_for_check_raise_and_all_in() {
        let settled_check = action_feedback_visual(ActionFeedbackKind::Check, 1.0);
        assert_eq!(settled_check.translation, Vec2::ZERO);
        assert_eq!(settled_check.alpha, 1.0);
        assert!(action_feedback_visual(ActionFeedbackKind::Raise, 0.14).scale > 1.0);
        let all_in = action_feedback_visual(ActionFeedbackKind::AllIn, 0.17);
        assert!(all_in.scale > 1.0);
        assert_ne!(all_in.translation.x, 0.0);
        assert_eq!(
            action_feedback_visual(ActionFeedbackKind::Fold, 2.0).alpha,
            1.0
        );

        let own_start = fold_card_visual(0, 0.0, true);
        let own_flipped = fold_card_visual(0, 0.30, true);
        let own_settled = fold_card_visual(0, 0.72, true);
        let own_rebuilt = fold_card_visual(0, 8.0, true);
        assert!(own_start.face_visible);
        assert!(!own_flipped.face_visible);
        assert!(own_start.transform.scale.y > own_settled.transform.scale.y);
        assert_eq!(own_settled.transform.scale, own_rebuilt.transform.scale);
        assert_eq!(
            own_settled.transform.translation,
            own_rebuilt.transform.translation
        );
    }
}
