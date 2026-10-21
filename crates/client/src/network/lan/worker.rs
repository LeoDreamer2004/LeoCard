use super::browser::{BrowserSnapshot, LanRoom};
use leocard_protocol::{AvatarId, RoomInfo};
use leocard_tcp::{LanDiscoverySocket, LanRoomAnnouncement, query_room_info};
use std::collections::HashMap;
use std::io;
use std::time::Duration;
use tokio::sync::{oneshot, watch};
use tokio::task::JoinSet;
use tokio::time::{Instant, MissedTickBehavior, interval};

const MAX_ROOMS: usize = 64;
const ROOM_LIFETIME: Duration = Duration::from_secs(4);

struct RoomEntry {
    room: LanRoom,
    last_seen: Instant,
    last_query: Option<Instant>,
}

pub(super) async fn browse(
    sender: &watch::Sender<BrowserSnapshot>,
    mut stopped: oneshot::Receiver<()>,
) -> io::Result<()> {
    let socket = LanDiscoverySocket::bind()?;
    let mut rooms = HashMap::<u64, RoomEntry>::new();
    let mut queries = JoinSet::new();
    let mut cleanup = interval(Duration::from_secs(1));
    cleanup.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        let changed = tokio::select! {
            _ = &mut stopped => return Ok(()),
            notice = socket.receive() => {
                let notice = notice?;
                let id = notice.server_id;
                let new = !rooms.contains_key(&id);
                if new && rooms.len() >= MAX_ROOMS { continue; }
                let entry = rooms.entry(id).or_insert_with(|| RoomEntry {
                    room: LanRoom { server_id: id, address: notice.address, info: notice.info.clone() },
                    last_seen: Instant::now(), last_query: None,
                });
                let changed = update_room(entry, &notice);
                if entry.room.info.host_avatar.is_some() && entry.room.info.avatar_png.is_none()
                    && queries.len() < 4 && entry.last_query.is_none_or(|last| last.elapsed() >= Duration::from_secs(3))
                {
                    entry.last_query = Some(Instant::now());
                    let address = entry.room.address;
                    let room_id = notice.room_id;
                    let avatar = entry.room.info.host_avatar;
                    queries.spawn(async move { (id, avatar, query_room_info(address, room_id).await.ok()) });
                }
                // Also publish a newly discovered entry even when its metadata matches the seed.
                changed || new
            }
            completed = queries.join_next(), if !queries.is_empty() => {
                match completed {
                    Some(Ok((id, avatar, Some(info)))) => apply_avatar(&mut rooms, id, avatar, info),
                    _ => false,
                }
            }
            _ = cleanup.tick() => {
                let before = rooms.len();
                rooms.retain(|_, room| room.last_seen.elapsed() < ROOM_LIFETIME);
                rooms.len() != before
            }
        };
        if changed {
            let mut visible: Vec<_> = rooms.values().map(|entry| entry.room.clone()).collect();
            visible.sort_by(|left, right| {
                left.info
                    .host_name
                    .cmp(&right.info.host_name)
                    .then(left.server_id.cmp(&right.server_id))
            });
            sender.send_replace(BrowserSnapshot {
                rooms: visible,
                error: None,
            });
        }
    }
}

fn update_room(entry: &mut RoomEntry, notice: &LanRoomAnnouncement) -> bool {
    let mut info = notice.info.clone();
    if entry.room.info.host_avatar == info.host_avatar {
        info.avatar_png.clone_from(&entry.room.info.avatar_png);
    } else {
        entry.last_query = None;
    }
    // The same room may advertise through both loopback and a LAN interface.
    let address = if !entry.room.address.ip().is_loopback() && notice.address.ip().is_loopback() {
        entry.room.address
    } else {
        notice.address
    };
    let changed = entry.room.info != info || entry.room.address != address;
    entry.room.info = info;
    entry.room.address = address;
    entry.last_seen = Instant::now();
    changed
}

fn apply_avatar(
    rooms: &mut HashMap<u64, RoomEntry>,
    id: u64,
    expected: Option<AvatarId>,
    info: RoomInfo,
) -> bool {
    let Some(entry) = rooms.get_mut(&id) else {
        return false;
    };
    if entry.room.info.host_avatar != expected
        || info.host_avatar != expected
        || entry.room.info.avatar_png == info.avatar_png
    {
        return false;
    }
    entry.room.info.avatar_png = info.avatar_png;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_protocol::{GameKind, RoomId};

    #[test]
    fn repeated_announcements_preserve_avatar_and_prefer_the_lan_address() {
        let mut notice = LanRoomAnnouncement {
            server_id: 7,
            room_id: RoomId(9),
            address: "192.168.1.2:54321".parse().unwrap(),
            info: RoomInfo {
                host_name: "房主".to_owned(),
                host_avatar: Some(AvatarId(1)),
                avatar_png: None,
                game: GameKind::Uno,
                playing: false,
            },
        };
        let mut entry = RoomEntry {
            room: LanRoom {
                server_id: notice.server_id,
                address: notice.address,
                info: notice.info.clone(),
            },
            last_seen: Instant::now(),
            last_query: Some(Instant::now()),
        };
        entry.room.info.avatar_png = Some(vec![1]);
        assert!(!update_room(&mut entry, &notice));
        notice.address = "127.0.0.1:54321".parse().unwrap();
        assert!(!update_room(&mut entry, &notice));
        assert!(!entry.room.address.ip().is_loopback());
        assert_eq!(entry.room.info.avatar_png, Some(vec![1]));
        notice.info.host_avatar = Some(AvatarId(2));
        notice.info.playing = true;
        assert!(update_room(&mut entry, &notice));
        assert_eq!(entry.room.info.avatar_png, None);
        assert!(entry.last_query.is_none());
        let mut rooms = HashMap::from([(notice.server_id, entry)]);
        let mut stale = notice.info;
        stale.host_avatar = Some(AvatarId(1));
        stale.avatar_png = Some(vec![1]);
        assert!(!apply_avatar(
            &mut rooms,
            notice.server_id,
            stale.host_avatar,
            stale
        ));
    }
}
