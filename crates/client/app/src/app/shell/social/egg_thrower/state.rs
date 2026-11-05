use bevy::prelude::*;
use leocard_client::{NetworkState, PlayerEconomy, PropId, TcpGameClient};
use leocard_game_common::PlayerId;
use leocard_protocol::{ProfileId, RoomId};
use std::time::Duration;

const EGG_COUNT: u16 = 100;
const EGG_INTERVAL: Duration = Duration::from_millis(100);

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
    pub last_sent_at: Duration,
    pub sent: u16,
}

impl AutoEggThrower {
    pub(crate) fn activate(
        &mut self,
        client: &TcpGameClient,
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
        let source = model.you().ok_or("尚未加入房间")?;
        if source == target {
            return Err("不能对自己使用自动砸蛋器".to_owned());
        }
        let target_profile = model.player(target).ok_or("目标玩家已离开房间")?.profile_id;
        economy.consume_prop(PropId::AutoEggThrower)?;
        self.active = Some(EggBurst {
            room: model.room_id(),
            source,
            target,
            target_profile,
            last_sent_at: now,
            sent: 0,
        });
        Ok(())
    }
}

impl EggBurst {
    pub(super) fn ready(&self, now: Duration) -> bool {
        !self.finished() && now.saturating_sub(self.last_sent_at) >= EGG_INTERVAL
    }

    pub(super) fn record_sent(&mut self, now: Duration) {
        self.sent += 1;
        self.last_sent_at = now;
    }

    pub(super) fn finished(&self) -> bool {
        self.sent == EGG_COUNT
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
