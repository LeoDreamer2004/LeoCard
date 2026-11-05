use super::super::{
    MahjongLinkUiAction, actions, playback::LinkPlayback, state::MahjongLinkUiState, table,
};
use super::{TestRoot, host_client, ui_assets};
use crate::app::games::GameTableContext;
use crate::app::presentation::mahjong::{MahjongPresentationPlugin, MahjongTileAssets};
use crate::app::presentation::{
    GameSummaryAnimation, MahjongTileMaterial, StartGameSeatTransition, TableBackgroundMaterial,
    TurnBorderMaterial,
};
use crate::app::runtime::{AppearancePreferences, AvatarImages, ClientResource, TableAppearance};
use crate::app::shell::{
    ChatPanelState, DeveloperHandInput, PressedUiAction, SocialUiState, UiAction,
};
use bevy::asset::AssetPlugin;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::scene::ScenePatch;
use bevy::shader::Shader;
use bevy::ui_widgets::Button;
use leocard_game_common::PlayerId;
use leocard_mahjong_link::{
    LinkCell, MahjongLinkAction, MahjongLinkGameState, MahjongLinkRuleSet, PushSide, build_deck,
};
use leocard_protocol::{
    GameSnapshot, MahjongLinkPlayerView, MahjongLinkSnapshot, MatchId, PROTOCOL_VERSION, Revision,
    SeatId, ServerEvent, ServerMessage,
};
use leocard_ui::{PointerInputPlugin, UiComposer, UiSceneProperties, UiSceneRegistry};

#[expect(
    clippy::too_many_arguments,
    reason = "the regression runs the real table compositor with its rendering resources"
)]
fn render_table(
    mut commands: UiComposer,
    client: Res<ClientResource>,
    tiles: Res<MahjongTileAssets>,
    mut ui: ResMut<MahjongLinkUiState>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut table_materials: ResMut<Assets<TableBackgroundMaterial>>,
    mut borders: ResMut<Assets<TurnBorderMaterial>>,
    roots: Query<Entity, With<TestRoot>>,
) {
    let game = client
        .0
        .model()
        .game_snapshot()
        .unwrap()
        .mahjong_link()
        .unwrap();
    ui.reconcile(game);
    let root = roots.single().unwrap();
    commands.begin(root);
    table::render(
        &mut commands,
        root,
        game,
        &ui,
        &tiles,
        &LinkPlayback::default(),
        GameTableContext {
            client: &client,
            social: &SocialUiState::default(),
            chat: &ChatPanelState::default(),
            developer_hand: &DeveloperHandInput::default(),
            form: &AppearancePreferences::default(),
            assets: &ui_assets(),
            avatars: &AvatarImages::default(),
            appearance: &TableAppearance::default(),
            game_summary: &GameSummaryAnimation::default(),
            transition: &StartGameSeatTransition::default(),
            table_materials: &mut table_materials,
            mahjong_tile_materials: &mut materials,
            turn_border_materials: &mut borders,
        },
    );
    commands.finish();
}

fn apply_snapshot(app: &mut App, snapshot: MahjongLinkSnapshot) {
    let mut client = app.world_mut().resource_mut::<ClientResource>();
    assert!(client.0.model_mut().apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: leocard_client::NETWORK_ROOM_ID,
        revision: Revision(100 + u64::from(snapshot.turn) * 2 + u64::from(snapshot.push_used)),
        in_reply_to: None,
        game_context: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::MahjongLink(snapshot)),
    }));
}

#[test]
fn tiles_can_be_selected_again_after_a_complete_round() {
    for count in 1..=MahjongLinkRuleSet::MAX_PLAYERS {
        verify_turn_selection(count);
    }
}

fn verify_turn_selection(count: u8) {
    let client = host_client();
    let lobby = client.model().lobby().unwrap();
    let you = client.model().you().unwrap();
    let rules = MahjongLinkRuleSet::default();
    let mut game = MahjongLinkGameState::new(count, build_deck()).unwrap();
    let mut snapshot = MahjongLinkSnapshot {
        match_id: MatchId([1; 16]),
        host_port: lobby.host_port,
        you,
        host: you,
        rules,
        board: game.board().clone(),
        players: (0..count)
            .map(|index| {
                let mut player = lobby.players[0].player.clone();
                player.id = PlayerId(you.0 + index);
                MahjongLinkPlayerView {
                    player,
                    seat: SeatId(index),
                    ready: false,
                    connected: true,
                    auto_play: false,
                    score: 0,
                }
            })
            .collect(),
        current_player: Some(you),
        turn: 0,
        push_used: false,
        consecutive_passes: 0,
        match_complete: false,
        last_action: None,
        reference_changes: Vec::new(),
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), PointerInputPlugin))
        .init_resource::<Assets<ScenePatch>>()
        .init_asset::<Image>()
        .init_asset::<Shader>()
        .add_plugins(MahjongPresentationPlugin)
        .init_resource::<Assets<TableBackgroundMaterial>>()
        .init_resource::<Assets<TurnBorderMaterial>>()
        .init_resource::<UiSceneRegistry>()
        .init_resource::<UiSceneProperties>()
        .init_resource::<MahjongLinkUiState>()
        .add_message::<PressedUiAction>()
        .insert_resource(ClientResource(client))
        .add_systems(Update, (actions::dispatch_actions, render_table).chain());
    app.world_mut().spawn((TestRoot, Node::default()));
    let cell = LinkCell { row: 7, column: 16 };
    let mut retained_tile = None;
    // Keep the fixture's matches inside its first row while covering a full round.
    let rounds = (count * 2).min(8);
    for turn in 0..=rounds {
        snapshot.board = game.board().clone();
        snapshot.turn = game.turn();
        snapshot.push_used = game.push_used();
        snapshot.current_player = game.current_player().map(|index| PlayerId(you.0 + index));
        apply_snapshot(&mut app, snapshot.clone());
        app.update();
        let tile = *retained_tile.get_or_insert_with(|| app.world_mut().query::<(Entity, &UiAction)>()
            .iter(app.world())
            .find(|(_, action)| matches!(action, UiAction::MahjongLink(MahjongLinkUiAction::Select(target)) if *target == cell))
            .map(|(entity, _)| entity).unwrap());
        let entity = app.world().entity(tile);
        let pickable = entity.get::<Pickable>().copied().unwrap_or_default();
        let button = entity.contains::<Button>();
        if turn % count == 0 {
            assert!(button, "turn {turn} must restore the tile button");
            assert!(
                pickable.is_hoverable,
                "turn {turn} leaves the tile unpickable"
            );
            assert!(app.world().get::<UiAction>(tile).is_some());
            app.world_mut()
                .write_message(PressedUiAction(UiAction::MahjongLink(
                    MahjongLinkUiAction::Select(cell),
                )));
            app.update();
            assert_eq!(
                app.world().resource::<MahjongLinkUiState>().selected,
                Some(cell)
            );
            if turn == count {
                game.act(
                    0,
                    MahjongLinkAction::Push {
                        row: 0,
                        from: PushSide::Right,
                        distance: 1,
                    },
                )
                .unwrap();
                snapshot.board = game.board().clone();
                snapshot.push_used = true;
                apply_snapshot(&mut app, snapshot.clone());
                app.update();
                assert_eq!(app.world().resource::<MahjongLinkUiState>().selected, None);
                assert!(
                    app.world()
                        .entity(tile)
                        .get::<Pickable>()
                        .unwrap()
                        .is_hoverable
                );
                let endpoint = LinkCell {
                    row: 0,
                    column: turn * 2 - 1,
                };
                assert!(game.board().maximum_push(0, PushSide::Left) > 0);
                app.world_mut()
                    .write_message(PressedUiAction(UiAction::MahjongLink(
                        MahjongLinkUiAction::Select(endpoint),
                    )));
                app.update();
                assert_eq!(
                    app.world().resource::<MahjongLinkUiState>().selected,
                    Some(endpoint)
                );
                assert!(!app.world_mut().query::<&UiAction>().iter(app.world()).any(
                    |action| matches!(
                        action,
                        UiAction::MahjongLink(MahjongLinkUiAction::Act(
                            MahjongLinkAction::Push { .. }
                        ))
                    )
                ));
            }
            app.world_mut()
                .resource_mut::<MahjongLinkUiState>()
                .selected = None;
        } else {
            assert!(!button);
            assert!(!pickable.is_hoverable);
        }
        if turn < rounds {
            let pair = turn * 2 - u8::from(turn >= count);
            game.act(
                turn % count,
                MahjongLinkAction::Match {
                    first: LinkCell {
                        row: 0,
                        column: pair,
                    },
                    second: LinkCell {
                        row: 0,
                        column: pair + 1,
                    },
                },
            )
            .unwrap();
        }
    }
}
