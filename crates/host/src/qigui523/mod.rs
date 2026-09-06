use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use leocard_protocol::{
    ChatContent, ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GamePhaseView,
    GameRules, GameSnapshot, GameViolation, JoinRequest, LobbySnapshot, MAX_PLAYER_NAME_CHARS,
    MatchId, PlayerId, PlayerInteraction, PlayerInteractionKind, PlayerPublicState,
    PlayerReferenceChange, PlayerScore, PublicPlay, PublicPlayRecord, QiGui523Command,
    QiGui523Event, QiGui523ProfileStats, QiGui523Snapshot, RejectReason, RequestId, RevealedHand,
    Revision, RoomId, RuleViolation, SeatId, ServerEvent, StartingCardView, TABLE_SEAT_COUNT,
    TrickView, TurnTimerView,
};
use leocard_qigui523::{
    GameError, GameState, Phase, PlayError, PlayRecord, QiGui523Bot, QiGui523BotRequest, QiGuiCard,
    QiGuiPlayKind, QiGuiPlayerId, QiGuiRuleSet, build_deck, classify, reference_point_deltas,
};

use crate::room::Participant;
use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    TurnTimerState, new_match_id, valid_identity_proof,
};

mod automation;
mod commands;
mod lifecycle;
mod snapshot;

#[cfg(test)]
mod tests;

/// 单房间权威会话。所有命令均按调用顺序串行处理。
#[derive(Clone, Debug)]
pub struct QiGui523Session {
    room: RoomSession,
    rules: QiGuiRuleSet,
    shuffled_deck: Option<Vec<QiGuiCard>>,
    game: Option<GameState>,
    match_id: Option<MatchId>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    match_profile_stats: Vec<QiGui523ProfileStats>,
    turn_timer: Option<TurnTimerState>,
    auto_play_delay: Option<AutoPlayDelayState>,
}

impl Deref for QiGui523Session {
    type Target = RoomSession;

    fn deref(&self) -> &Self::Target {
        &self.room
    }
}

impl DerefMut for QiGui523Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.room
    }
}

fn validate_deck(deck_count: u8, deck: &[QiGuiCard]) -> Result<(), HostError> {
    let expected_deck = build_deck(deck_count);
    if deck.len() != expected_deck.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = expected_deck.into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

fn record_qigui523_play(stats: &mut QiGui523ProfileStats, kind: &QiGuiPlayKind) {
    match kind {
        QiGuiPlayKind::Straight { card_count } => {
            stats.straight_plays = stats.straight_plays.saturating_add(1);
            stats.longest_straight = stats
                .longest_straight
                .max(u16::try_from(*card_count).unwrap_or(u16::MAX));
        }
        QiGuiPlayKind::ConsecutivePairs { pair_count } => {
            stats.consecutive_pair_plays = stats.consecutive_pair_plays.saturating_add(1);
            stats.longest_consecutive_pairs = stats
                .longest_consecutive_pairs
                .max(u16::try_from(*pair_count).unwrap_or(u16::MAX));
        }
        QiGuiPlayKind::Airplane { triple_count } => {
            stats.airplane_plays = stats.airplane_plays.saturating_add(1);
            stats.longest_airplane = stats
                .longest_airplane
                .max(u16::try_from(*triple_count).unwrap_or(u16::MAX));
        }
        QiGuiPlayKind::Bomb(_) => stats.bomb_plays = stats.bomb_plays.saturating_add(1),
        QiGuiPlayKind::HeavenBomb => {
            stats.heaven_bomb_plays = stats.heaven_bomb_plays.saturating_add(1);
        }
        QiGuiPlayKind::Single
        | QiGuiPlayKind::Pair
        | QiGuiPlayKind::Triple
        | QiGuiPlayKind::TripleWithSingle
        | QiGuiPlayKind::TripleWithPair => {}
    }
}

fn merge_qigui523_play_stats(aggregate: &mut QiGui523ProfileStats, current: &QiGui523ProfileStats) {
    aggregate.straight_plays = aggregate
        .straight_plays
        .saturating_add(current.straight_plays);
    aggregate.consecutive_pair_plays = aggregate
        .consecutive_pair_plays
        .saturating_add(current.consecutive_pair_plays);
    aggregate.airplane_plays = aggregate
        .airplane_plays
        .saturating_add(current.airplane_plays);
    aggregate.bomb_plays = aggregate.bomb_plays.saturating_add(current.bomb_plays);
    aggregate.heaven_bomb_plays = aggregate
        .heaven_bomb_plays
        .saturating_add(current.heaven_bomb_plays);
    aggregate.longest_straight = aggregate.longest_straight.max(current.longest_straight);
    aggregate.longest_consecutive_pairs = aggregate
        .longest_consecutive_pairs
        .max(current.longest_consecutive_pairs);
    aggregate.longest_airplane = aggregate.longest_airplane.max(current.longest_airplane);
}

fn to_core_player(player: PlayerId) -> QiGuiPlayerId {
    QiGuiPlayerId(usize::from(player.0))
}

fn from_core_player(player: QiGuiPlayerId) -> PlayerId {
    PlayerId(u8::try_from(player.0).expect("core supports at most six players"))
}

fn map_game_error(error: &GameError) -> RuleViolation {
    match error {
        GameError::InvalidPlayer(_) => RuleViolation::InvalidPlayer,
        GameError::NotPlayersTurn { .. } => RuleViolation::NotPlayersTurn,
        GameError::GameAlreadyFinished => RuleViolation::GameAlreadyFinished,
        GameError::MustLeadWithCards => RuleViolation::MustLeadWithCards,
        GameError::CardNotInHand(_) => RuleViolation::CardNotInHand,
        GameError::InvalidPlay(
            PlayError::Empty
            | PlayError::DuplicatePhysicalCard(_)
            | PlayError::CardOutsideConfiguredDeck(_)
            | PlayError::InvalidPattern,
        )
        | GameError::InvalidRules(_)
        | GameError::InvalidDeckSize { .. }
        | GameError::InvalidDeckContents => RuleViolation::InvalidPattern,
        GameError::PlayDoesNotBeatCurrent => RuleViolation::PlayDoesNotBeatCurrent,
    }
}

fn duration_ceil_seconds(duration: Duration) -> u16 {
    let milliseconds = duration.as_millis();
    u16::try_from(milliseconds.div_ceil(1_000)).unwrap_or(u16::MAX)
}
