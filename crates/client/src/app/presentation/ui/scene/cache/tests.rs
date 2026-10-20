use super::super::{UiComposer, UiSceneProperties, UiSceneRegistry};
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::scene::ScenePatch;

#[derive(Component)]
struct Leaf;

#[derive(Component)]
struct FirstRevision;

#[derive(Resource)]
struct Scene {
    root: Entity,
    revision: u64,
    visible: bool,
    builds: usize,
}

fn compose(mut commands: UiComposer, mut scene: ResMut<Scene>) {
    commands.begin(scene.root);
    if scene.visible {
        let (panel, retained) =
            commands.cached_child(scene.root, "table", scene.revision, Node::default());
        if !retained {
            scene.builds += 1;
            let leaf = commands.child(
                panel,
                (Leaf, Text::new(format!("round {}", scene.revision))),
            );
            if scene.revision == 1 {
                commands.entity(leaf).insert(FirstRevision);
            }
        }
    }
    commands.finish();
}

#[test]
fn retained_subtrees_preserve_live_state_and_reconcile_when_invalidated_or_removed() {
    let mut app = App::new();
    let mut properties = UiSceneProperties::default();
    properties.preserve::<Text>();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<Assets<ScenePatch>>()
        .init_resource::<UiSceneRegistry>()
        .insert_resource(properties)
        .add_systems(Update, compose);
    let root = app.world_mut().spawn(Node::default()).id();
    app.insert_resource(Scene {
        root,
        revision: 1,
        visible: true,
        builds: 0,
    });
    app.update();
    let leaf = app
        .world_mut()
        .query_filtered::<Entity, With<Leaf>>()
        .single(app.world())
        .unwrap();
    app.world_mut().get_mut::<Text>(leaf).unwrap().0 = "live state".into();
    let tick = app
        .world()
        .entity(leaf)
        .get_ref::<Text>()
        .unwrap()
        .last_changed();
    app.update();
    assert_eq!(app.world().resource::<Scene>().builds, 1);
    assert_eq!(app.world().get::<Text>(leaf).unwrap().0, "live state");
    assert_eq!(
        app.world()
            .entity(leaf)
            .get_ref::<Text>()
            .unwrap()
            .last_changed(),
        tick
    );
    assert!(app.world().get::<FirstRevision>(leaf).is_some());

    app.world_mut().resource_mut::<Scene>().revision = 2;
    app.update();
    assert_eq!(app.world().resource::<Scene>().builds, 2);
    assert_eq!(app.world().get::<Text>(leaf).unwrap().0, "round 2");
    assert!(app.world().get::<FirstRevision>(leaf).is_none());

    app.world_mut().resource_mut::<Scene>().visible = false;
    app.update();
    assert!(!app.world().entities().contains(leaf));
    assert!(app.world().entities().contains(root));
}
