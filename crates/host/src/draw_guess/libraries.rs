use super::DrawGuessSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_draw_guess::{DrawGuessError, WordLibrary};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    DrawGuessCommand, DrawGuessLibraryEvent, GameViolation, RejectReason, RequestId, RoomViolation,
    ServerEvent, WordLibraryOffer,
};

#[derive(Clone, Debug)]
pub(super) struct HostedWordLibraries {
    pub selected: WordLibrary,
    pub offers: Vec<WordLibraryOffer>,
    next_offer: u64,
}

impl Default for HostedWordLibraries {
    fn default() -> Self {
        Self {
            selected: WordLibrary::builtin(),
            offers: Vec::new(),
            next_offer: 0,
        }
    }
}

impl DrawGuessSession {
    pub(super) fn library_state(&self, recipient: PlayerId) -> ServerEvent {
        let host = self.room.host_player_id() == Some(recipient);
        let offers =
            self.libraries
                .offers
                .iter()
                .filter(|offer| {
                    (host || offer.sender == recipient)
                        && self.room.participants().any(|player| {
                            player.id == offer.sender && player.connected && !player.left
                        })
                })
                .cloned()
                .collect();
        ServerEvent::GameEvent(
            DrawGuessLibraryEvent::State {
                selected: (&self.libraries.selected).into(),
                offers,
            }
            .into(),
        )
    }

    pub(super) fn handle_library_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: DrawGuessCommand,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self.require_player(connection)?;
        if self.game.is_some() {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        let host = self.room.host_connection == Some(connection);
        let mut extra = Vec::new();
        match command {
            DrawGuessCommand::SelectLibrary { library } => {
                if !host {
                    return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
                }
                self.libraries.selected = library
                    .unwrap_or_else(WordLibrary::builtin)
                    .validate()
                    .map_err(|error| RejectReason::Game(error.into()))?;
                self.room.reset_ready_after_rules_change();
            }
            DrawGuessCommand::UploadLibrary { library } => {
                if host {
                    return Err(RejectReason::Game(GameViolation::InvalidRuleConfiguration));
                }
                self.libraries.offers.retain(|offer| {
                    self.room.participants().any(|participant| {
                        participant.id == offer.sender && participant.connected && !participant.left
                    })
                });
                if self
                    .libraries
                    .offers
                    .iter()
                    .any(|offer| offer.sender == player)
                {
                    return Err(RejectReason::Game(
                        DrawGuessError::WordLibraryOfferPending.into(),
                    ));
                }
                let library = library
                    .validate()
                    .map_err(|error| RejectReason::Game(error.into()))?;
                let sender_name = self
                    .room
                    .participants()
                    .find(|participant| participant.id == player)
                    .expect("joined player")
                    .name
                    .clone();
                self.libraries.next_offer += 1;
                self.libraries.offers.push(WordLibraryOffer {
                    id: self.libraries.next_offer,
                    sender: player,
                    sender_name,
                    library,
                });
            }
            DrawGuessCommand::ResolveLibraryUpload { offer_id, accept } => {
                if !host {
                    return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
                }
                let index = self
                    .libraries
                    .offers
                    .iter()
                    .position(|offer| offer.id == offer_id)
                    .ok_or(RejectReason::Game(
                        DrawGuessError::WordLibraryOfferUnavailable.into(),
                    ))?;
                let offer = self.libraries.offers.remove(index);
                if let Some(sender) = self
                    .room
                    .participants()
                    .find(|sender| sender.id == offer.sender && sender.connected && !sender.left)
                {
                    extra.push((
                        sender.connection,
                        DrawGuessLibraryEvent::UploadResolved {
                            title: offer.library.title,
                            accepted: accept,
                        },
                    ));
                }
            }
            _ => unreachable!("library commands only"),
        }
        self.room.bump_revision();
        let mut deliveries = self.broadcast_lobby(Some((connection, request)));
        deliveries.extend(extra.into_iter().map(|(recipient, event)| {
            self.room
                .delivery(recipient, None, ServerEvent::GameEvent(event.into()))
        }));
        Ok(deliveries)
    }
}
