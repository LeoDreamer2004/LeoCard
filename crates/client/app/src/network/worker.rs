use super::connection::{complete_handshake, run_connected};
use super::state::{
    CommandReceiver, EventQueue, NetworkEvent, NetworkLaunch, RECONNECT_ATTEMPTS,
    RECONNECT_CONNECT_TIMEOUT, push_event,
};
use leocard_protocol::ClientMessage;
use leocard_tcp::{TcpClient, TcpServerHandle};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

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
            let result = async {
                let client = complete_handshake(client, &reconnect_join, events).await?;
                push_event(
                    events,
                    NetworkEvent::Connected(format!(
                        "已监听端口 {port}；请分享 房主局域网IP:{port}"
                    )),
                );
                run_connected(client, &mut commands, events).await
            }
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
            let client = complete_handshake(client, &reconnect_join, events).await?;
            push_event(events, NetworkEvent::Connected(format!("已连接 {address}")));
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
    let mut failures = 0;
    loop {
        let connected_at = Instant::now();
        let disconnected = match run_connected(client, commands, events).await {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };
        if connected_at.elapsed() >= Duration::from_secs(10) {
            failures = 0;
        }
        drain_pending_commands(commands);
        let reconnect = async {
            let mut last_error = disconnected;
            let mut reconnected = None;
            while failures < RECONNECT_ATTEMPTS {
                failures += 1;
                let attempt = failures;
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
                        match complete_handshake(replacement, reconnect_join, events).await {
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
            Ok(replacement)
        };
        tokio::pin!(reconnect);
        let replacement = loop {
            tokio::select! {
                biased;
                command = commands.recv() => {
                    if command.is_none() {
                        return Ok(());
                    }
                }
                result = &mut reconnect => break result?,
            }
        };
        drain_pending_commands(commands);
        push_event(events, NetworkEvent::Connected(format!("已连接 {address}")));
        client = replacement;
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
