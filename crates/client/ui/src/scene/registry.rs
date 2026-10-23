use super::properties::UiSceneProperties;
use super::property::PropertyValue;
use bevy::ecs::component::ComponentId;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// A local identity within a parent. Dynamic lists use their domain identity here.
#[derive(Component, Clone, Default, Eq, PartialEq)]
pub struct UiSceneKey(pub String);

pub(super) struct MountedEntity {
    pub entity: Entity,
    pub owner: Entity,
    pub frame: u64,
    pub retained: bool,
    pub authored: Vec<ComponentId>,
    pub next_authored: HashSet<ComponentId>,
    pub properties: HashMap<TypeId, PropertyValue>,
    pub runtime: HashMap<TypeId, Option<PropertyValue>>,
}

#[derive(Resource, Default)]
pub struct UiSceneRegistry {
    pub(super) entities: HashMap<(Entity, String), MountedEntity>,
    pub(super) frames: HashMap<Entity, u64>,
    pub(super) active: Option<(Entity, u64)>,
    pub(super) slots: HashMap<(Entity, String), u32>,
    pub(super) children: HashMap<Entity, Vec<Entity>>,
    pub(super) keys: HashMap<Entity, (Entity, String)>,
}

impl UiSceneRegistry {
    pub(super) fn touch(
        &mut self,
        parent: Entity,
        key: String,
        owner: Entity,
        frame: u64,
        create: impl FnOnce() -> Entity,
    ) -> Entity {
        let identity = (parent, key);
        let entry = self
            .entities
            .entry(identity.clone())
            .or_insert_with(|| MountedEntity {
                entity: create(),
                owner,
                frame,
                retained: false,
                authored: Vec::new(),
                next_authored: HashSet::new(),
                properties: HashMap::new(),
                runtime: HashMap::new(),
            });
        if entry.frame != frame {
            entry.next_authored.clear();
            entry.frame = frame;
        }
        entry.retained = false;
        self.keys.insert(entry.entity, identity);
        entry.entity
    }

    pub(super) fn scope(&self, parent: Entity) -> (Entity, u64) {
        self.active
            .or_else(|| {
                self.keys
                    .get(&parent)
                    .and_then(|key| self.entities.get(key))
                    .map(|entry| (entry.owner, entry.frame))
            })
            .unwrap_or((parent, 0))
    }

    pub(super) fn finish(&mut self, world: &mut World, owner: Entity, frame: u64) {
        let expired: Vec<_> = self
            .entities
            .values()
            .filter(|entry| entry.owner == owner && entry.frame != frame)
            .map(|entry| entry.entity)
            .collect();
        for entity in expired {
            if world.entities().contains(entity) {
                world.despawn(entity);
            }
        }
        if world
            .get_resource::<InputFocus>()
            .and_then(InputFocus::get)
            .is_some_and(|entity| !world.entities().contains(entity))
        {
            world.resource_mut::<InputFocus>().clear();
        }
        self.entities
            .retain(|_, entry| world.entities().contains(entry.entity));
        self.keys
            .retain(|entity, _| world.entities().contains(*entity));
        world.resource_scope(|world, properties: Mut<UiSceneProperties>| {
            for entry in self
                .entities
                .values_mut()
                .filter(|entry| entry.owner == owner && entry.frame == frame && !entry.retained)
            {
                properties.reconcile(world, entry);
                let mut required = HashSet::new();
                for id in &entry.next_authored {
                    if let Some(info) = world.components().get_info(*id) {
                        required.extend(info.required_components().iter_ids());
                    }
                }
                let mut omitted_required = Vec::new();
                for id in &entry.authored {
                    if !entry.next_authored.contains(id) && required.contains(id) {
                        omitted_required.push(*id);
                    }
                    if !entry.next_authored.contains(id) && !required.contains(id) {
                        world.entity_mut(entry.entity).remove_by_id(*id);
                        if let Some(type_id) = world
                            .components()
                            .get_info(*id)
                            .and_then(|info| info.type_id())
                        {
                            entry.properties.remove(&type_id);
                        }
                    }
                }
                properties.reset_required(world, entry, &omitted_required);
                entry.authored = entry.next_authored.iter().copied().collect();
            }
        });
        let managed: HashSet<_> = self
            .entities
            .values()
            .filter(|entry| entry.owner == owner)
            .map(|entry| entry.entity)
            .collect();
        for (parent, children) in self.children.drain() {
            if !world.entities().contains(parent) {
                continue;
            }
            let mut desired: Vec<_> = children
                .into_iter()
                .filter(|child| world.entities().contains(*child))
                .collect();
            let previous = world
                .get::<Children>(parent)
                .map(|children| children.iter().collect::<Vec<_>>())
                .unwrap_or_default();
            let authored: HashSet<_> = desired.iter().copied().collect();
            desired.extend(
                previous
                    .iter()
                    .filter(|child| !managed.contains(child) && !authored.contains(child))
                    .copied(),
            );
            if previous != desired {
                world.entity_mut(parent).replace_children(&desired);
            }
        }
    }

    pub(super) fn child(&mut self, parent: Entity, child: Entity) {
        let children = self.children.entry(parent).or_default();
        if !children.contains(&child) {
            children.push(child);
        }
    }

    pub(super) fn key_for(&self, entity: Entity) -> Option<(Entity, String)> {
        self.keys.get(&entity).cloned()
    }
}
