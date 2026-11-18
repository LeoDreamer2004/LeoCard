//! GPU-backed fixtures exercise the real game view without opening the client.

use super::super::assets::{self, DrawGuessAssets};
use super::super::canvas::{self, CanvasRenderer};
use super::super::client::DrawGuessClient;
use super::super::control_skin;
use super::super::feedback::{self, DrawingFeedback};
use super::super::libraries::{
    self, LibrarySelection, LibraryView, WordLibraryManager, WordLibraryRoom,
};
use super::super::state::{CanvasPaper, DrawGuessUiState, GuessInput};
use super::super::view::{self, DrawingContext};
use super::super::widgets;
use super::super::{palette, reveal};
use super::picking::assert_drawing_surfaces_receive_the_pointer;
use super::preview_libraries;
use crate::app::presentation::{
    GameSeatTransitionTarget, GameSummaryAnimation, StartGameSeatTransition,
    TurnBorderAnimationState, TurnBorderMaterial, animate_turn_border_traces,
};
use crate::app::runtime::{AvatarImages, GameProjection, UiAssets, load_ui_assets};
use crate::app::shell::SocialUiState;
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_resource::{PollType, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::text::{EditableText, TextEdit};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use leocard_draw_guess::{
    CanvasPoint, DrawAction, DrawGuessGame, DrawGuessPhase, DrawGuessRuleSet, GuessMessage, Ink,
    RasterCanvas,
};
use leocard_game_common::{PlayerId, guessing_reference_point_deltas};
use leocard_protocol::{
    DrawGuessPlayerView, DrawGuessSnapshot, MatchId, PlayerGameProfiles, PlayerPublicInfo,
    PlayerReferenceChange, ProfileId, SeatId, ServerEvent,
};
use leocard_ui::{UiComposer, UiSceneProperties, UiSceneRegistry, image_node_eq};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
enum SceneKind {
    Artist,
    Guesser,
    Choosing,
    Reveal,
    Finished,
    Libraries,
}

#[derive(Resource)]
struct PreviewScene {
    root: Entity,
    kind: SceneKind,
    snapshot: DrawGuessSnapshot,
    mount: bool,
}

fn scene(kind: SceneKind, count: u8) -> DrawGuessSnapshot {
    let rules = DrawGuessRuleSet {
        rounds: 1,
        drawing_seconds: 90,
    };
    let mut game = DrawGuessGame::new(
        rules,
        (0..count).map(PlayerId).collect(),
        vec!["小猫".into(), "向日葵".into(), "热气球".into()],
    )
    .unwrap();
    if !matches!(kind, SceneKind::Choosing | SceneKind::Libraries) {
        game.choose_word(PlayerId(0), 0).unwrap();
        for points in [
            vec![
                (250, 260),
                (258, 190),
                (296, 213),
                (340, 195),
                (384, 213),
                (420, 190),
                (430, 260),
                (418, 304),
                (380, 330),
                (300, 330),
                (260, 304),
                (250, 260),
            ],
            vec![(288, 256), (296, 246), (305, 256)],
            vec![(375, 256), (383, 246), (392, 256)],
            vec![(327, 277), (338, 283), (349, 277)],
            vec![(338, 283), (338, 296), (328, 306), (315, 297)],
            vec![(338, 296), (349, 306), (362, 297)],
            vec![(255, 272), (215, 264)],
            vec![(255, 286), (215, 290)],
            vec![(423, 272), (463, 264)],
            vec![(423, 286), (463, 290)],
            vec![
                (295, 328),
                (286, 390),
                (316, 405),
                (358, 405),
                (388, 390),
                (380, 328),
            ],
            vec![
                (388, 390),
                (420, 375),
                (435, 342),
                (460, 333),
                (475, 350),
                (462, 371),
                (445, 368),
            ],
        ] {
            let points: Vec<_> = points
                .into_iter()
                .map(|(x, y)| CanvasPoint { x, y })
                .collect();
            game.draw(
                PlayerId(0),
                &DrawAction::Begin {
                    point: points[0],
                    color: Ink([50, 115, 225]),
                    size: 7,
                    eraser: false,
                },
            )
            .unwrap();
            game.draw(
                PlayerId(0),
                &DrawAction::Extend {
                    points: points[1..].to_vec(),
                },
            )
            .unwrap();
            game.draw(PlayerId(0), &DrawAction::End).unwrap();
        }
        for (id, text) in [(1, "小狗"), (2, "狐狸"), (3, "兔子"), (4, "小猫")] {
            if id < count {
                game.guess(PlayerId(id), text).unwrap();
            }
            game.advance_ms(400);
        }
        if matches!(kind, SceneKind::Reveal) && game.phase() == DrawGuessPhase::Drawing {
            game.advance_ms(game.remaining_ms());
        } else if matches!(kind, SceneKind::Finished) {
            game.advance_ms(u32::MAX);
        }
    }
    let you = if matches!(kind, SceneKind::Guesser) {
        PlayerId(1)
    } else {
        PlayerId(0)
    };
    let mut state = game.view(you);
    let reference_changes = if matches!(kind, SceneKind::Finished) {
        for (index, player) in state.players.iter_mut().enumerate() {
            player.score = (u32::from(count) - index as u32) * 100;
        }
        let scores: Vec<_> = state.players.iter().map(|player| player.score).collect();
        state
            .players
            .iter()
            .zip(guessing_reference_point_deltas(&scores, rules.rounds).unwrap())
            .map(|(player, delta)| PlayerReferenceChange {
                player: player.id,
                profile_id: ProfileId([player.id.0 + 1; 32]),
                delta,
            })
            .collect()
    } else {
        Vec::new()
    };
    let names = [
        "玩家", "星河", "团子", "青柠", "木木", "阿狸", "小熊", "小雨",
    ];
    DrawGuessSnapshot {
        match_id: MatchId([33; 16]),
        host_port: 52300,
        you,
        host: PlayerId(0),
        rules,
        players: (0..count)
            .map(|index| DrawGuessPlayerView {
                player: PlayerPublicInfo {
                    id: PlayerId(index),
                    profile_id: ProfileId([index + 1; 32]),
                    name: names[usize::from(index)].to_owned(),
                    avatar: None,
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles: PlayerGameProfiles::default(),
                },
                seat: SeatId(index),
                ready: false,
                connected: true,
            })
            .collect(),
        state,
        match_complete: matches!(kind, SceneKind::Finished),
        reference_changes,
    }
}

#[derive(SystemParam)]
struct PreviewAssets<'w> {
    assets: Res<'w, UiAssets>,
    art: Res<'w, DrawGuessAssets>,
    avatars: Res<'w, AvatarImages>,
    transition: Res<'w, StartGameSeatTransition>,
    ui: Res<'w, DrawGuessUiState>,
    prediction: ResMut<'w, DrawGuessClient>,
    feedback: ResMut<'w, DrawingFeedback>,
    libraries: Res<'w, WordLibraryManager>,
    library_room: Res<'w, WordLibraryRoom>,
    turn_border_materials: ResMut<'w, Assets<TurnBorderMaterial>>,
    summary: Res<'w, GameSummaryAnimation>,
}

fn mount(
    mut commands: UiComposer,
    mut assets: PreviewAssets,
    mut fixture: ResMut<PreviewScene>,
    mut images: ResMut<Assets<Image>>,
) {
    if !fixture.mount {
        return;
    }
    fixture.mount = false;
    assets
        .prediction
        .apply(&ServerEvent::GameSnapshot(fixture.snapshot.clone().into()));
    assets.feedback.observe(&fixture.snapshot);
    let mut raster = RasterCanvas::default();
    raster.render(&fixture.snapshot.state.canvas);
    images.get_mut(&assets.art.canvas).unwrap().data = Some(raster.pixels);
    commands.begin(fixture.root);
    if matches!(fixture.kind, SceneKind::Libraries) {
        libraries::render(
            &mut commands,
            fixture.root,
            LibraryView {
                manager: &assets.libraries,
                room: &assets.library_room,
                host: true,
                assets: &assets.assets,
            },
        );
    } else {
        view::render(
            &mut commands,
            fixture.root,
            &fixture.snapshot,
            &assets.ui,
            &assets.art,
            DrawingContext {
                assets: &assets.assets,
                avatars: &assets.avatars,
                social: &SocialUiState::default(),
                transition: &assets.transition,
                summary: &assets.summary,
                turn_border_materials: &mut assets.turn_border_materials,
                reconnecting: None,
            },
        );
    }
    commands.finish();
}

fn update(app: &mut App) {
    app.update();
    app.world()
        .resource::<RenderDevice>()
        .wgpu_device()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .unwrap();
}

#[test]
#[ignore = "explicit visual fixture needs a GPU and writes preview images"]
fn renders_real_artist_guesser_and_eight_player_screens() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let output = root_dir.join("output/draw-guess-preview");
    fs::create_dir_all(&output).unwrap();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: root_dir.join("assets").to_string_lossy().into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .set(LogPlugin {
                filter: "error".to_owned(),
                ..default()
            })
            .disable::<WinitPlugin>(),
    )
    .add_plugins((leocard_ui::TextInputPlugin, leocard_ui::PointerInputPlugin))
    .add_plugins(UiMaterialPlugin::<TurnBorderMaterial>::default())
    .init_resource::<TurnBorderAnimationState>()
    .init_resource::<UiSceneRegistry>()
    .init_resource::<AvatarImages>()
    .init_resource::<DrawGuessUiState>()
    .init_resource::<WordLibraryManager>()
    .init_resource::<WordLibraryRoom>()
    .init_resource::<StartGameSeatTransition>()
    .init_resource::<DrawGuessClient>()
    .init_resource::<CanvasRenderer>()
    .init_resource::<DrawingFeedback>()
    .init_resource::<GameSummaryAnimation>()
    .add_systems(Startup, (load_ui_assets, assets::load))
    .add_systems(
        Update,
        (
            control_skin::prepare,
            mount,
            canvas::sync_canvas,
            view::fit_canvas,
            widgets::sync_buttons,
            palette::sync_palette,
            feedback::animate_feedback,
            feedback::animate_history,
            feedback::animate_choices,
            reveal::animate,
            animate_turn_border_traces,
        )
            .chain(),
    );
    let mut properties = UiSceneProperties::default();
    properties.preserve::<Node>();
    properties.preserve::<Text>();
    properties.preserve::<TextFont>();
    properties.preserve::<TextColor>();
    properties.preserve::<UiTransform>();
    properties.preserve::<Visibility>();
    properties.preserve_with::<ImageNode>(image_node_eq);
    app.insert_resource(properties);
    app.finish();
    app.cleanup();
    let camera = app.world_mut().spawn(Camera2d).id();
    let cases = [
        ("artist", SceneKind::Artist, 6, 1280, 720),
        ("guesser", SceneKind::Guesser, 6, 1280, 720),
        ("wide-artist", SceneKind::Artist, 8, 1680, 720),
        ("small-guesser", SceneKind::Guesser, 8, 720, 480),
        ("choosing", SceneKind::Choosing, 6, 1280, 720),
        ("reveal", SceneKind::Reveal, 6, 1280, 720),
        ("small-reveal", SceneKind::Reveal, 8, 720, 480),
        ("results", SceneKind::Finished, 8, 1280, 720),
        ("small-results", SceneKind::Finished, 8, 720, 480),
        ("word-libraries", SceneKind::Libraries, 6, 1280, 720),
        ("word-libraries-small", SceneKind::Libraries, 6, 960, 540),
        ("word-libraries-builtin", SceneKind::Libraries, 6, 1280, 720),
    ];
    let mut previous_root = None;
    for (name, kind, count, width, height) in cases {
        if let Some(root) = previous_root {
            app.world_mut().despawn(root);
        }
        let mut target =
            Image::new_target_texture(width, height, TextureFormat::Rgba8UnormSrgb, None);
        target.asset_usage = RenderAssetUsages::RENDER_WORLD;
        let target: RenderTarget = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(target)
            .into();
        app.world_mut().entity_mut(camera).insert(target.clone());
        let root = app
            .world_mut()
            .spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                UiTargetCamera(camera),
            ))
            .id();
        app.insert_resource(UiScale(
            (width as f32 / 1280.0).min(height as f32 / 720.0).max(0.5),
        ));
        app.insert_resource(DrawingFeedback::default());
        app.insert_resource(GameSummaryAnimation {
            elapsed: 6.0,
            ..default()
        });
        let mut libraries = preview_libraries::manager();
        if name == "word-libraries-builtin" {
            libraries.select(LibrarySelection::Builtin);
        }
        app.insert_resource(libraries);
        app.insert_resource(PreviewScene {
            root,
            kind,
            snapshot: scene(kind, count),
            mount: true,
        });
        for _ in 0..40 {
            update(&mut app);
            thread::sleep(Duration::from_millis(10));
        }
        let papers: Vec<_> = app
            .world_mut()
            .query_filtered::<&ComputedNode, With<CanvasPaper>>()
            .iter(app.world())
            .map(|node| node.size())
            .collect();
        if matches!(kind, SceneKind::Libraries) {
            assert!(papers.is_empty());
            if name == "word-libraries-builtin" {
                preview_libraries::assert_builtin_scrolls(&mut app);
            } else {
                preview_libraries::assert_multiline_editor(&mut app, &target);
            }
        } else {
            assert_eq!(papers.len(), 1);
            assert!(
                (papers[0].x - papers[0].y * 1.6).abs() <= 1.6,
                "{name}: {papers:?}"
            );
        }
        let inputs = app
            .world_mut()
            .query_filtered::<Entity, With<GuessInput>>()
            .iter(app.world())
            .count();
        assert_eq!(inputs, usize::from(matches!(kind, SceneKind::Guesser)));
        if matches!(kind, SceneKind::Guesser) {
            let editor = app
                .world_mut()
                .query_filtered::<Entity, With<GuessInput>>()
                .single(app.world())
                .unwrap();
            app.insert_resource(InputFocus::from_entity(editor));
            app.world_mut()
                .get_mut::<EditableText>(editor)
                .unwrap()
                .queue_edit(TextEdit::Insert("正在输入".into()));
            for _ in 0..3 {
                update(&mut app);
            }
            let mut fixture = app.world_mut().resource_mut::<PreviewScene>();
            fixture.snapshot.state.history.push(GuessMessage {
                serial: 5,
                player: PlayerId(2),
                text: "熊猫".into(),
                correct: false,
                points: 0,
                private: false,
            });
            fixture.mount = true;
            for _ in 0..12 {
                update(&mut app);
            }
            assert_eq!(
                app.world()
                    .get::<EditableText>(editor)
                    .unwrap()
                    .value()
                    .to_string(),
                "正在输入"
            );
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(editor));
            app.world_mut()
                .get_mut::<EditableText>(editor)
                .unwrap()
                .clear();
            for _ in 0..3 {
                update(&mut app);
            }
        }
        let portraits = app
            .world_mut()
            .query_filtered::<Entity, With<GameSeatTransitionTarget>>()
            .iter(app.world())
            .count();
        assert_eq!(
            portraits,
            if matches!(kind, SceneKind::Libraries) {
                0
            } else {
                usize::from(count)
            }
        );
        if matches!(kind, SceneKind::Artist) {
            assert_drawing_surfaces_receive_the_pointer(&mut app, &target);
        }
        app.world_mut()
            .spawn(Screenshot::image(target.as_image().unwrap().clone()))
            .observe(save_to_disk(output.join(format!("{name}.png"))));
        for _ in 0..8 {
            update(&mut app);
        }
        previous_root = Some(root);
    }
    for _ in 0..8 {
        update(&mut app);
    }
    for name in [
        "artist",
        "guesser",
        "wide-artist",
        "small-guesser",
        "choosing",
        "results",
    ] {
        assert!(output.join(format!("{name}.png")).exists());
    }
}
