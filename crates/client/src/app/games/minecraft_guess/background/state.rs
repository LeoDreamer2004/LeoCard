use bevy::prelude::*;
use leocard_minecraft_guess::{GuessResult, MatchQuality, RoundView};
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub(super) struct BackgroundPlayback {
    pub round: Option<(MatchId, u8)>,
    pub panorama: Option<Handle<Image>>,
    pub pending_panorama: Option<PendingPanorama>,
    pub seconds: f32,
    pub transition: f32,
    revealing: bool,
}

impl BackgroundPlayback {
    pub fn begin_round(&mut self, round: Option<(MatchId, u8)>) {
        if round.is_none() {
            *self = Self::default();
            return;
        }
        self.round = round;
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
                // Replace the texture only when dirt completely covers the old panorama.
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

pub(super) fn matched_version(round: &RoundView) -> Option<&str> {
    round.guesses.iter().find_map(|guess| {
        let GuessResult::Known { entry, matches, .. } = &guess.result else {
            return None;
        };
        entry
            .attributes
            .iter()
            .zip(matches)
            .find_map(|(attribute, feedback)| {
                (attribute.label == "首次加入版本" && feedback.quality == MatchQuality::Exact)
                    .then(|| attribute.values.first().map(String::as_str))
                    .flatten()
            })
    })
}
