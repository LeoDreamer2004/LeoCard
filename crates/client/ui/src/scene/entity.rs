use super::commands::UiComposer;
use super::properties::author;
use super::registry::UiSceneRegistry;
use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use std::marker::PhantomData;

/// Additional components belong to the same authored frame as the BSN scene.
pub struct UiEntityCommands<'a, 'w, 's> {
    pub(super) composer: &'a mut UiComposer<'w, 's>,
    pub(super) entity: Entity,
}

pub struct UiComponentEntry<'a, 'w, 's, T> {
    composer: &'a mut UiComposer<'w, 's>,
    entity: Entity,
    marker: PhantomData<T>,
}

fn mark_authored<T: Component>(world: &mut World, entity: Entity) {
    let Some(id) = world.component_id::<T>() else {
        return;
    };
    let mut registry = world.resource_mut::<UiSceneRegistry>();
    if let Some(key) = registry.key_for(entity) {
        let entry = registry.entities.get_mut(&key).expect("mounted entity");
        entry.retained = false;
        entry.next_authored.insert(id);
    }
}

impl<T: Component<Mutability = Mutable>> UiComponentEntry<'_, '_, '_, T> {
    pub fn or_insert(&mut self, value: T) {
        let entity = self.entity;
        let composing = self.composer.registry.active.is_some();
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                let Ok(mut target) = world.get_entity_mut(entity) else {
                    return;
                };
                target.entry::<T>().or_insert(value);
                if composing {
                    mark_authored::<T>(world, entity);
                }
            });
    }

    pub fn and_modify(&mut self, modify: impl FnOnce(Mut<T>) + Send + Sync + 'static) {
        let entity = self.entity;
        let composing = self.composer.registry.active.is_some();
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                if let Some(component) = world.get_mut::<T>(entity) {
                    modify(component);
                }
                if composing {
                    mark_authored::<T>(world, entity);
                }
            });
    }
}

impl<'a, 'w, 's> UiEntityCommands<'a, 'w, 's> {
    pub fn id(&self) -> Entity {
        self.entity
    }

    pub fn entry<T: Component>(&mut self) -> UiComponentEntry<'_, 'w, 's, T> {
        UiComponentEntry {
            composer: self.composer,
            entity: self.entity,
            marker: PhantomData,
        }
    }

    /// One-shot runtime state owns its removal; composing a page does not reset it.
    pub fn init<T: Component<Mutability = Mutable>>(&mut self, value: T) -> &mut Self {
        let entity = self.entity;
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                if let Ok(mut target) = world.get_entity_mut(entity) {
                    target.entry::<T>().or_insert(value);
                }
            });
        self
    }

    pub fn insert<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        let entity = self.entity;
        let composing = self.composer.registry.active.is_some();
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                if !world.entities().contains(entity) {
                    return;
                }
                if !composing {
                    world.entity_mut(entity).insert(bundle);
                    return;
                }
                world.resource_scope(|world, mut registry: Mut<UiSceneRegistry>| {
                    author(world, &mut registry, &[entity], |world| {
                        world.entity_mut(entity).insert(bundle);
                        Ok(())
                    })
                    .expect("bundle insertion cannot fail");
                });
            });
        self
    }

    pub fn add_child(&mut self, child: Entity) -> &mut Self {
        let entity = self.entity;
        let composing = self.composer.registry.active.is_some();
        if composing {
            self.composer.registry.child(entity, child);
        }
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                if !world.entities().contains(entity) || !world.entities().contains(child) {
                    return;
                }
                world.entity_mut(entity).add_child(child);
            });
        self
    }

    pub fn with_child<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        self.composer.child(self.entity, bundle);
        self
    }

    pub(super) fn add_child_to(&mut self, parent: Entity) -> &mut Self {
        self.composer.entity(parent).add_child(self.entity);
        self
    }

    pub fn remove<B: Bundle>(&mut self) -> &mut Self {
        let entity = self.entity;
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                if let Ok(mut entity) = world.get_entity_mut(entity) {
                    entity.remove::<B>();
                }
            });
        self
    }

    pub fn despawn(&mut self) {
        let entity = self.entity;
        self.composer
            .raw_commands()
            .queue(move |world: &mut World| {
                if world.entities().contains(entity) {
                    world.despawn(entity);
                }
            });
    }
}
