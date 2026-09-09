pub(super) use super::super::*;
pub(super) use ed25519_dalek::{Signer, SigningKey};
pub(super) use leocard_host::GameSetup;
pub(super) use leocard_protocol::{
    ClientCommand, GameSnapshot, JoinRequest, PlayerGameProfiles, PlayerId, ProfileId,
    QiGui523Snapshot, ReconnectToken, RequestId, RoomId, SeatId, ServerEvent, ShengjiSnapshot,
    TexasHoldemSnapshot, UnoSnapshot, join_identity_payload,
};
pub(super) use leocard_qigui523::{QiGuiRuleSet, build_deck};
pub(super) use leocard_shengji::ShengjiRuleSet;
pub(super) use leocard_texas_holdem::TexasHoldemRuleSet;
pub(super) use leocard_uno::UnoRuleSet;
pub(super) use tokio::time::{Duration, timeout};

pub(super) async fn receive_joined(client: &mut TcpClient) -> PlayerId {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::Joined { you } = client.receive().await.unwrap().event {
                return you;
            }
        }
    })
    .await
    .expect("server should confirm the join")
}

pub(super) async fn receive_ready(client: &mut TcpClient, you: PlayerId) {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::LobbySnapshot(snapshot) = client.receive().await.unwrap().event
                && snapshot
                    .players
                    .iter()
                    .any(|player| player.id == you && player.ready)
            {
                return;
            }
        }
    })
    .await
    .expect("server should publish the ready state");
}

pub(super) async fn receive_game(client: &mut TcpClient) -> QiGui523Snapshot {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::GameSnapshot(snapshot) = client.receive().await.unwrap().event {
                return snapshot.into_qigui523().expect("七鬼五二三快照");
            }
        }
    })
    .await
    .expect("server should send a game snapshot")
}

pub(super) async fn receive_texas_game(client: &mut TcpClient) -> TexasHoldemSnapshot {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)) =
                client.receive().await.unwrap().event
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("server should send a Texas Hold'em snapshot")
}

pub(super) async fn receive_shengji_dealt_card(client: &mut TcpClient) -> ShengjiSnapshot {
    timeout(Duration::from_secs(3), async {
        loop {
            if let ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) =
                client.receive().await.unwrap().event
                && !snapshot.your_hand.is_empty()
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("slow deal should reach every Shengji player")
}

pub(super) async fn receive_uno_game(client: &mut TcpClient) -> UnoSnapshot {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) =
                client.receive().await.unwrap().event
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("server should send an UNO snapshot")
}
