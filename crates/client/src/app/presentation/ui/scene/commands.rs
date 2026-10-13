use super::entity::UiEntityCommands;
use super::reconcile::{Mount, prepare};
use super::registry::UiSceneRegistry;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::scene::ScenePatch;
use std::any::TypeId;
use std::ops::{Deref, DerefMut};
use std::panic::Location;

/// Scene construction and entity ownership share one boundary; ordinary commands
/// remain available for non-visual effects such as audio.
#[derive(SystemParam)]
pub(crate) struct UiComposer<'w, 's> {
    commands: Commands<'w, 's>,
    pub(super) registry: ResMut<'w, UiSceneRegistry>,
    alive: Query<'w, 's, Entity>,
    assets: Res<'w, AssetServer>,
    patches: Res<'w, Assets<ScenePatch>>,
}

impl<'w, 's> UiComposer<'w, 's> {
    pub(super) fn raw_commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }

    pub(crate) fn begin(&mut self, owner: Entity) {
        self.registry
            .entities
            .retain(|_, entry| self.alive.contains(entry.entity));
        self.registry
            .keys
            .retain(|entity, _| self.alive.contains(*entity));
        self.registry.slots.clear();
        let frame = self.registry.frames.entry(owner).or_default();
        *frame += 1;
        let frame = *frame;
        self.registry.active = Some((owner, frame));
        // Root page markers participate in the same omission cleanup as children.
        self.registry
            .touch(owner, "$root".into(), owner, frame, || owner);
    }

    pub(crate) fn finish(&mut self) {
        let Some((owner, frame)) = self.registry.active.take() else {
            return;
        };
        self.commands.queue(move |world: &mut World| {
            world.resource_scope(|world, mut registry: Mut<UiSceneRegistry>| {
                registry.finish(world, owner, frame);
            });
        });
    }

    pub(crate) fn entity_for(&mut self, parent: Entity, key: impl Into<String>) -> Entity {
        let (owner, frame) = self.registry.scope(parent);
        self.registry.touch(parent, key.into(), owner, frame, || {
            self.commands.spawn_empty().id()
        })
    }

    pub(crate) fn entity(&mut self, entity: Entity) -> UiEntityCommands<'_, 'w, 's> {
        UiEntityCommands {
            composer: self,
            entity,
        }
    }

    pub(crate) fn declared<T: Component>(&self, entity: Entity) -> Option<&T> {
        let key = self.registry.keys.get(&entity)?;
        self.registry
            .entities
            .get(key)?
            .properties
            .get(&TypeId::of::<T>())?
            .downcast_ref()
    }

    /// A leaf bundle uses the same identity and ownership as a scene hierarchy.
    #[track_caller]
    pub(crate) fn child<B: Bundle>(&mut self, parent: Entity, bundle: B) -> Entity {
        let key = self.slot_key(parent);
        self.keyed_child(parent, key, bundle)
    }

    pub(crate) fn keyed_child<B: Bundle>(
        &mut self,
        parent: Entity,
        key: impl Into<String>,
        bundle: B,
    ) -> Entity {
        let entity = if self.registry.active.is_some() {
            self.entity_for(parent, key)
        } else {
            self.commands.spawn_empty().id()
        };
        self.entity(entity).insert(bundle).add_child_to(parent);
        entity
    }

    #[track_caller]
    pub(crate) fn spawn<B: Bundle>(&mut self, bundle: B) -> UiEntityCommands<'_, 'w, 's> {
        assert!(
            self.registry.active.is_none(),
            "visual UI entities need a parent: use child or mount"
        );
        let entity = self.commands.spawn_empty().id();
        let mut commands = self.entity(entity);
        commands.insert(bundle);
        commands
    }

    pub(crate) fn effect<B: Bundle>(&mut self, bundle: B) -> UiEntityCommands<'_, 'w, 's> {
        let entity = self.commands.spawn_empty().id();
        let mut commands = self.entity(entity);
        commands.insert(bundle);
        commands
    }

    pub(crate) fn effect_child<B: Bundle>(&mut self, parent: Entity, bundle: B) -> Entity {
        let entity = self.effect(bundle).id();
        self.entity(parent).add_child(entity);
        entity
    }

    pub(crate) fn mount(
        &mut self,
        parent: Entity,
        key: impl Into<String>,
        scene: impl Scene,
    ) -> Entity {
        let mut key = key.into();
        let (entity, owner, frame) = if let Some((owner, frame)) = self.registry.active {
            (self.entity_for(parent, key.clone()), owner, frame)
        } else {
            // Hover windows and transient effects have their own lifetime. A page
            // refresh must not retire them merely because it did not create them.
            let entity = self.commands.spawn_empty().id();
            key = format!("effect:{}", entity.to_bits());
            self.registry
                .touch(parent, key.clone(), entity, 0, || entity);
            (entity, entity, 0)
        };
        // Reserve the complete hierarchy before queuing application. Callers can
        // immediately ask for a named child without queuing its creation too late.
        let prepared = prepare(
            &mut self.registry,
            &self.assets,
            &self.patches,
            Mount {
                parent,
                entity,
                owner,
                frame,
            },
            scene,
            || self.commands.spawn_empty().id(),
        )
        .expect("inline UI scenes resolve from preloaded handles");
        self.commands
            .queue(move |world: &mut World| prepared.apply(world));
        entity
    }

    #[track_caller]
    pub(crate) fn node(&mut self, parent: Entity, scene: impl Scene) -> Entity {
        let key = self.slot_key(parent);
        self.mount(parent, key, scene)
    }

    #[track_caller]
    fn slot_key(&mut self, parent: Entity) -> String {
        let location = Location::caller();
        let base = format!(
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        let slot = self
            .registry
            .slots
            .entry((parent, base.clone()))
            .or_default();
        let key = format!("{base}:{}", *slot);
        *slot += 1;
        key
    }
}

impl<'w, 's> Deref for UiComposer<'w, 's> {
    type Target = Commands<'w, 's>;

    fn deref(&self) -> &Self::Target {
        &self.commands
    }
}

impl DerefMut for UiComposer<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.commands
    }
}
