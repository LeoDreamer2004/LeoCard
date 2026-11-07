use bevy::prelude::*;
use leocard_client::{NetworkState, PlayerEconomy, PropId, TcpGameClient};
use leocard_game_common::PlayerId;
use leocard_protocol::{ClientCommand, PlayerInteractionKind, ProfileId, RoomId};
use std::time::Duration;

const BUSY_DURATION: Duration = Duration::from_secs(16);

#[derive(Component)]
pub(crate) struct EggThrowerCount;

#[derive(Resource, Default)]
pub(crate) struct AutoEggThrower {
    pub(super) active: Option<EggBurst>,
}

pub(super) struct EggBurst {
    pub room: RoomId,
    pub source: PlayerId,
    pub target: PlayerId,
    pub target_profile: ProfileId,
    pub started_at: Duration,
}

impl AutoEggThrower {
    pub(crate) fn activate(
        &mut self,
        client: &mut TcpGameClient,
        target: PlayerId,
        now: Duration,
        economy: &mut PlayerEconomy,
    ) -> Result<(), String> {
        if self.active.is_some() {
            return Err("自动砸蛋器正在连发，请稍后再使用".to_owned());
        }
        if !matches!(client.state(), NetworkState::Connected(_)) {
            return Err("尚未连接房间，无法使用自动砸蛋器".to_owned());
        }
        let model = client.model();
        if !model.can_send_interactions() {
            return Err("房主已禁止旁观玩家互动".to_owned());
        }
        let source = model.you().ok_or("尚未加入房间")?;
        if source == target {
            return Err("不能对自己使用自动砸蛋器".to_owned());
        }
        let target_profile = model.player(target).ok_or("目标玩家已离开房间")?.profile_id;
        let active = EggBurst {
            room: model.room_id(),
            source,
            target,
            target_profile,
            started_at: now,
        };
        economy.consume_prop(PropId::AutoEggThrower)?;
        if !client.send(ClientCommand::Interact {
            target,
            kind: PlayerInteractionKind::EggBarrage,
        }) {
            return Err("无法发送自动砸蛋消息".to_owned());
        }
        self.active = Some(active);
        Ok(())
    }
}

impl EggBurst {
    pub(super) fn finished(&self, now: Duration) -> bool {
        now.saturating_sub(self.started_at) >= BUSY_DURATION
    }

    pub(super) fn still_valid(&self, client: &TcpGameClient) -> bool {
        let model = client.model();
        matches!(client.state(), NetworkState::Connected(_))
            && model.room_id() == self.room
            && model.you() == Some(self.source)
            && model
                .player(self.target)
                .is_some_and(|player| player.profile_id == self.target_profile)
    }
}
