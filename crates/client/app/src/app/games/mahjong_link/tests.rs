mod turns;

use super::lobby;
use crate::app::games::GameLobbyContext;
use crate::app::presentation::LobbySeatHover;
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::scene::ScenePatch;
use leocard_client::{
    LocalPlayerConnection, LocalPlayerProfile, PlayerIdentity, PlayerRatingProfile, TcpGameClient,
};
use leocard_mahjong_link::MahjongLinkRuleSet;
use leocard_protocol::{ClientCommand, PlayerInteractionKind, SeatId};
use leocard_ui::{UiComposer, UiSceneProperties, UiSceneRegistry};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Component)]
struct TestRoot;

fn render_lobby(
    mut commands: UiComposer,
    client: Res<ClientResource>,
    assets: Res<UiAssets>,
    avatars: Res<AvatarImages>,
    roots: Query<Entity, With<TestRoot>>,
) {
    let root = roots
        .iter()
        .next()
        .unwrap_or_else(|| commands.spawn((TestRoot, Node::default())).id());
    commands.begin(root);
    lobby::render(
        &mut commands,
        root,
        client.0.model().lobby().expect("joined lobby"),
        GameLobbyContext {
            client: &client,
            assets: &assets,
            avatars: &avatars,
        },
    );
    commands.finish();
}

fn host_client() -> TcpGameClient {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut client = TcpGameClient::host_with_profile(
        port,
        MahjongLinkRuleSet::default().into(),
        LocalPlayerConnection::from_profile(
            "房主",
            None,
            &LocalPlayerProfile {
                identity: PlayerIdentity::generate().unwrap(),
                rating: PlayerRatingProfile {
                    reference_points: 0,
                    completed_games: 0,
                    applied_matches: default(),
                    last_change: None,
                },
                game_profiles: default(),
            },
            default(),
            default(),
        ),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while client.model().lobby().is_none() {
        client.poll();
        assert!(Instant::now() < deadline, "lobby connection timed out");
        thread::sleep(Duration::from_millis(10));
    }
    client
}

fn ui_assets() -> UiAssets {
    let mut assets = UiAssets::default();
    for kind in [
        PlayerInteractionKind::Flower,
        PlayerInteractionKind::Egg,
        PlayerInteractionKind::Wine,
        PlayerInteractionKind::Shoe,
    ] {
        assets
            .social
            .interaction_images
            .insert((kind, false), default());
    }
    assets
}

#[test]
fn clicking_mahjong_link_creates_and_renders_a_room() {
    let mut client = host_client();
    let mut app = App::new();
    client.send(ClientCommand::SelectSeat { seat: SeatId(0) });
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<Assets<ScenePatch>>()
        .init_resource::<UiSceneRegistry>()
        .init_resource::<UiSceneProperties>()
        .insert_resource(ui_assets())
        .init_resource::<AvatarImages>()
        .insert_resource(ClientResource(client))
        .add_systems(Update, render_lobby);
    app.update();
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0 == "麻将连连看")
    );
    assert_eq!(
        app.world_mut()
            .query::<&LobbySeatHover>()
            .iter(app.world())
            .count(),
        usize::from(MahjongLinkRuleSet::MAX_PLAYERS),
    );
}
