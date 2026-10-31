use crate::app::runtime::{ClientResource, ServerNotification};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameEvent, GameSnapshot, MatchId, PlayerGender, ServerEvent, TexasEvent, TexasPhaseView,
};

use leocard_texas_holdem::{TexasAction, TexasRuleSet, TexasStreet};
use std::collections::{HashMap, VecDeque};

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasVoicePlayback;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TexasVoiceKind {
    Raise,
    AllIn,
    Call,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TexasVoiceCue {
    pub(super) gender: PlayerGender,
    pub(super) kind: TexasVoiceKind,
    pub(super) variant: usize,
}

pub(super) fn texas_action_voice_plan(
    action: TexasAction,
    gender: PlayerGender,
    seed: u64,
) -> Option<TexasVoiceCue> {
    let (kind, variant) = match action {
        TexasAction::RaiseTo(target) => (TexasVoiceKind::Raise, usize::from(target >= 10)),
        TexasAction::AllIn => (TexasVoiceKind::AllIn, 2 + seed as usize % 3),
        TexasAction::Call => (TexasVoiceKind::Call, 5 + seed as usize % 2),
        _ => return None,
    };
    Some(TexasVoiceCue {
        gender,
        kind,
        variant,
    })
}

#[derive(Resource, Default)]
pub(crate) struct TexasVoiceState {
    observed: Option<(MatchId, u32)>,
    committed: HashMap<PlayerId, u32>,
    bet: u32,
    raised: bool,
    serial: u64,
    pub(super) pending: VecDeque<TexasVoiceCue>,
    pub(super) busy_for: f32,
}

/// Voice state follows accepted game facts independently of the chip ledger.
pub(crate) fn sync_texas_voices(
    client: Option<Res<ClientResource>>,
    mut notifications: MessageReader<ServerNotification>,
    mut voices: ResMut<TexasVoiceState>,
) {
    let events = notifications
        .read()
        .filter_map(|notification| match &notification.event {
            ServerEvent::GameEvent(GameEvent::Texas(event)) => Some(event),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some(game) = client.as_ref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::texas)
    }) else {
        *voices = TexasVoiceState::default();
        return;
    };
    let observed = Some((game.match_id, game.hand_number));
    if voices.observed != observed {
        *voices = TexasVoiceState::default();
        voices.observed = observed;
        voices.committed = game
            .players
            .iter()
            .map(|player| (player.player.id, player.committed_street))
            .collect();
        voices.bet = game.current_bet;
        voices.raised = match game.phase {
            TexasPhaseView::Betting {
                street: TexasStreet::PreFlop,
            } => game.current_bet > TexasRuleSet::BIG_BLIND,
            TexasPhaseView::Betting { .. } => game.current_bet > 0,
            TexasPhaseView::HandComplete { .. } => false,
        };
        return;
    }
    for event in events {
        voices.serial = voices.serial.wrapping_add(1);
        match *event {
            TexasEvent::ActionApplied {
                player,
                action,
                amount,
                ..
            } => {
                let committed = voices
                    .committed
                    .get(&player)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(amount);
                let raises = matches!(action, TexasAction::RaiseTo(_))
                    || (action == TexasAction::AllIn && committed > voices.bet);
                if (action != TexasAction::Call || voices.raised)
                    && let Some(profile) = game
                        .players
                        .iter()
                        .find(|profile| profile.player.id == player)
                    && let Some(cue) = texas_action_voice_plan(
                        action,
                        profile.player.game_profiles.gender,
                        voices.serial.wrapping_mul(17),
                    )
                {
                    voices.pending.push_back(cue);
                }
                voices.committed.insert(player, committed);
                voices.bet = voices.bet.max(committed);
                voices.raised |= raises;
            }
            TexasEvent::StreetAdvanced { .. } => {
                voices.committed.clear();
                voices.bet = 0;
                voices.raised = false;
            }
            _ => {}
        }
    }
    voices.committed = game
        .players
        .iter()
        .map(|player| (player.player.id, player.committed_street))
        .collect();
    voices.bet = game.current_bet;
}
