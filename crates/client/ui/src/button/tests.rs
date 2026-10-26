use super::{ButtonHighlight, TextureButton, update_button_highlights};
use crate::{PointerInputPlugin, UiComposer, UiSceneProperties, UiSceneRegistry};
use bevy::asset::AssetPlugin;
use bevy::ecs::system::RunSystemOnce;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::scene::ScenePatch;
use bevy::ui_widgets::Button;

#[derive(Clone, Component, Debug, PartialEq)]
struct Action(u8);

#[derive(Resource)]
struct View {
    root: Entity,
    action: Option<Action>,
    button: Option<Entity>,
}

fn compose(mut commands: UiComposer, mut view: ResMut<View>) {
    commands.begin(view.root);
    let (button, _) = TextureButton {
        node: Node::default(),
        normal: ImageNode::default(),
        hovered: ImageNode::default(),
        action: view.action.clone(),
        label: Some("通用动作"),
        font_size: 14.0,
        color: Color::WHITE,
        icon: None,
    }
    .mount(&mut commands, view.root, &Handle::default());
    view.button = Some(button);
    commands.finish();
}

#[test]
fn generic_actions_and_feedback_follow_enabled_state_across_scene_rebuilds() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), PointerInputPlugin))
        .init_resource::<Assets<ScenePatch>>()
        .init_resource::<UiSceneRegistry>()
        .init_resource::<UiSceneProperties>()
        .add_systems(Update, compose);
    let root = app.world_mut().spawn(Node::default()).id();
    app.insert_resource(View {
        root,
        action: Some(Action(1)),
        button: None,
    });
    app.update();
    let button = app.world().resource::<View>().button.unwrap();
    assert_eq!(app.world().get::<Action>(button), Some(&Action(1)));
    assert!(app.world().get::<Button>(button).is_some());
    let overlay = app.world().get::<ButtonHighlight>(button).unwrap().overlay;
    app.world_mut()
        .entity_mut(button)
        .insert(PickingInteraction::Hovered);
    app.world_mut()
        .run_system_once(update_button_highlights)
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(overlay),
        Some(&Visibility::Visible)
    );

    app.world_mut().resource_mut::<View>().action = None;
    app.update();
    assert_eq!(app.world().resource::<View>().button, Some(button));
    assert!(app.world().get::<Action>(button).is_none());
    assert!(app.world().get::<Button>(button).is_none());
    assert!(app.world().get::<ButtonHighlight>(button).is_none());
    assert!(!app.world().get::<Pickable>(button).unwrap().is_hoverable);

    app.world_mut().resource_mut::<View>().action = Some(Action(9));
    app.update();
    assert_eq!(app.world().resource::<View>().button, Some(button));
    assert_eq!(app.world().get::<Action>(button), Some(&Action(9)));
    assert!(app.world().get::<ButtonHighlight>(button).is_some());
    assert!(app.world().get::<Pickable>(button).unwrap().is_hoverable);
}
