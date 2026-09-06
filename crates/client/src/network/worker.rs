use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use leocard_protocol::{ClientCommand, ClientMessage, RequestId, ServerEvent};
use leocard_tcp::{TcpClient, TcpServerHandle};

use super::{
    CONNECTION_HEARTBEAT_TIMEOUT, CommandReceiver, EventQueue, NETWORK_ROOM_ID, NetworkEvent,
    NetworkLaunch, PING_INTERVAL, RECONNECT_ATTEMPTS, RECONNECT_CONNECT_TIMEOUT, push_event,
};

pub(super) async fn run_network(
    launch: NetworkLaunch,
    mut commands: CommandReceiver,
    events: &EventQueue,
    reconnect_join: ClientMessage,
) -> Result<(), String> {
    match launch {
        NetworkLaunch::Host { port, session } => {
            let bind_address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
            let server = TcpServerHandle::bind(bind_address, *session)
                .await
                .map_err(|error| format!("无法开放端口 {port}：{error}"))?;
            let local_address = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port);
            let client = TcpClient::connect(local_address)
                .await
                .map_err(|error| format!("房主无法回连本机端口 {port}：{error}"))?;
            let result = run_connected(
                client,
                &mut commands,
                events,
                format!("已监听端口 {port}；请分享 房主局域网IP:{port}"),
            )
            .await;
            let shutdown = server
                .shutdown()
                .await
                .map_err(|error| format!("关闭房间失败：{error}"));
            result.and(shutdown)
        }
        NetworkLaunch::Join { address } => {
            let client = TcpClient::connect(address.as_str())
                .await
                .map_err(|error| format!("连接 {address} 失败：{error}"))?;
            run_joined_network(client, address, &mut commands, events, &reconnect_join).await
        }
    }
}

async fn run_joined_network(
    mut client: TcpClient,
    address: String,
    commands: &mut CommandReceiver,
    events: &EventQueue,
    reconnect_join: &ClientMessage,
) -> Result<(), String> {
    loop {
        let disconnected =
            match run_connected(client, commands, events, format!("已连接 {address}")).await {
                Ok(()) => return Ok(()),
                Err(error) => error,
            };
        drain_pending_commands(commands);

        let mut last_error = disconnected;
        let mut reconnected = None;
        for attempt in 1..=RECONNECT_ATTEMPTS {
            let delay = reconnect_delay(attempt);
            let status = format!(
                "连接已断开；{} 秒后进行第 {attempt}/{RECONNECT_ATTEMPTS} 次重连",
                delay.as_secs()
            );
            push_event(events, NetworkEvent::Reconnecting(status));
            tokio::time::sleep(delay).await;

            match tokio::time::timeout(
                RECONNECT_CONNECT_TIMEOUT,
                TcpClient::connect(address.as_str()),
            )
            .await
            {
                Ok(Ok(replacement)) => {
                    match complete_reconnect_handshake(replacement, reconnect_join, events).await {
                        Ok(replacement) => {
                            reconnected = Some(replacement);
                            break;
                        }
                        Err(error) => {
                            last_error = format!("第 {attempt} 次重连握手失败：{error}");
                        }
                    }
                }
                Ok(Err(error)) => {
                    last_error = format!("第 {attempt} 次重连失败：{error}");
                }
                Err(_) => {
                    last_error = format!("第 {attempt} 次重连超时");
                }
            }
        }

        let Some(replacement) = reconnected else {
            return Err(format!(
                "已连续重连 {RECONNECT_ATTEMPTS} 次，仍无法连接房主：{last_error}"
            ));
        };
        drain_pending_commands(commands);
        client = replacement;
    }
}

async fn complete_reconnect_handshake(
    mut client: TcpClient,
    reconnect_join: &ClientMessage,
    events: &EventQueue,
) -> Result<TcpClient, String> {
    client
        .send(reconnect_join)
        .await
        .map_err(|error| error.to_string())?;
    let message = tokio::time::timeout(RECONNECT_CONNECT_TIMEOUT, client.receive())
        .await
        .map_err(|_| "等待房主确认超时".to_owned())?
        .map_err(|error| error.to_string())?;
    match &message.event {
        ServerEvent::Joined { .. } => {
            push_event(events, NetworkEvent::Message(Box::new(message)));
            Ok(client)
        }
        ServerEvent::Rejected { reason } => Err(format!("房主拒绝恢复身份：{reason:?}")),
        _ => Err("房主未确认恢复玩家身份".to_owned()),
    }
}

async fn run_connected(
    client: TcpClient,
    commands: &mut CommandReceiver,
    events: &EventQueue,
    connected_label: String,
) -> Result<(), String> {
    let (mut reader, mut writer) = client.into_split();
    push_event(events, NetworkEvent::Connected(connected_label));
    let heartbeat_timeout = tokio::time::sleep(CONNECTION_HEARTBEAT_TIMEOUT);
    tokio::pin!(heartbeat_timeout);
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = &mut heartbeat_timeout => {
                return Err("与房主的连接心跳超时".to_owned());
            }
            message = reader.receive() => {
                let message = message.map_err(|error| format!("与房间的连接已断开：{error}"))?;
                heartbeat_timeout.as_mut().reset(
                    tokio::time::Instant::now() + CONNECTION_HEARTBEAT_TIMEOUT
                );
                if matches!(&message.event, ServerEvent::Heartbeat) {
                    continue;
                }
                let connection_finished = matches!(
                    &message.event,
                    ServerEvent::RoomClosed | ServerEvent::LeftRoom
                );
                push_event(events, NetworkEvent::Message(Box::new(message)));
                if connection_finished {
                    return Ok(());
                }
            }
            command = commands.recv() => {
                let Some(command) = command else { return Ok(()) };
                writer.send(&command).await
                    .map_err(|error| format!("发送游戏指令失败：{error}"))?;
            }
            _ = ping_interval.tick() => {
                let ping = ClientMessage::new(
                    NETWORK_ROOM_ID,
                    RequestId(0),
                    ClientCommand::Ping,
                );
                writer.send(&ping).await
                    .map_err(|error| format!("发送连接心跳失败：{error}"))?;
            }
        }
    }
}

fn drain_pending_commands(commands: &mut CommandReceiver) {
    while commands.try_recv().is_ok() {}
}

fn reconnect_delay(attempt: u8) -> Duration {
    match attempt {
        1 => Duration::ZERO,
        2 => Duration::from_secs(1),
        3 => Duration::from_secs(2),
        4 => Duration::from_secs(4),
        5 => Duration::from_secs(8),
        _ => Duration::from_secs(15),
    }
}
