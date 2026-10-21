use super::{Participant, RoomSession};
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    JoinRequest, MAX_PLAYER_NAME_CHARS, PlayerId, PlayerViolation, RejectReason, RequestId,
    RoomViolation, ServerEvent,
};

impl RoomSession {
    pub(crate) fn join(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        request: JoinRequest,
        game_started: bool,
        capacity: u8,
    ) -> Result<Vec<Delivery>, RejectReason> {
        if self.player_id(connection).is_some() {
            return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
        }
        let normalized_name = request.name.trim();
        if normalized_name.is_empty() {
            return Err(RejectReason::Player(PlayerViolation::NameEmpty));
        }
        if normalized_name.chars().count() > MAX_PLAYER_NAME_CHARS {
            return Err(RejectReason::Player(PlayerViolation::NameTooLong {
                max_chars: MAX_PLAYER_NAME_CHARS as u16,
            }));
        }
        if !crate::valid_identity_proof(self.room_id, &request) {
            return Err(RejectReason::Player(PlayerViolation::InvalidIdentityProof));
        }
        let validated_request = request.clone();
        let JoinRequest {
            name,
            reconnect_token,
            profile_id,
            reference_points,
            completed_games,
            game_profiles,
            identity_signature: _,
        } = request;
        let name = name.trim();
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.reconnect_token == reconnect_token && !player.left)
        {
            if self.players[index].name != name || self.players[index].profile_id != profile_id {
                return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
            }
            return Ok(self.reconnect(index, connection, request_id, game_started));
        }
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.profile_id == profile_id && !player.left)
        {
            if self.players[index].connected {
                return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
            }
            self.players[index].reconnect_token = reconnect_token;
            self.last_requests.remove(&self.players[index].connection);
            return Ok(self.reconnect(index, connection, request_id, game_started));
        }
        if let Some(index) = self
            .spectators
            .iter()
            .position(|player| player.profile_id == profile_id && !player.left)
        {
            let previous = &mut self.spectators[index];
            if previous.reconnect_token != reconnect_token && previous.connected {
                return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
            }
            let old_connection = previous.connection;
            previous.connection = connection;
            previous.reconnect_token = reconnect_token;
            previous.connected = true;
            let player = previous.id;
            self.last_requests.remove(&old_connection);
            self.bump_revision();
            return Ok(self.join_deliveries(connection, request_id, player));
        }
        if game_started {
            return self.join_spectator(connection, request_id, validated_request);
        }
        if self.players.iter().filter(|player| !player.left).count() >= usize::from(capacity) {
            return Err(RejectReason::Room(RoomViolation::RoomFull));
        }

        let vacant = self.players.iter().position(|player| player.left);
        let player = PlayerId(
            vacant
                .unwrap_or(self.players.len())
                .try_into()
                .expect("a room contains at most six active player slots"),
        );
        let joining_as_host = self.host_connection.is_none();
        if joining_as_host {
            self.host_connection = Some(connection);
        }
        let seat = self
            .random_available_seat()
            .expect("a non-full room always has an available seat");
        let participant = Participant {
            id: player,
            profile_id,
            connection,
            name: name.to_owned(),
            avatar: None,
            avatar_png: None,
            reconnect_token,
            seat: Some(seat),
            ready: joining_as_host,
            connected: true,
            auto_play: false,
            is_bot: false,
            left: false,
            reference_points,
            completed_games,
            game_profiles,
        };
        if let Some(index) = vacant {
            self.players[index] = participant;
        } else {
            self.players.push(participant);
        }
        self.bump_revision();
        Ok(self.join_deliveries(connection, request_id, player))
    }

    fn reconnect(
        &mut self,
        index: usize,
        connection: ConnectionId,
        request_id: RequestId,
        game_started: bool,
    ) -> Vec<Delivery> {
        let previous_connection = self.players[index].connection;
        let player = self.players[index].id;
        let reassigned_seat = (!game_started && self.players[index].seat.is_none())
            .then(|| self.random_available_seat())
            .flatten();
        self.players[index].connection = connection;
        self.players[index].connected = true;
        if let Some(seat) = reassigned_seat {
            self.players[index].seat = Some(seat);
        }
        if self.host_connection == Some(previous_connection) {
            self.host_connection = Some(connection);
        }
        let previous_last_request = self.last_requests.remove(&previous_connection);
        if let Some(last_request) = self.last_requests.get_mut(&connection) {
            if let Some(previous_last_request) = previous_last_request {
                *last_request = (*last_request).max(previous_last_request);
            }
        } else if let Some(previous_last_request) = previous_last_request {
            self.last_requests.insert(connection, previous_last_request);
        }
        self.bump_revision();
        self.join_deliveries(connection, request_id, player)
    }

    pub(super) fn join_deliveries(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        player: PlayerId,
    ) -> Vec<Delivery> {
        let mut deliveries = vec![self.delivery(
            connection,
            Some(request_id),
            ServerEvent::Joined { you: player },
        )];
        deliveries.extend(
            self.participants()
                .filter(|player| !player.left)
                .filter_map(|participant| {
                    Some(self.delivery(
                        connection,
                        None,
                        ServerEvent::AvatarData {
                            id: participant.avatar?,
                            png: participant.avatar_png.clone()?,
                        },
                    ))
                }),
        );
        deliveries
    }
}
