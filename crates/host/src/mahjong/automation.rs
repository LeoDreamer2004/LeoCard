use super::*;

impl MahjongSession {
    pub(super) fn snapshot(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    pub(super) fn current_automatic_player(&self) -> Option<PlayerId> {
        let game = self.game.as_ref()?;
        let automatic = |player: MahjongPlayerId| {
            let player = from_core_player(player);
            self.room
                .players
                .iter()
                .find(|participant| participant.id == player && !participant.left)
                .is_some_and(|participant| participant.is_bot || participant.auto_play)
                .then_some(player)
        };
        match game.phase() {
            Phase::Playing => automatic(game.current_player()),
            Phase::WaitingForClaims(pending) => {
                pending.waiting_for().into_iter().find_map(automatic)
            }
            Phase::Dealing { .. } | Phase::ReplacingFlower { .. } | Phase::Finished(_) => None,
        }
    }

    pub(super) fn play_automatic_action(
        &mut self,
        player: PlayerId,
    ) -> Option<(Option<MahjongEvent>, ActionOutcome)> {
        let core_player = to_core_player(player);
        let action = automatic_mahjong_action(self.game.as_ref()?, core_player)?;
        let game = self.game.as_mut()?;
        let public_event = match action {
            AutomaticMahjongAction::Discard(tile) => {
                Some(MahjongEvent::TileDiscarded { player, tile })
            }
            _ => None,
        };
        let outcome = match action {
            AutomaticMahjongAction::Discard(tile) => game.discard(core_player, tile),
            AutomaticMahjongAction::Respond(claim) => game.respond_to_claim(core_player, claim),
            AutomaticMahjongAction::SelfDraw => game.declare_self_draw(core_player),
            AutomaticMahjongAction::ConcealedKong(tile) => {
                game.declare_concealed_kong(core_player, tile)
            }
            AutomaticMahjongAction::AddedKong(tile) => game.declare_added_kong(core_player, tile),
        }
        .ok()?;
        Some((public_event, outcome))
    }
}
