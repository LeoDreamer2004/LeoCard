use super::super::ShengjiPlayFeedback;
use super::assets::ShengjiVoiceAssets;
use crate::app::games::GameFeedback;
use crate::app::presentation::{CardVoice, CardVoices};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, MatchId};
use leocard_shengji::{ShengjiCategory, ShengjiClassifiedPlay, ShengjiComponent};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShengjiVoiceKind {
    Tractor,
    Triple,
    Titanic,
    Bomb,
    Spaceship,
    Throw,
    TrumpLead,
    TrumpKill,
}

impl ShengjiVoiceKind {
    pub(super) const ALL: [Self; 8] = [
        Self::Tractor,
        Self::Triple,
        Self::Titanic,
        Self::Bomb,
        Self::Spaceship,
        Self::Throw,
        Self::TrumpLead,
        Self::TrumpKill,
    ];

    pub(super) const fn asset_name(self) -> &'static str {
        match self {
            Self::Tractor => "tractor",
            Self::Triple => "triple",
            Self::Titanic => "titanic",
            Self::Bomb => "bomb",
            Self::Spaceship => "spaceship",
            Self::Throw => "throw",
            Self::TrumpLead => "trump_lead",
            Self::TrumpKill => "trump_kill",
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct ShengjiVoiceState {
    pub(super) observed: Option<(MatchId, u32)>,
    pub(super) pending: VecDeque<Handle<AudioSource>>,
}

/// 只消费已接受的实时事件，快照重建和上一墩回看不重复报牌。
pub(super) fn sync_shengji_voices(
    client: Option<Res<ClientResource>>,
    mut feedback: MessageReader<ShengjiPlayFeedback>,
    mut voices: ResMut<ShengjiVoiceState>,
    assets: Res<ShengjiVoiceAssets>,
    card_voices: Res<CardVoices>,
) {
    let events = feedback.read();
    let Some(game) = client.as_ref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::shengji)
    }) else {
        events.for_each(|_| {});
        *voices = ShengjiVoiceState::default();
        return;
    };
    let observed = Some((game.match_id, game.hand_number));
    if voices.observed != observed {
        voices.pending.clear();
        voices.observed = observed;
    }
    for event in events {
        if event.match_id != game.match_id || event.hand_number != game.hand_number {
            continue;
        }
        let Some(player) = game
            .players
            .iter()
            .find(|player| player.player.id == event.play.player)
        else {
            continue;
        };
        let gender = player.player.game_profiles.gender;
        let kinds = [
            event.play.play.feedback(event.is_lead),
            event.trump_kill.then_some(ShengjiVoiceKind::TrumpKill),
        ];
        // 吊主、毙牌等特殊播报优先，不再叠加单牌或对子的点数语音。
        if kinds.iter().all(Option::is_none)
            && let Some(pattern) = event.play.play.feedback(())
        {
            voices
                .pending
                .push_back(card_voices.random_clip(gender, pattern));
        }
        for kind in kinds.into_iter().flatten() {
            voices.pending.push_back(assets.random_clip(gender, kind));
        }
    }
}

impl GameFeedback<()> for ShengjiClassifiedPlay {
    type Cue = Option<CardVoice>;

    fn feedback(&self, _context: ()) -> Self::Cue {
        match self.components.as_slice() {
            [ShengjiComponent::Single { card, .. }] => Some(CardVoice::Single((*card).into())),
            [ShengjiComponent::Pair { cards, .. }] => Some(CardVoice::Pair(cards[0].into())),
            _ => None,
        }
    }
}

impl GameFeedback<bool> for ShengjiClassifiedPlay {
    type Cue = Option<ShengjiVoiceKind>;

    fn feedback(&self, is_lead: bool) -> Self::Cue {
        if self.category == ShengjiCategory::Mixed {
            return None;
        }
        if self.is_throw() {
            // 跟牌时凑出的多个结构只是垫牌，不能作为甩牌报出。
            return is_lead.then_some(ShengjiVoiceKind::Throw);
        }
        match self.strongest_component() {
            ShengjiComponent::Single { .. } | ShengjiComponent::Pair { .. } => (is_lead
                && self.category == ShengjiCategory::Trump)
                .then_some(ShengjiVoiceKind::TrumpLead),
            ShengjiComponent::Tractor { .. } => Some(ShengjiVoiceKind::Tractor),
            ShengjiComponent::Triple { .. } => Some(ShengjiVoiceKind::Triple),
            ShengjiComponent::Titanic { .. } => Some(ShengjiVoiceKind::Titanic),
            ShengjiComponent::Quad { .. } => Some(ShengjiVoiceKind::Bomb),
            ShengjiComponent::Spaceship { .. } => Some(ShengjiVoiceKind::Spaceship),
        }
    }
}
