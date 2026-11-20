//! Headless room for real browser/FRP transport checks. No player data is loaded.
//! Usage: cargo run -p leocard-tcp --example socket_probe -- ADDRESS OUTPUT [CERT KEY]
//! OUTPUT contains signed test frames, usable with the browser WebSocket API.

use ed25519_dalek::{Signer, SigningKey};
use leocard_host::{GameSetup, HostSession};
use leocard_protocol::{
    ChatContent, ClientCommand, ClientMessage, GameRules, JoinRequest, ProfileId, ReconnectToken,
    RequestId, RoomId, SeatId, ServerEvent, encode_frame,
};
use leocard_tcp::{ServerTls, TcpClient, TcpServerHandle};
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{Instant, interval, sleep_until};

const ROOM: RoomId = RoomId(523);

fn command(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

fn join(seed: u8) -> ClientMessage {
    let key = SigningKey::from_bytes(&[seed; 32]);
    let mut request = JoinRequest {
        name: format!("Probe{seed}"),
        reconnect_token: ReconnectToken(u64::from(seed)),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: Default::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(ROOM))
        .to_bytes()
        .to_vec();
    command(1, ClientCommand::join(request))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 && args.len() != 4 {
        return Err(io::Error::other("usage: socket_probe ADDRESS OUTPUT [CERT KEY]").into());
    }
    let address: SocketAddr = args[0].parse()?;
    let output = PathBuf::from(&args[1]);
    let session = HostSession::new(
        ROOM,
        GameSetup::shuffled(address.port(), GameRules::Uno(Default::default())),
    )?;
    let server = if args.len() == 4 {
        let certificate = fs::read(&args[2])?;
        let key = fs::read(&args[3])?;
        TcpServerHandle::bind_with_tls(address, session, ServerTls::from_pem(&certificate, &key)?)
            .await?
    } else {
        TcpServerHandle::bind(address, session).await?
    };
    let local = SocketAddr::new("127.0.0.1".parse()?, server.local_addr().port());
    let mut host = TcpClient::connect(local).await?;
    host.send(&join(1)).await?;
    loop {
        if matches!(host.receive().await?.event, ServerEvent::LobbySnapshot(_)) {
            break;
        }
    }
    host.send(&command(
        2,
        ClientCommand::ConfigureBotSeat {
            seat: SeatId(1),
            occupied: true,
        },
    ))
    .await?;
    host.send(&command(3, ClientCommand::StartGame)).await?;
    let host_id = loop {
        match host.receive().await?.event {
            ServerEvent::GameSnapshot(game) => break game.you(),
            ServerEvent::Rejected { reason } => {
                return Err(io::Error::other(reason.to_string()).into());
            }
            _ => {}
        }
    };
    let peers = (2..=7).map(|seed| {
        let text = format!("browser-probe-{seed}");
        Ok(json!({
            "name": format!("Probe{seed}"),
            "text": text,
            "reply": format!("tcp-ack-{text}"),
            "join": encode_frame(&join(seed))?,
            "chat": encode_frame(&command(2, ClientCommand::Chat { content: ChatContent::Text(text) }))?,
            "leave": encode_frame(&command(3, ClientCommand::LeaveRoom))?,
        }))
    }).collect::<Result<Vec<_>, leocard_protocol::FrameError>>()?;
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("frames.json"),
        serde_json::to_vec_pretty(&json!({
            "peers": peers,
            "ping": encode_frame(&command(0, ClientCommand::Ping))?,
        }))?,
    )?;
    println!(
        "READY TCP/WS{} {}",
        if args.len() == 4 { "/WSS" } else { "" },
        server.local_addr()
    );
    println!("FRAMES {}", output.join("frames.json").display());
    let mut ping = interval(Duration::from_secs(3));
    let end = Instant::now() + Duration::from_secs(600);
    let mut request = 10;
    loop {
        tokio::select! {
            _ = sleep_until(end) => break,
            _ = ping.tick() => host.send(&command(0, ClientCommand::Ping)).await?,
            message = host.receive() => match message?.event {
                ServerEvent::ChatMessage(message) => {
                    if let ChatContent::Text(text) = message.content {
                        println!("CHAT {} {text}", message.source.0);
                        if message.source != host_id {
                            request += 1;
                            host.send(&command(request, ClientCommand::Chat {
                                content: ChatContent::Text(format!("tcp-ack-{text}")),
                            })).await?;
                        }
                    }
                }
                ServerEvent::Rejected { reason } => eprintln!("REJECTED {reason}"),
                ServerEvent::RoomClosed => break,
                _ => {}
            }
        }
    }
    server.shutdown().await?;
    Ok(())
}
