use super::*;
use leocard_protocol::ShengjiEvent;
use leocard_shengji::{Category, ShengjiBidKind, compare_for_trick};

pub fn sync_shengji_presentation(
    mut client: Option<ResMut<ClientResource>>,
    mut state: ResMut<ShengjiPresentationState>,
    mut ui: ResMut<UiState>,
) {
    let Some(client) = client.as_deref_mut() else {
        if state.active.take().is_some() {
            ui.dirty = true;
        }
        state.observed_match = None;
        state.queued.clear();
        state.audio_cues.clear();
        state.bottom_burier = None;
        state.observed_throw_failure = None;
        state.observed_dealer = None;
        state.clear_trick_history();
        return;
    };
    let game_state = client.0.model().shengji_game().map(|game| {
        (
            game.match_id,
            game.hand_number,
            game.throw_failure.clone(),
            game.rules.throw_penalty,
            game.dealer,
            game.trump,
        )
    });
    let Some((match_id, hand_number, throw_failure, throw_penalty, current_dealer, current_trump)) =
        game_state
    else {
        if state.active.take().is_some() {
            ui.dirty = true;
        }
        state.observed_match = None;
        state.queued.clear();
        state.audio_cues.clear();
        state.bottom_burier = None;
        state.observed_throw_failure = None;
        state.observed_dealer = None;
        state.clear_trick_history();
        client.0.take_shengji_events();
        return;
    };
    let match_changed = state.observed_match != Some(match_id);
    if match_changed || state.observed_hand != hand_number {
        if match_changed {
            state.observed_dealer = None;
        }
        state.observed_match = Some(match_id);
        state.observed_hand = hand_number;
        state.bottom_copy_count = 0;
        state.bottom_burier = None;
        state.observed_throw_failure = None;
        state.active = None;
        state.queued.clear();
        state.audio_cues.clear();
        state.clear_trick_history();
    }

    let observed_throw_failure =
        throw_failure
            .as_ref()
            .map(|failure| ObservedShengjiThrowFailure {
                match_id,
                hand_number,
                player: failure.player,
                attempted: failure.attempted.clone(),
            });
    if state.observed_throw_failure != observed_throw_failure {
        if throw_failure
            .as_ref()
            .is_some_and(|failure| failure.stage == ShengjiThrowFailureStage::Showing)
        {
            queue_throw_failure_audio(&mut state.audio_cues, throw_penalty);
        }
        state.observed_throw_failure = observed_throw_failure;
    }

    let events = client.0.take_shengji_events();
    if events.is_empty() {
        state.observed_dealer = current_dealer;
        return;
    }
    for event in events {
        let trump_kill = match &event {
            ShengjiEvent::CardsPlayed { play, is_lead } => {
                state.winning_trump_kill(play, *is_lead, current_trump)
            }
            _ => None,
        };
        state.observe_trick_event(&event);
        match event {
            ShengjiEvent::DeclarationChanged { declaration } => {
                let label = match declaration.kind {
                    ShengjiBidKind::Initial => "亮主",
                    ShengjiBidKind::Protect => "自保",
                    ShengjiBidKind::Counter => "反主",
                    ShengjiBidKind::SelfCounter => "自反",
                };
                let start = state.activate(
                    ShengjiPresentationKind::Declaration {
                        player: declaration.player,
                        trump: declaration.trump,
                        label,
                    },
                    1.65,
                );
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Confirm,
                    start,
                    0.28,
                    3,
                ));
            }
            ShengjiEvent::BiddingLocked { .. } => {
                state
                    .audio_cues
                    .push(ShengjiAudioCue::new(ShengjiSoundKind::Lock, 0.0, 0.32, 5));
            }
            ShengjiEvent::PowerOutageDealerChanged { dealer, level } => {
                let from_dealer = state.observed_dealer;
                let start = state.activate(
                    ShengjiPresentationKind::PowerOutage {
                        from_dealer,
                        dealer,
                        level,
                    },
                    2.0,
                );
                state.audio_cues.extend([
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerDown, start, 0.52, 7),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerRelay, start + 0.34, 0.25, 9),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerRelay, start + 0.49, 0.29, 10),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerRelay, start + 0.64, 0.34, 12),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerUp, start + 0.78, 0.48, 11),
                ]);
            }
            ShengjiEvent::BottomCardRevealed { reveal } => {
                let matches = reveal.matches.clone();
                let dealer = reveal.dealer;
                let start = state.activate(
                    ShengjiPresentationKind::BottomFlip {
                        card: reveal.card,
                        matches: matches.clone(),
                        dealer,
                    },
                    2.65,
                );
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Flip,
                    start,
                    0.46,
                    13,
                ));
                if !matches.is_empty() {
                    state.audio_cues.push(ShengjiAudioCue::new(
                        ShengjiSoundKind::Confirm,
                        start + 0.82,
                        0.25,
                        14,
                    ));
                }
                if dealer.is_some() {
                    state.audio_cues.push(ShengjiAudioCue::new(
                        ShengjiSoundKind::Lock,
                        start + 1.72,
                        0.31,
                        15,
                    ));
                }
            }
            ShengjiEvent::BottomCopied { declaration } => {
                state.bottom_copy_count = state.bottom_copy_count.saturating_add(1);
                let count = state.bottom_copy_count;
                let from_player = state.bottom_burier;
                let start = state.activate(
                    ShengjiPresentationKind::BottomCopy {
                        from_player,
                        player: declaration.player,
                        trump: declaration.trump,
                        count,
                    },
                    1.75,
                );
                state.audio_cues.extend([
                    ShengjiAudioCue::new(ShengjiSoundKind::Copy, start, 0.38, 17),
                    ShengjiAudioCue::new(ShengjiSoundKind::Lock, start + 0.72, 0.26, 19),
                ]);
            }
            ShengjiEvent::FiveTrumpCrossingStarted { players } => {
                let start =
                    state.activate(ShengjiPresentationKind::CrossingStarted { players }, 1.85);
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Crossing,
                    start + 0.04,
                    0.48,
                    23,
                ));
            }
            ShengjiEvent::FiveTrumpCrossingReturned { player, complete } => {
                let start = state.activate(
                    ShengjiPresentationKind::CrossingReturned { player, complete },
                    if complete { 1.55 } else { 1.35 },
                );
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Crossing,
                    start,
                    0.34,
                    29,
                ));
            }
            ShengjiEvent::CardsBuried { dealer } => {
                state.bottom_burier = Some(dealer);
            }
            ShengjiEvent::CardsPlayed { play, is_lead } => {
                let kind = classify_play_presentation(&play.play);
                let start = if let Some(covered) = trump_kill {
                    let start = state.activate(
                        ShengjiPresentationKind::TrumpKill {
                            player: play.player,
                            covered,
                        },
                        1.12,
                    );
                    state.audio_cues.extend([
                        ShengjiAudioCue::new(
                            ShengjiSoundKind::TrumpKillLaunch,
                            start + 0.10,
                            0.34,
                            play.player.0 as u64 + 61,
                        ),
                        ShengjiAudioCue::new(
                            ShengjiSoundKind::TrumpKillImpact,
                            start + 0.52,
                            0.46,
                            play.player.0 as u64 + 67,
                        ),
                    ]);
                    start
                } else if should_show_play_presentation(kind, is_lead, play.throw_penalty) {
                    state.activate(
                        ShengjiPresentationKind::Play {
                            player: play.player,
                            kind,
                        },
                        kind.duration(),
                    )
                } else {
                    0.0
                };
                queue_play_audio(&mut state.audio_cues, kind, play.player.0 as u64, start);
            }
            _ => {}
        }
    }
    state.observed_dealer = current_dealer;
    ui.dirty = true;
}

pub(super) fn queue_throw_failure_audio(
    cues: &mut Vec<ShengjiAudioCue>,
    penalty: ShengjiThrowPenalty,
) {
    cues.extend([
        ShengjiAudioCue::new(ShengjiSoundKind::ThrowFail, 0.18, 0.44, 37),
        ShengjiAudioCue::new(ShengjiSoundKind::CardShove, 1.08, 0.38, 41),
    ]);
    match penalty {
        ShengjiThrowPenalty::None => {}
        ShengjiThrowPenalty::FivePerCard => cues.push(ShengjiAudioCue::new(
            ShengjiSoundKind::PenaltyFive,
            0.42,
            0.40,
            43,
        )),
        ShengjiThrowPenalty::TenPerCard => cues.extend([
            ShengjiAudioCue::new(ShengjiSoundKind::PenaltyTen, 0.42, 0.48, 47),
            ShengjiAudioCue::new(ShengjiSoundKind::Heavy, 0.49, 0.24, 53),
        ]),
    }
}

impl ShengjiPresentationState {
    pub(super) fn winning_trump_kill(
        &self,
        challenger: &ShengjiPublicPlay,
        is_lead: bool,
        trump: Option<ShengjiTrump>,
    ) -> Option<bool> {
        let trump = trump?;
        let lead = &self.current_trick_plays.first()?.play;
        if is_lead
            || !matches!(lead.category, Category::Suit(_))
            || challenger.play.category != Category::Trump
        {
            return None;
        }
        let mut winner = lead;
        for previous in self.current_trick_plays.iter().skip(1) {
            if compare_for_trick(lead, winner, &previous.play, trump).is_gt() {
                winner = &previous.play;
            }
        }
        compare_for_trick(lead, winner, &challenger.play, trump)
            .is_gt()
            .then_some(winner.category == Category::Trump)
    }

    pub fn has_previous_trick(&self) -> bool {
        self.previous_trick_plays.len() == SHENGJI_TRICK_PLAY_COUNT
    }

    pub fn revealed_previous_trick(&self) -> Option<&[ShengjiPublicPlay]> {
        (self.previous_trick_reveal_remaining > 0.0 && self.has_previous_trick())
            .then_some(self.previous_trick_plays.as_slice())
    }

    pub fn reveal_previous_trick(&mut self) {
        if self.has_previous_trick() {
            self.previous_trick_reveal_remaining = 2.0;
        }
    }

    fn clear_trick_history(&mut self) {
        self.current_trick_plays.clear();
        self.previous_trick_plays.clear();
        self.previous_trick_reveal_remaining = 0.0;
    }

    pub(super) fn observe_trick_event(&mut self, event: &ShengjiEvent) {
        match event {
            ShengjiEvent::CardsPlayed { play, is_lead } => {
                if *is_lead {
                    self.current_trick_plays.clear();
                }
                if let Some(existing) = self
                    .current_trick_plays
                    .iter_mut()
                    .find(|existing| existing.player == play.player)
                {
                    *existing = play.clone();
                } else {
                    self.current_trick_plays.push(play.clone());
                }
            }
            ShengjiEvent::TrickFinished { .. } => {
                if self.current_trick_plays.len() == SHENGJI_TRICK_PLAY_COUNT {
                    self.previous_trick_plays = std::mem::take(&mut self.current_trick_plays);
                } else {
                    self.current_trick_plays.clear();
                }
                self.previous_trick_reveal_remaining = 0.0;
            }
            _ => {}
        }
    }

    pub(super) fn activate(&mut self, kind: ShengjiPresentationKind, duration: f32) -> f32 {
        let start_delay = self
            .active
            .as_ref()
            .map(|active| (active.duration - active.elapsed).max(0.0))
            .unwrap_or(0.0)
            + self
                .queued
                .iter()
                .map(|presentation| presentation.duration)
                .sum::<f32>();
        let presentation = ActiveShengjiPresentation {
            kind,
            elapsed: 0.0,
            duration,
        };
        if self.active.is_none() {
            self.active = Some(presentation);
        } else {
            self.queued.push_back(presentation);
        }
        start_delay
    }
}
