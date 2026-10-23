use super::super::ShengjiPlayFeedback;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerGender};
use leocard_shengji::{Category, Component as PlayComponent, ShengjiClassifiedPlay};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShengjiVoiceKind {
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

    pub(super) fn for_play(play: &ShengjiClassifiedPlay, is_lead: bool) -> Option<Self> {
        if play.category == Category::Mixed {
            return None;
        }
        if play.is_throw() {
            // 跟牌时凑出的多个结构只是垫牌，不能作为甩牌报出。
            return is_lead.then_some(Self::Throw);
        }
        match play.strongest_component() {
            PlayComponent::Single { .. } | PlayComponent::Pair { .. } => {
                (is_lead && play.category == Category::Trump).then_some(Self::TrumpLead)
            }
            PlayComponent::Tractor { .. } => Some(Self::Tractor),
            PlayComponent::Triple { .. } => Some(Self::Triple),
            PlayComponent::Titanic { .. } => Some(Self::Titanic),
            PlayComponent::Quad { .. } => Some(Self::Bomb),
            PlayComponent::Spaceship { .. } => Some(Self::Spaceship),
        }
    }
}

pub(super) struct ShengjiVoiceCue {
    pub(super) gender: PlayerGender,
    pub(super) kind: ShengjiVoiceKind,
}

#[derive(Resource, Default)]
pub(super) struct ShengjiVoiceState {
    pub(super) observed: Option<(MatchId, u32)>,
    pub(super) pending: VecDeque<ShengjiVoiceCue>,
}

/// 只消费已接受的实时事件；快照重建和上一墩回看不重复报牌。
pub(super) fn sync_shengji_voices(
    client: Option<Res<ClientResource>>,
    mut feedback: MessageReader<ShengjiPlayFeedback>,
    mut voices: ResMut<ShengjiVoiceState>,
) {
    let events = feedback.read();
    let Some(game) = client
        .as_ref()
        .and_then(|client| client.0.model().shengji_game())
    else {
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
            .find(|player| player.id == event.play.player)
        else {
            continue;
        };
        for kind in [
            ShengjiVoiceKind::for_play(&event.play.play, event.is_lead),
            event.trump_kill.then_some(ShengjiVoiceKind::TrumpKill),
        ]
        .into_iter()
        .flatten()
        {
            voices.pending.push_back(ShengjiVoiceCue {
                gender: player.game_profiles.gender,
                kind,
            });
        }
    }
}
