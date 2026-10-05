use crate::app::presentation::{SCORE_ROLL_DURATION, ease_out_cubic};
use leocard_client::TexasSpectatorPreferences;
use leocard_protocol::{MatchId, PlayerId, TexasHoldemSnapshot};
use std::collections::HashMap;

pub(crate) struct TexasSpectatorUiState {
    pub drawer_open: bool,
    pub preferences: TexasSpectatorPreferences,
    hand: Option<(MatchId, u32)>,
    rates: HashMap<PlayerId, WinRateRoll>,
}

impl Default for TexasSpectatorUiState {
    fn default() -> Self {
        Self {
            drawer_open: false,
            preferences: TexasSpectatorPreferences::load().unwrap_or_else(|error| {
                bevy::log::warn!("{error}");
                TexasSpectatorPreferences::default()
            }),
            hand: None,
            rates: HashMap::new(),
        }
    }
}

impl TexasSpectatorUiState {
    pub fn clear(&mut self) {
        self.drawer_open = false;
        self.hand = None;
        self.rates.clear();
    }

    pub fn sync(&mut self, game: &TexasHoldemSnapshot, delta: f32, reveal_ready: bool) {
        let hand = (game.match_id, game.hand_number);
        if self.hand != Some(hand) {
            self.clear();
            self.hand = Some(hand);
        }
        let Some(rates) = &game.spectator_equities else {
            self.rates.clear();
            return;
        };
        for player in &game.players {
            let target = if player.folded {
                Some(0)
            } else if reveal_ready {
                rates
                    .iter()
                    .find(|rate| rate.player == player.id)
                    .map(|rate| rate.basis_points)
            } else {
                None
            };
            if let Some(target) = target {
                self.rates
                    .entry(player.id)
                    .or_insert_with(|| WinRateRoll::new(target))
                    .advance(target, delta);
            } else if let Some(roll) = self.rates.get_mut(&player.id) {
                roll.advance(roll.target, delta);
            }
        }
    }

    pub fn displayed(&self, player: PlayerId) -> Option<f32> {
        self.rates.get(&player).map(|roll| roll.displayed() / 100.0)
    }
}

/// 胜率异步送达时只更新数字，保留正在播放的翻牌与筹码动画。
pub(crate) fn only_texas_equities_changed(
    before: &TexasHoldemSnapshot,
    after: &TexasHoldemSnapshot,
) -> bool {
    let mut before = before.clone();
    before
        .spectator_equities
        .clone_from(&after.spectator_equities);
    before == *after
}

struct WinRateRoll {
    from: f32,
    target: u16,
    elapsed: f32,
}

impl WinRateRoll {
    fn new(target: u16) -> Self {
        Self {
            from: f32::from(target),
            target,
            elapsed: SCORE_ROLL_DURATION,
        }
    }

    fn displayed(&self) -> f32 {
        let progress = ease_out_cubic((self.elapsed / SCORE_ROLL_DURATION).clamp(0.0, 1.0));
        self.from + (f32::from(self.target) - self.from) * progress
    }

    fn advance(&mut self, target: u16, delta: f32) {
        if self.target != target {
            self.from = self.displayed();
            self.target = target;
            self.elapsed = 0.0;
        }
        self.elapsed = (self.elapsed + delta).min(SCORE_ROLL_DURATION);
    }
}
