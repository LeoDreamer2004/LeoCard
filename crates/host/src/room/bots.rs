use super::{Participant, RoomSession};
use crate::ConnectionId;
use leocard_protocol::{
    GameViolation, PlayerGameProfiles, PlayerId, PlayerViolation, ProfileId, ReconnectToken,
    RejectReason, RoomViolation, SeatId,
};

impl RoomSession {
    pub(crate) fn configure_bot_seat(
        &mut self,
        connection: ConnectionId,
        seat: SeatId,
        occupied: bool,
        game_started: bool,
    ) -> Result<(), RejectReason> {
        self.player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
        }
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if seat.0 >= self.seat_count {
            return Err(RejectReason::Room(RoomViolation::InvalidSeat));
        }
        let occupant = self
            .players
            .iter()
            .position(|player| !player.left && player.seat == Some(seat));
        if let Some(index) = occupant {
            if !self.players[index].is_bot {
                return Err(RejectReason::Room(RoomViolation::SeatTaken));
            }
            if occupied {
                return Ok(());
            }
            let player = &mut self.players[index];
            player.seat = None;
            player.ready = false;
            player.connected = false;
            player.auto_play = false;
            player.left = true;
            self.bump_revision();
            return Ok(());
        }
        if !occupied {
            return Ok(());
        }

        let ordinal = (1..=self.seat_count)
            .find(|ordinal| {
                let name = format!("机器人{ordinal}");
                self.players
                    .iter()
                    .all(|player| player.left || player.name != name)
            })
            .expect("there are at most six robot seats");
        let vacant = self.players.iter().position(|player| player.left);
        let id = PlayerId(vacant.unwrap_or(self.players.len()) as u8);
        let participant = Participant {
            id,
            profile_id: ProfileId([0; 32]),
            connection: ConnectionId(u64::MAX - u64::from(seat.0)),
            name: format!("机器人{ordinal}"),
            avatar: None,
            avatar_png: None,
            reconnect_token: ReconnectToken(u64::MAX - u64::from(seat.0)),
            seat: Some(seat),
            ready: true,
            connected: false,
            auto_play: true,
            is_bot: true,
            left: false,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
        };
        if let Some(index) = vacant {
            self.players[index] = participant;
        } else {
            self.players.push(participant);
        }
        self.bump_revision();
        Ok(())
    }

    pub(crate) fn remove_bots(&mut self) {
        for player in self.players.iter_mut().filter(|player| player.is_bot) {
            player.seat = None;
            player.ready = false;
            player.connected = false;
            player.auto_play = false;
            player.left = true;
        }
    }
}
