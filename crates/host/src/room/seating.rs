use super::RoomSession;
use crate::ConnectionId;
use leocard_protocol::{
    GameViolation, PlayerViolation, ReconnectToken, RejectReason, RoomViolation, SeatId,
};

impl RoomSession {
    /// Keep valid seats and move players from removed positions into the remaining vacancies.
    /// The caller owns the ready-state reset, revision and lobby broadcast.
    pub(crate) fn resize_seats(&mut self, seat_count: u8) -> Result<(), RejectReason> {
        if seat_count == 0 {
            return Err(RejectReason::Room(RoomViolation::InvalidSeat));
        }
        if self.players.iter().filter(|player| !player.left).count() > usize::from(seat_count) {
            return Err(RejectReason::Room(RoomViolation::RoomFull));
        }
        let mut occupied = vec![false; usize::from(seat_count)];
        for player in self.players.iter_mut().filter(|player| !player.left) {
            if let Some(seat) = player.seat
                && let Some(used) = occupied.get_mut(usize::from(seat.0))
                && !*used
            {
                *used = true;
            } else {
                player.seat = None;
            }
        }
        let mut available = occupied
            .iter()
            .enumerate()
            .filter_map(|(index, used)| (!used).then_some(SeatId(index as u8)));
        for player in self.players.iter_mut().filter(|player| !player.left) {
            let seat = *player
                .seat
                .get_or_insert_with(|| available.next().expect("player count fits the new seats"));
            if player.is_bot {
                // Bot connections encode the seat; relocating one must not collide with a later bot.
                let token = u64::MAX - u64::from(seat.0);
                player.connection = ConnectionId(token);
                player.reconnect_token = ReconnectToken(token);
            }
        }
        self.seat_count = seat_count;
        Ok(())
    }

    pub(crate) fn select_seat(
        &mut self,
        connection: ConnectionId,
        seat: SeatId,
        game_started: bool,
    ) -> Result<(), RejectReason> {
        let player = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if seat.0 >= self.seat_count {
            return Err(RejectReason::Room(RoomViolation::InvalidSeat));
        }
        if self.players.iter().any(|participant| {
            !participant.left && participant.id != player && participant.seat == Some(seat)
        }) {
            return Err(RejectReason::Room(RoomViolation::SeatTaken));
        }
        let is_host = self.host_connection == Some(connection);
        let participant = &mut self.players[usize::from(player.0)];
        if participant.seat != Some(seat) {
            participant.seat = Some(seat);
            participant.ready = is_host;
            self.bump_revision();
        }
        Ok(())
    }

    pub(super) fn random_available_seat(&self) -> Option<SeatId> {
        let mut available = (0..self.seat_count)
            .map(SeatId)
            .filter(|seat| {
                !self
                    .players
                    .iter()
                    .any(|player| !player.left && player.seat == Some(*seat))
            })
            .collect::<Vec<_>>();
        fastrand::shuffle(&mut available);
        available.pop()
    }
}
