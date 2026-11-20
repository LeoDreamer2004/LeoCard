use super::handshake::{Handshake, validate_message};
use super::state::{
    CONNECTION_HEARTBEAT_TIMEOUT, CommandReceiver, EventQueue, NETWORK_ROOM_ID, NetworkEvent,
    PING_INTERVAL, push_event,
};
use leocard_protocol::{ClientCommand, ClientMessage, RequestId, ServerEvent};
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
        let mut handshake = Handshake::default();
        loop {
            let message = client.receive().await.map_err(|error| error.to_string())?;
            validate_message(&message)?;
            let ready = handshake.accept(&message)?;
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
                let finished = matches!(&message.event, ServerEvent::RoomClosed | ServerEvent::LeftRoom | ServerEvent::KickedFromRoom);
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

fn send_error(error: TcpError) -> String {
    format!("发送游戏指令失败：{error}")
}
