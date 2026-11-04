use crate::app::presentation::CornerNotifications;
use bevy::prelude::*;
use leocard_client::PlayerEconomy;

pub(super) fn queue_coin_notifications(
    mut economy: ResMut<PlayerEconomy>,
    mut notices: ResMut<CornerNotifications>,
) {
    for delta in economy.take_coin_changes() {
        notices.coin(delta);
    }
}
