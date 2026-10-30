use super::state::{
    CONNECTION_HEARTBEAT_TIMEOUT, CommandReceiver, EventQueue, NETWORK_ROOM_ID, NetworkEvent,
    PING_INTERVAL, push_event,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, PROTOCOL_VERSION, RequestId, ServerEvent, ServerMessage,
};
use leocard_tcp::{TcpClient, TcpError};
use tokio::time::{Instant, MissedTickBehavior, interval, sleep, timeout};

/// Transport connectivity alone is insufficient: wait for identity and authoritative state.
pub(super) async fn complete_handshake(
    mut client: TcpClient,
    join: &ClientMessage,
    events: &EventQueue,
) -> Result<TcpClient, String> {
    client.send(join).await.map_err(|error| error.to_string())?;
    timeout(CONNECTION_HEARTBEAT_TIMEOUT, async {
        let mut joined = None;
        let mut rules = None;
        loop {
            let message = client.receive().await.map_err(|error| error.to_string())?;
            validate_message(&message)?;
            let ready = match &message.event {
                ServerEvent::Heartbeat => continue,
                ServerEvent::Joined { you } => {
                    joined = Some(*you);
                    false
                }
                ServerEvent::RulesSnapshot(snapshot) => {
                    rules = Some(snapshot.kind());
                    false
                }
                ServerEvent::Rejected { reason } => return Err(reason.to_string()),
                ServerEvent::RoomClosed | ServerEvent::LeftRoom => {
                    return Err("房间已关闭".to_owned());
                }
                ServerEvent::LobbySnapshot(snapshot) => joined.is_some() && snapshot.host.is_some(),
                ServerEvent::GameSnapshot(snapshot) => {
                    // A spectator's identity differs from the player whose view is streamed.
                    joined.is_some() && rules == Some(snapshot.kind())
                }
                _ => false,
            };
            push_event(events, NetworkEvent::Message(Box::new(message)));
            if ready {
                return Ok(client);
            }
        }
    })
    .await
    .map_err(|_| "等待房主同步完整状态超时".to_owned())?
}

pub(super) async fn run_connected(
    client: TcpClient,
    commands: &mut CommandReceiver,
    events: &EventQueue,
) -> Result<(), String> {
    let (mut reader, mut writer) = client.into_split();
    let heartbeat_timeout = sleep(CONNECTION_HEARTBEAT_TIMEOUT);
    tokio::pin!(heartbeat_timeout);
    let mut ping_interval = interval(PING_INTERVAL);
    ping_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = &mut heartbeat_timeout => return Err("与房主的连接心跳超时".to_owned()),
            message = reader.receive() => {
                let message = message.map_err(|error| format!("与房间的连接已断开：{error}"))?;
                validate_message(&message)?;
                heartbeat_timeout.as_mut().reset(Instant::now() + CONNECTION_HEARTBEAT_TIMEOUT);
                if matches!(&message.event, ServerEvent::Heartbeat) {
                    continue;
                }
                let finished = matches!(&message.event, ServerEvent::RoomClosed | ServerEvent::LeftRoom);
                push_event(events, NetworkEvent::Message(Box::new(message)));
                if finished {
                    return Ok(());
                }
            }
            command = commands.recv() => {
                let Some(command) = command else { return Ok(()) };
                writer.send(&command).await.map_err(send_error)?;
            }
            _ = ping_interval.tick() => {
                writer.send(&ClientMessage::new(NETWORK_ROOM_ID, RequestId(0), ClientCommand::Ping))
                    .await.map_err(send_error)?;
            }
        }
    }
}

fn validate_message(message: &ServerMessage) -> Result<(), String> {
    if message.protocol_version != PROTOCOL_VERSION {
        return Err("与房主的协议版本不一致，请使用相同版本的游戏".to_owned());
    }
    if message.room_id != NETWORK_ROOM_ID {
        return Err("房主返回了错误的房间编号".to_owned());
    }
    Ok(())
}

fn send_error(error: TcpError) -> String {
    format!("发送游戏指令失败：{error}")
}
