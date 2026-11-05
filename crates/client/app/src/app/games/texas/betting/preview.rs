use super::super::{TexasUiState, chips::tests::pot_test_snapshot};
use super::view::add_texas_actions;
use crate::app::runtime::{ControlAssets, HomeAssets, UI_FONT_ASSET, UiAssets};
use bevy::audio::AudioPlugin;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use leocard_ui::{TextInputPlugin, UiComposer, UiSceneProperties, UiSceneRegistry};
use std::time::Duration;

#[derive(Resource)]
struct Target(Handle<Image>);

fn setup(mut commands: Commands, server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_target_texture(
        760,
        480,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    let camera = commands
        .spawn((Camera2d, RenderTarget::Image(image.clone().into())))
        .id();
    commands.insert_resource(Target(image));
    commands.insert_resource(UiAssets {
        font: server.load(UI_FONT_ASSET),
        controls: ControlAssets {
            game_play_button: server.load("ui/game-buttons/play-normal.png"),
            game_play_button_hover: server.load("ui/game-buttons/play-hover.png"),
            game_pass_button: server.load("ui/game-buttons/pass-normal.png"),
            game_pass_button_hover: server.load("ui/game-buttons/pass-hover.png"),
            game_hint_button: server.load("ui/game-buttons/hint-normal.png"),
            game_hint_button_hover: server.load("ui/game-buttons/hint-hover.png"),
            game_warning_button: server.load("ui/game-buttons/warning-normal.png"),
            game_warning_button_hover: server.load("ui/game-buttons/warning-hover.png"),
            ..default()
        },
        home: HomeAssets {
            slider: server.load("ui/home/slider.png"),
            input: server.load("ui/home/cozy-input-compact.png"),
            focused_input: server.load("ui/home/cozy-input-focused-compact.png"),
            ..default()
        },
        ..default()
    });
    commands.spawn((
        Node {
            width: px(760),
            height: px(480),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.16, 0.11)),
        UiTargetCamera(camera),
    ));
}

fn compose(
    mut commands: UiComposer,
    assets: Res<UiAssets>,
    roots: Query<Entity, With<UiTargetCamera>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    *done = true;
    let root = roots.single().unwrap();
    let mut game = pot_test_snapshot(&[(0, false, false), (2, false, false)]);
    game.minimum_raise_to = 4;
    commands.begin(root);
    for (index, open) in [false, true].into_iter().enumerate() {
        let panel = commands.child(
            root,
            Node {
                position_type: PositionType::Absolute,
                top: px(index as f32 * 240.0),
                width: px(760),
                height: px(240),
                ..default()
            },
        );
        let mut ui = TexasUiState {
            raise_to: 20,
            raise_editor_open: open,
            ..default()
        };
        add_texas_actions(
            &mut commands,
            panel,
            &game,
            &game.players[0],
            &mut ui,
            &assets,
        );
    }
    commands.finish();
}

#[test]
#[ignore]
fn temporary_betting_preview() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets").into(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<AudioPlugin>(),
    )
    .init_resource::<UiSceneRegistry>()
    .init_resource::<UiSceneProperties>()
    .add_plugins(TextInputPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, compose);
    app.finish();
    app.cleanup();
    for frame in 0..240 {
        if frame == 140 {
            let handle = app.world().resource::<Target>().0.clone();
            app.world_mut()
                .spawn(Screenshot::image(handle))
                .observe(save_to_disk("/tmp/leocard-texas-slider-fixed.png"));
        }
        app.update();
        std::thread::sleep(Duration::from_millis(12));
    }
}
