use super::properties::author;
use super::registry::{UiSceneKey, UiSceneRegistry};
use bevy::ecs::bundle::{BundleScratch, BundleWriter};
use bevy::ecs::component::ComponentsRegistrator;
use bevy::ecs::error::Result;
use bevy::ecs::template::{SceneEntityReference, template};
use bevy::prelude::*;
use bevy::scene::{ResolvedScene, ResolvedSceneRoot, ScenePatch};
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

#[derive(Component, Clone)]
struct SceneReferencesBound;

pub(super) struct Mount {
    pub parent: Entity,
    pub entity: Entity,
    pub owner: Entity,
    pub frame: u64,
}

pub(super) struct PreparedScene {
    resolved: ResolvedSceneRoot,
    entities: Vec<Entity>,
    mount: Mount,
}

pub(super) fn prepare(
    registry: &mut UiSceneRegistry,
    assets: &AssetServer,
    patches: &Assets<ScenePatch>,
    mount: Mount,
    scene: impl Scene,
    reserve: impl FnMut() -> Entity,
) -> Result<PreparedScene> {
    let bindings = Arc::new(OnceLock::<Vec<(SceneEntityReference, Entity)>>::new());
    let bound = bindings.clone();
    let bind = template(move |context| {
        for (reference, entity) in bound
            .get()
            .expect("scene references are resolved before applying")
        {
            context.entity_references.set(*reference, *entity);
        }
        Ok(SceneReferencesBound)
    });
    let mut resolved = ResolvedSceneRoot::resolve(Box::new((bind, scene)), assets, patches)?;
    let (entities, references) = {
        let mut hierarchy = SceneHierarchy {
            reserve,
            registry,
            owner: mount.owner,
            frame: mount.frame,
            entities: Vec::new(),
            references: Vec::new(),
        };
        hierarchy.prepare(&mut resolved.scene, mount.parent, mount.entity);
        (hierarchy.entities, hierarchy.references)
    };
    bindings
        .set(references)
        .expect("scene reference bindings are set once");
    Ok(PreparedScene {
        resolved,
        entities,
        mount,
    })
}

impl PreparedScene {
    pub(super) fn apply(self, world: &mut World) -> Result {
        let Mount { parent, entity, .. } = self.mount;
        if !world.entities().contains(parent) || !world.entities().contains(entity) {
            return Ok(());
        }
        world.resource_scope(|world, mut registry: Mut<UiSceneRegistry>| {
            author(world, &mut registry, &self.entities, |world| {
                self.resolved
                    .apply(&mut world.entity_mut(entity), &mut BundleScratch::default())?;
                world.entity_mut(entity).insert(ChildOf(parent));
                Ok(())
            })
        })
    }
}

struct SceneHierarchy<'a, F> {
    reserve: F,
    registry: &'a mut UiSceneRegistry,
    owner: Entity,
    frame: u64,
    entities: Vec<Entity>,
    references: Vec<(SceneEntityReference, Entity)>,
}

impl<F: FnMut() -> Entity> SceneHierarchy<'_, F> {
    fn prepare(&mut self, scene: &mut ResolvedScene, parent: Entity, entity: Entity) {
        if scene.entity_references.is_empty() {
            scene.entity_references.push(SceneEntityReference::new(
                (file!(), 0, 0),
                self.entities.len(),
                self.frame,
            ));
        }
        self.references.extend(
            scene
                .entity_references
                .iter()
                .map(|reference| (*reference, entity)),
        );
        self.entities.push(entity);
        self.registry.child(parent, entity);
        let related = scene.get_or_insert_related_resolved_scenes::<ChildOf>();
        // Keep the existing relationship target. Reusing named children then keeps
        // their parent intact; the final ordered child list is applied after composition.
        related.insert_relationship_target = keep_children;
        let children = &mut related.scenes;
        let mut keys = HashSet::new();
        for (index, child) in children.iter_mut().enumerate() {
            let key = child
                .get_direct_erased_template(TypeId::of::<UiSceneKey>())
                .and_then(|template| (template as &dyn Any).downcast_ref::<UiSceneKey>())
                .map(|key| key.0.clone())
                .or_else(|| {
                    child
                        .get_direct_erased_template(TypeId::of::<Name>())
                        .and_then(|template| (template as &dyn Any).downcast_ref::<Name>())
                        .map(|name| name.as_str().to_owned())
                })
                .unwrap_or_else(|| index.to_string());
            assert!(
                keys.insert(key.clone()),
                "duplicate UI scene key within one parent: {key}"
            );
            let child_entity =
                self.registry
                    .touch(entity, key.clone(), self.owner, self.frame, || {
                        (self.reserve)()
                    });
            self.prepare(child, entity, child_entity);
        }
    }
}

fn keep_children(
    _writer: &mut BundleWriter,
    _components: &mut ComponentsRegistrator,
    _capacity: usize,
) {
}
