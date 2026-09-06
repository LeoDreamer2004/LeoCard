use super::*;
use leocard_protocol::{
    GameEvent, GameKind, GameRules, GameSnapshot, GameViolation, LobbySnapshot, PlayerId,
    RejectReason, RequestId, ServerEvent, UnoEvent, UnoPendingSwapView, UnoPhaseView,
    UnoPlayerResult, UnoPlayerState, UnoRevealedHand, UnoSnapshot,
};
use leocard_uno::{GameError, PendingSwap, Phase, UnoCard};

impl UnoSession {
    pub(super) fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::Uno, GameRules::Uno(self.rules))
    }

    pub(super) fn broadcast_lobby(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::Uno, GameRules::Uno(self.rules), origin)
    }

    pub(super) fn broadcast_game(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        assert!(self.game.is_some(), "game broadcast requires an UNO game");
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                self.room.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::Uno(self.game_snapshot(player.id))),
                )
            })
            .collect()
    }

    pub(super) fn broadcast_events(&self, events: Vec<UnoEvent>) -> Vec<Delivery> {
        events
            .into_iter()
            .flat_map(|event| {
                self.room
                    .players
                    .iter()
                    .filter(|player| player.connected && !player.left)
                    .map(move |player| {
                        self.room.delivery(
                            player.connection,
                            None,
                            ServerEvent::GameEvent(GameEvent::Uno(event.clone())),
                        )
                    })
            })
            .collect()
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> UnoSnapshot {
        let game = self.game.as_ref().expect("an UNO snapshot requires a game");
        let turn = game.turn();
        let players = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|participant| {
                let state = game
                    .player(to_core_player(participant.id))
                    .expect("room and UNO core players stay aligned");
                let mut inactive_hand = if self.rules.is_flip() && participant.id != recipient {
                    state
                        .hand()
                        .iter()
                        .filter_map(|card| card.opposite_public_face())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                inactive_hand.sort_unstable_by_key(|card| (card.color(), card.face(), card.copy()));
                UnoPlayerState {
                    id: participant.id,
                    profile_id: participant.profile_id,
                    name: participant.name.clone(),
                    avatar: participant.avatar,
                    seat: participant.seat.expect("started players retain seats"),
                    hand_len: state.hand().len() as u8,
                    inactive_hand,
                    ready: participant.ready,
                    connected: (participant.connected || participant.is_bot) && !participant.left,
                    auto_play: participant.auto_play,
                    reference_points: participant.reference_points,
                    completed_games: participant.completed_games,
                    game_profiles: participant.game_profiles.clone(),
                    skipped_turns: game
                        .skipped_turns(to_core_player(participant.id))
                        .unwrap_or(0),
                    eliminated: state.eliminated(),
                }
            })
            .collect();
        let phase = match game.phase() {
            Phase::Playing => UnoPhaseView::Playing,
            Phase::Finished(result) => UnoPhaseView::Finished {
                winner: from_core_player(result.winner),
                results: result
                    .hand_scores
                    .iter()
                    .enumerate()
                    .map(|(index, hand_score)| UnoPlayerResult {
                        player: PlayerId(index as u8),
                        hand_score: *hand_score,
                        placement: result.placements[index],
                    })
                    .collect(),
                remaining_hands: game
                    .players()
                    .iter()
                    .map(|player| UnoRevealedHand {
                        player: from_core_player(player.id()),
                        cards: player
                            .hand()
                            .iter()
                            .map(|card| card.public_face())
                            .collect(),
                    })
                    .collect(),
                reference_changes: self
                    .finished_reference_changes
                    .clone()
                    .expect("UNO points are applied before final snapshot"),
            },
        };
        let mut snapshot = UnoSnapshot {
            match_id: self.match_id.expect("a running UNO game has a match id"),
            host_port: self.room.host_port,
            you: recipient,
            host: self
                .room
                .host_player_id()
                .expect("a running room has a host"),
            rules: self.rules,
            players,
            your_hand: game
                .player(to_core_player(recipient))
                .expect("recipient belongs to game")
                .hand()
                .iter()
                .map(|card| card.public_face())
                .collect(),
            draw_pile_len: game.draw_pile_len() as u16,
            draw_pile_inactive_cards: game
                .draw_pile()
                .take(6)
                .filter_map(UnoCard::opposite_public_face)
                .collect(),
            discard_top: game.top_card().public_face(),
            discard_pile: game
                .discard_pile()
                .iter()
                .rev()
                .take(6)
                .map(|card| card.public_face())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            current_color: game.current_color(),
            flip_side: game.flip_side(),
            current_player: turn.map(|turn| from_core_player(turn.current_player)),
            direction: game.direction(),
            pending_draw: turn.map_or(0, |turn| turn.pending_draw),
            pending_kind: turn.and_then(|turn| turn.pending_kind),
            challenge_offender: turn
                .and_then(|turn| turn.challenge_offender)
                .map(from_core_player),
            pending_skip: turn.map_or(0, |turn| turn.pending_skip),
            pending_swap: game.pending_swap().map(|pending| match pending {
                PendingSwap::SwapOneTarget { player } => UnoPendingSwapView::SwapOneTarget {
                    player: from_core_player(player),
                },
                PendingSwap::SwapOneGive { player, target } => UnoPendingSwapView::SwapOneGive {
                    player: from_core_player(player),
                    target: from_core_player(target),
                },
                PendingSwap::ForceTrade { player } => UnoPendingSwapView::ForceTrade {
                    player: from_core_player(player),
                },
                PendingSwap::ChooseColor { player } => UnoPendingSwapView::ChooseColor {
                    player: from_core_player(player),
                },
                PendingSwap::SevenSwap { player } => UnoPendingSwapView::SevenSwap {
                    player: from_core_player(player),
                },
                PendingSwap::ColorRoulette { player } => UnoPendingSwapView::ColorRoulette {
                    player: from_core_player(player),
                },
            }),
            your_drawn_card: turn
                .filter(|turn| from_core_player(turn.current_player) == recipient)
                .and_then(|turn| turn.drawn_card)
                .map(UnoCard::public_face),
            your_jump_in_card: game
                .jump_in_card(to_core_player(recipient))
                .map(UnoCard::public_face),
            uno_exposed: game.uno_exposed_players().map(from_core_player).collect(),
            uno_declared: game.uno_declared_players().map(from_core_player).collect(),
            can_call_uno: game.can_call_uno(to_core_player(recipient)),
            phase,
        };
        if let Some(reveal) = self.pending_draw_reveal.as_ref() {
            let hidden_cards = &reveal.cards[reveal.revealed..];
            let hidden_count = u8::try_from(hidden_cards.len()).unwrap_or(u8::MAX);
            let pending_backs = hidden_cards
                .iter()
                .filter_map(|card| card.opposite_public_face())
                .chain(snapshot.draw_pile_inactive_cards.iter().copied())
                .take(6)
                .collect::<Vec<_>>();
            if !pending_backs.is_empty() {
                snapshot.draw_pile_inactive_cards = pending_backs;
            }
            if let Some(player) = snapshot
                .players
                .iter_mut()
                .find(|player| player.id == reveal.player)
            {
                player.hand_len = player.hand_len.saturating_sub(hidden_count);
                let hidden_inactive = hidden_cards
                    .iter()
                    .filter_map(|card| card.opposite_public_face())
                    .collect::<Vec<_>>();
                for card in hidden_inactive {
                    if let Some(index) = player
                        .inactive_hand
                        .iter()
                        .position(|candidate| *candidate == card)
                    {
                        player.inactive_hand.remove(index);
                    }
                }
            }
            snapshot.draw_pile_len = snapshot
                .draw_pile_len
                .saturating_add(u16::from(hidden_count));
            snapshot.current_player = None;
            snapshot.your_jump_in_card = None;
            if recipient == reveal.player {
                snapshot.your_hand.retain(|card| {
                    !hidden_cards
                        .iter()
                        .any(|hidden| hidden.public_face() == *card)
                });
                if snapshot.your_drawn_card.is_some_and(|card| {
                    hidden_cards
                        .iter()
                        .any(|hidden| hidden.public_face() == card)
                }) {
                    snapshot.your_drawn_card = None;
                }
            }
            snapshot.can_call_uno =
                snapshot.your_hand.len() == 1 && snapshot.uno_exposed.contains(&snapshot.you);
        }
        snapshot
    }

    pub(super) fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: &GameError,
    ) -> Vec<Delivery> {
        self.room.reject(
            connection,
            request_id,
            RejectReason::GameViolation(GameViolation::Uno(map_game_error(error))),
        )
    }
}
