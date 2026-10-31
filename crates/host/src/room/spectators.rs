use super::{Participant, RoomSession};
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    JoinRequest, PlayerViolation, RejectReason, RequestId, RoomViolation, ServerEvent,
};

const MAX_SPECTATORS: usize = 8;
const FIRST_SPECTATOR_ID: u8 = 128;

impl RoomSession {
    pub(crate) fn viewing_player(&self, recipient: PlayerId) -> PlayerId {
        if !self
            .spectators
            .iter()
            .any(|player| player.id == recipient && !player.left)
        {
            return recipient;
        }
        self.spectator_views
            .get(&recipient)
            .copied()
            .filter(|id| {
                self.players
                    .iter()
                    .any(|player| player.id == *id && !player.left)
            })
            .or_else(|| {
                self.players
                    .iter()
                    .find(|player| !player.left)
                    .map(|player| player.id)
            })
            .unwrap_or(recipient)
    }

    pub(crate) fn set_spectator_view(
        &mut self,
        connection: ConnectionId,
        target: PlayerId,
    ) -> Result<(), RejectReason> {
        if !self.is_spectator(connection)
            || !self
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
        {
            return Err(RejectReason::Player(PlayerViolation::NotJoined));
        }
        let spectator = self.player_id(connection).expect("joined spectator");
        self.spectator_views.insert(spectator, target);
        self.bump_revision();
        Ok(())
    }

    pub(crate) fn spectator_join_notice(&self, connection: ConnectionId) -> Vec<Delivery> {
        let Some(player) = self
            .spectators
            .iter()
            .find(|player| player.connection == connection && !player.left)
        else {
            return Vec::new();
        };
        self.broadcast_event(
            None,
            ServerEvent::SpectatorJoined {
                name: player.name.clone(),
            },
        )
        .into_iter()
        .filter(|delivery| {
            self.players
                .iter()
                .any(|player| player.connection == delivery.recipient)
        })
        .collect()
    }

    pub(crate) fn participants(&self) -> impl Iterator<Item = &Participant> {
        self.players.iter().chain(&self.spectators)
    }

    pub(crate) fn participants_mut(&mut self) -> impl Iterator<Item = &mut Participant> {
        self.players.iter_mut().chain(&mut self.spectators)
    }

    pub(super) fn participant_mut(&mut self, id: PlayerId) -> Option<&mut Participant> {
        self.participants_mut()
            .find(|player| player.id == id && !player.left)
    }

    pub(crate) fn is_spectator(&self, connection: ConnectionId) -> bool {
        self.spectators
            .iter()
            .any(|player| player.connection == connection && !player.left)
    }

    pub(crate) fn has_spectators(&self) -> bool {
        self.spectators
            .iter()
            .any(|player| player.connected && !player.left)
    }

    pub(super) fn join_spectator(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        request: JoinRequest,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let vacant = self.spectators.iter().position(|player| player.left);
        let index = vacant.unwrap_or(self.spectators.len());
        if index >= MAX_SPECTATORS {
            return Err(RejectReason::Room(RoomViolation::RoomFull));
        }
        let id = PlayerId(FIRST_SPECTATOR_ID + index as u8);
        self.spectator_views.remove(&id);
        let participant = Participant {
            id,
            profile_id: request.profile_id,
            connection,
            name: request.name.trim().to_owned(),
            avatar: None,
            avatar_png: None,
            reconnect_token: request.reconnect_token,
            seat: None,
            ready: false,
            connected: true,
            auto_play: false,
            is_bot: false,
            left: false,
            reference_points: request.reference_points,
            completed_games: request.completed_games,
            game_profiles: request.game_profiles,
        };
        if let Some(index) = vacant {
            self.spectators[index] = participant;
        } else {
            self.spectators.push(participant);
        }
        self.bump_revision();
        Ok(self.join_deliveries(connection, request_id, id))
    }

    pub(crate) fn disconnect_spectator(&mut self, connection: ConnectionId) -> bool {
        let Some(player) = self
            .spectators
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
        else {
            return false;
        };
        player.connected = false;
        self.bump_revision();
        true
    }

    /// Leaving an observer slot never changes a match's roster or readiness.
    pub(crate) fn remove_spectator(&mut self, connection: ConnectionId) -> bool {
        let Some(player) = self
            .spectators
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
        else {
            return false;
        };
        player.connected = false;
        player.left = true;
        self.last_requests.remove(&connection);
        self.bump_revision();
        true
    }

    /// Admission is only called after the old game has been discarded.
    pub(crate) fn admit_spectators(&mut self, capacity: u8) -> Vec<Delivery> {
        let mut changed = false;
        for index in 0..self.spectators.len() {
            if !self.spectators[index].connected || self.spectators[index].left {
                continue;
            }
            if self.players.iter().filter(|player| !player.left).count() >= usize::from(capacity) {
                break;
            }
            let Some(seat) = self.random_available_seat() else {
                break;
            };
            let vacant = self.players.iter().position(|player| player.left);
            let id = PlayerId(vacant.unwrap_or(self.players.len()) as u8);
            let mut player = self.spectators[index].clone();
            self.spectators[index].left = true;
            self.spectators[index].connected = false;
            player.id = id;
            player.seat = Some(seat);
            player.ready = false;
            if let Some(index) = vacant {
                self.players[index] = player;
            } else {
                self.players.push(player);
            }
            self.bump_revision();
            changed = true;
        }
        if !changed {
            return Vec::new();
        }
        self.active_participants()
            .map(|player| {
                self.delivery(
                    player.connection,
                    None,
                    ServerEvent::Joined { you: player.id },
                )
            })
            .collect()
    }
}
