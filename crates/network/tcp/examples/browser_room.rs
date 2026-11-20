//! Native test host for the browser client. Uses an isolated, public test identity.
//! Usage: browser_room ADDRESS GAME [spectate] [AVATAR_PNG]

use ed25519_dalek::{Signer, SigningKey};
use leocard_host::{GameSetup, HostSession};
use leocard_protocol::{
    ChatContent, ClientCommand, ClientMessage, GameRules, JoinRequest, ProfileId, ReconnectToken,
    RequestId, RoomId, SeatId, ServerEvent,
};
use leocard_tcp::{TcpClient, TcpServerHandle};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::{Instant, interval};

const ROOM: RoomId = RoomId(523);

fn rules(name: &str) -> Result<GameRules, io::Error> {
    Ok(match name {
        "uno" => GameRules::Uno(Default::default()),
        "mahjong" => GameRules::Mahjong(Default::default()),
        "texas" => GameRules::Texas(Default::default()),
        "shengji" => GameRules::Shengji(Default::default()),
        "qigui523" => GameRules::QiGui523(Default::default()),
        "doudizhu" => GameRules::Doudizhu(Default::default()),
        "guandan" => GameRules::Guandan(Default::default()),
        "blackjack" => GameRules::Blackjack(Default::default()),
        "mahjong-link" => GameRules::MahjongLink(Default::default()),
        "mc-guess" => GameRules::McGuess(Default::default()),
        "draw-guess" => GameRules::DrawGuess(Default::default()),
        _ => return Err(io::Error::other("unknown game")),
    })
}

fn player_count(rules: &GameRules) -> u8 {
    match rules {
        GameRules::QiGui523(rules) => rules.player_count,
        GameRules::Texas(rules) => rules.player_count,
        GameRules::Shengji(rules) => rules.player_count(),
        GameRules::Doudizhu(rules) => rules.mode.players(),
        GameRules::Mahjong(_) | GameRules::Guandan(_) => 4,
        _ => 2,
    }
}

fn join(avatar_png: Option<Vec<u8>>) -> ClientCommand {
    let key = SigningKey::from_bytes(&[113; 32]);
    let mut request = JoinRequest {
        name: "测试房主".to_owned(),
        reconnect_token: ReconnectToken(113),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: Default::default(),
        identity_signature: Vec::new(),
        avatar_png,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(ROOM))
        .to_bytes()
        .to_vec();
    ClientCommand::join(request)
}

async fn send(
    host: &mut TcpClient,
    sequence: &mut u64,
    command: ClientCommand,
) -> Result<(), Box<dyn Error>> {
    *sequence += 1;
    host.send(&ClientMessage::new(ROOM, RequestId(*sequence), command))
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        return Err(io::Error::other("browser_room ADDRESS GAME [spectate] [AVATAR_PNG]").into());
    }
    let address: SocketAddr = args[0].parse()?;
    let rules = rules(&args[1])?;
    let count = player_count(&rules);
    let spectate = args.get(2).is_some_and(|argument| argument == "spectate");
    let avatar = args.get(3).map(fs::read).transpose()?;
    let session = HostSession::new(ROOM, GameSetup::shuffled(address.port(), rules))?;
    let server = TcpServerHandle::bind(address, session).await?;
    let local = SocketAddr::new("127.0.0.1".parse()?, server.local_addr().port());
    let mut host = TcpClient::connect(local).await?;
    let mut sequence = 0;
    send(&mut host, &mut sequence, join(avatar)).await?;
    let mut host_id = loop {
        if let ServerEvent::Joined { you } = host.receive().await?.event {
            break you;
        }
    };
    send(
        &mut host,
        &mut sequence,
        ClientCommand::SelectSeat {
            seat: SeatId(count - 1),
        },
    )
    .await?;
    for seat in u8::from(!spectate)..count - 1 {
        send(
            &mut host,
            &mut sequence,
            ClientCommand::ConfigureBotSeat {
                seat: SeatId(seat),
                occupied: true,
            },
        )
        .await?;
    }
    send(
        &mut host,
        &mut sequence,
        ClientCommand::SetReady { ready: true },
    )
    .await?;
    println!("READY {} ws://{}/", args[1], server.local_addr());
    let mut tick = interval(Duration::from_millis(200));
    let mut ping = interval(Duration::from_secs(3));
    let mut start_at = None;
    let mut automated = false;
    loop {
        tokio::select! {
            _ = ping.tick() => host.send(&ClientMessage::new(ROOM, RequestId(0), ClientCommand::Ping)).await?,
            _ = tick.tick() => {
                if start_at.is_some_and(|deadline| Instant::now() >= deadline) {
                    start_at = None;
                    send(&mut host, &mut sequence, ClientCommand::StartGame).await?;
                }
            }
            message = host.receive() => match message?.event {
                ServerEvent::LobbySnapshot(lobby) => {
                    host_id = lobby.host.unwrap_or(host_id);
                    automated = false;
                    if lobby.players.len() == usize::from(count) && lobby.players.iter().all(|player| player.ready && player.connected) {
                        start_at.get_or_insert_with(|| Instant::now() + Duration::from_secs(3));
                    } else { start_at = None; }
                }
                ServerEvent::GameSnapshot(game) => {
                    host_id = game.you();
                    if !automated {
                        automated = true;
                        println!("PLAYING {:?}", game.kind());
                        if let Some(command) = game.toggle_auto_play_command() {
                            send(&mut host, &mut sequence, ClientCommand::Game(command)).await?;
                        }
                    }
                }
                ServerEvent::ChatMessage(message) => {
                    if message.source != host_id && let ChatContent::Text(text) = message.content {
                        println!("CHAT {text}");
                        send(&mut host, &mut sequence, ClientCommand::Chat { content: ChatContent::Text(format!("tcp-ack-{text}")) }).await?;
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
