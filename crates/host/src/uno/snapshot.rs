use super::{UnoSession, map_game_error};
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameEvent, GameViolation, RejectReason, RequestId, ServerEvent, UnoEvent, UnoPendingSwapView,
    UnoPhaseView, UnoPlayerResult, UnoPlayerView, UnoRevealedHand, UnoSnapshot,
};
use leocard_uno::{UnoCard, UnoGameError, UnoPendingSwap, UnoPhase};

impl UnoSession {
    pub(super) fn broadcast_events(&self, events: Vec<UnoEvent>) -> Vec<Delivery> {
        self.room
            .broadcast_game_events(events, self.match_id.map(|id| (id, None)))
            .into_iter()
            .filter(|delivery| match &delivery.message.event {
                ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ActionAnalyzed {
                    player, ..
                })) => self.room.player_id(delivery.recipient) == Some(*player),
                _ => true,
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
                let public = participant.public_metadata();
                let state = game
                    .player(public.player.id)
                    .expect("room and UNO core players stay aligned");
                let mut inactive_hand = if self.rules.is_flip() && public.player.id != recipient {
                    state
                        .hand()
                        .iter()
                        .filter_map(|card| card.opposite_public_face())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                inactive_hand.sort_unstable_by_key(|card| (card.color(), card.face(), card.copy()));
                UnoPlayerView {
                    seat: public.seat.expect("started players retain seats"),
                    hand_len: state.hand().len() as u8,
                    inactive_hand,
                    ready: public.ready,
                    connected: public.connected,
                    auto_play: public.auto_play,
                    skipped_turns: game.skipped_turns(public.player.id).unwrap_or(0),
                    eliminated: state.eliminated(),
                    player: public.player,
                }
            })
            .collect();
        let phase = match game.phase() {
            UnoPhase::Playing => UnoPhaseView::Playing,
            UnoPhase::Finished(result) => UnoPhaseView::Finished {
                winner: result.winner,
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
                        player: player.id(),
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
                .player(recipient)
                .into_iter()
                .flat_map(|player| player.hand().iter())
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
            current_player: turn.map(|turn| turn.current_player),
            direction: game.direction(),
            pending_draw: turn.map_or(0, |turn| turn.pending_draw),
            pending_kind: turn.and_then(|turn| turn.pending_kind),
            sudden_death_free_play: turn.is_some_and(|turn| turn.sudden_death_free_play),
            challenge_offender: turn.and_then(|turn| turn.challenge_offender),
            pending_skip: turn.map_or(0, |turn| turn.pending_skip),
            pending_swap: game.pending_swap().map(|pending| match pending {
                UnoPendingSwap::SwapOneTarget { player } => {
                    UnoPendingSwapView::SwapOneTarget { player }
                }
                UnoPendingSwap::SwapOneGive { player, target } => {
                    UnoPendingSwapView::SwapOneGive { player, target }
                }
                UnoPendingSwap::ForceTrade { player } => UnoPendingSwapView::ForceTrade { player },
                UnoPendingSwap::ChooseColor { player } => {
                    UnoPendingSwapView::ChooseColor { player }
                }
                UnoPendingSwap::SevenSwap { player } => UnoPendingSwapView::SevenSwap { player },
                UnoPendingSwap::ColorRoulette { player } => {
                    UnoPendingSwapView::ColorRoulette { player }
                }
            }),
            your_drawn_card: turn
                .filter(|turn| turn.current_player == recipient)
                .and_then(|turn| turn.drawn_card)
                .map(UnoCard::public_face),
            your_jump_in_card: game.jump_in_card(recipient).map(UnoCard::public_face),
            uno_exposed: game.uno_exposed_players().collect(),
            uno_declared: game.uno_declared_players().collect(),
            can_call_uno: game.can_call_uno(recipient),
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
                .find(|player| player.player.id == reveal.player)
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
        error: &UnoGameError,
    ) -> Vec<Delivery> {
        self.room.reject(
            connection,
            request_id,
            RejectReason::Game(GameViolation::Uno(map_game_error(error))),
        )
    }
}
