use bevy::prelude::*;
use leocard_mc_guess::{McGuessMatchQuality, McGuessResult, McGuessRoundView};
use leocard_protocol::MatchId;
use sha2::{Digest, Sha256};

#[derive(Resource, Default)]
pub(super) struct BackgroundPlayback {
    pub round: Option<(MatchId, u8)>,
    pub panorama: Option<Handle<Image>>,
    pub pending_panorama: Option<PendingPanorama>,
    pub seconds: f32,
    pub transition: f32,
    pub tile_index: usize,
    revealing: bool,
}

impl BackgroundPlayback {
    pub fn begin_round(&mut self, round: Option<(MatchId, u8)>, tile_count: usize) {
        let Some((match_id, game_index)) = round else {
            *self = Self::default();
            return;
        };
        self.round = round;
        // Fixed byte order and hash keep desktop, Android and reconnects in sync.
        let digest = Sha256::new()
            .chain_update(b"leocard-minecraft-background")
            .chain_update(match_id.0)
            .chain_update([game_index])
            .finalize();
        let seed = u64::from_le_bytes(digest[..8].try_into().expect("eight hash bytes"));
        self.tile_index = (seed % tile_count as u64) as usize;
        self.pending_panorama = None;
        self.revealing = false;
    }

    pub fn needs_panorama(&self) -> bool {
        !self.revealing && self.pending_panorama.is_none()
    }

    pub fn advance(&mut self, delta_seconds: f32, transition_seconds: f32) {
        self.seconds += delta_seconds;
        let step = delta_seconds / transition_seconds;
        if self.revealing {
            self.transition = (self.transition + step).min(1.0);
        } else {
            self.transition = (self.transition - step).max(0.0);
            if self.transition == 0.0 {
                // Replace the texture only when the tiles completely cover the old panorama.
                self.panorama = self
                    .pending_panorama
                    .take_if(|panorama| panorama.ready)
                    .map(|panorama| panorama.image);
                self.revealing = self.panorama.is_some();
            }
        }
    }
}

pub(super) struct PendingPanorama {
    pub image: Handle<Image>,
    pub ready: bool,
}

pub(super) fn matched_version(round: &McGuessRoundView) -> Option<&str> {
    round.guesses.iter().find_map(|guess| {
        let McGuessResult::Known { entry, matches, .. } = &guess.result else {
            return None;
        };
        entry
            .attributes
            .iter()
            .zip(matches)
            .find_map(|(attribute, feedback)| {
                (attribute.label == "首次加入版本"
                    && feedback.quality == McGuessMatchQuality::Exact)
                    .then(|| attribute.values.first().map(String::as_str))
                    .flatten()
            })
    })
}
