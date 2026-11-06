use crate::{ConnectionId, Delivery};
use leocard_protocol::{GameCommand, GameKind, RequestId};

/// Route protocol commands once, then handle this game's concrete command type.
pub(crate) trait GameCommandHandler {
    type Command;

    fn parse_command(command: GameCommand) -> Result<Self::Command, GameKind>;

    fn handle_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: Self::Command,
    ) -> Vec<Delivery>;
}
