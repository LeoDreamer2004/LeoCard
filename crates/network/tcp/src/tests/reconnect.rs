use super::support::{Peer, command, join, server};
use futures_util::StreamExt;
use leocard_protocol::{
    ChatContent, ClientCommand, GameCommand, RejectReason, RequestId, RequestViolation,
    ServerEvent, UnoCommand,
};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn reconnect_syncs_a_lagging_player_without_inheriting_old_request_numbers() {
    // Refresh resets the sequence, while an automatic reconnect keeps counting.
    for (secure, next_request) in [(false, 2), (true, 2), (false, 21)] {
        let server = server().await;
        let address = server.local_addr();
        let mut host = Peer::tcp(address).await;
        host.send(join(1)).await;
        host.until(|event| matches!(event, ServerEvent::LobbySnapshot(_)))
            .await;
        let mut previous = Peer::ws(address, secure).await;
        previous.send(join(2)).await;
        let joined = previous
            .until(|event| matches!(event, ServerEvent::Joined { .. }))
            .await;
        previous
            .send(command(20, ClientCommand::SetReady { ready: true }))
            .await;
        host.until(|event| matches!(event, ServerEvent::LobbySnapshot(lobby) if lobby.players.len() == 2 && lobby.players.iter().all(|player| player.ready))).await;

        // The old browser has not consumed the transition from lobby to game.
        host.send(command(2, ClientCommand::StartGame)).await;
        host.until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
            .await;
        host.send(command(
            3,
            ClientCommand::Game(GameCommand::Uno(UnoCommand::SetAutoPlay { enabled: true })),
        ))
        .await;
        let latest = host
            .until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
            .await;
        assert!(latest.revision > joined.revision);
        let mut resumed = Peer::ws(address, secure).await;
        resumed.send(join(2)).await;
        let snapshot = resumed
            .until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
            .await;
        assert!(snapshot.revision >= latest.revision);
        let (ServerEvent::GameSnapshot(before), ServerEvent::GameSnapshot(after)) =
            (latest.event, snapshot.event)
        else {
            unreachable!()
        };
        assert_eq!(before.match_id(), after.match_id());
        assert_eq!(after.players().len(), 2);
        assert!(
            after
                .uno()
                .unwrap()
                .players
                .iter()
                .find(|player| player.player.id == after.host())
                .unwrap()
                .auto_play
        );
        assert!(!after.uno().unwrap().your_hand.is_empty());
        // Starting the game can renumber seats, so compare the restored profile.
        assert_eq!(after.player(after.you()).unwrap().name, "Socket2");

        resumed
            .send(command(next_request, ClientCommand::RequestSnapshot))
            .await;
        let synced = resumed
            .until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
            .await;
        assert_eq!(synced.in_reply_to, Some(RequestId(next_request)));

        // Replaced sockets are retired even if their disconnect arrives later.
        let Peer::WebSocket(socket) = &mut previous else {
            unreachable!()
        };
        timeout(Duration::from_secs(2), async {
            while let Some(Ok(message)) = socket.next().await {
                if matches!(message, Message::Close(_)) {
                    break;
                }
            }
        })
        .await
        .expect("old socket retired after reconnect");

        // Duplicate and stale protection still applies within the new socket.
        for (request, expected) in [
            (
                next_request,
                RequestViolation::DuplicateRequest {
                    last_seen: RequestId(next_request),
                },
            ),
            (
                next_request - 1,
                RequestViolation::StaleRequest {
                    last_seen: RequestId(next_request),
                },
            ),
        ] {
            resumed
                .send(command(request, ClientCommand::RequestSnapshot))
                .await;
            let rejected = resumed
                .until(|event| matches!(event, ServerEvent::Rejected { .. }))
                .await;
            assert_eq!(
                rejected.event,
                ServerEvent::Rejected {
                    reason: RejectReason::Request(expected),
                }
            );
        }
        let text = "reconnected";
        resumed
            .send(command(
                next_request + 1,
                ClientCommand::Chat {
                    content: ChatContent::Text(text.into()),
                },
            ))
            .await;
        host.until(|event| matches!(event, ServerEvent::ChatMessage(message) if message.content == ChatContent::Text(text.into()))).await;
        server.shutdown().await.unwrap();
    }
}
