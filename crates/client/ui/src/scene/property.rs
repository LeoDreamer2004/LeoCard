use super::registry::MountedEntity;
use bevy::ecs::change_detection::Tick;
use bevy::ecs::component::{ComponentId, Mutable};
use bevy::prelude::*;
use std::any::{Any, TypeId};

pub(super) type PropertyValue = Box<dyn Any + Send + Sync>;

pub(super) trait Property: Send + Sync {
    fn capture(&self, world: &World, entity: Entity) -> Option<PropertyValue>;
    fn remember(
        &self,
        world: &World,
        entry: &mut MountedEntity,
        old: Option<PropertyValue>,
        authored: &[ComponentId],
    );
    fn reconcile(&self, world: &mut World, entry: &mut MountedEntity);
    fn reset_required(&self, world: &mut World, entry: &mut MountedEntity, omitted: &[ComponentId]);
}

pub(super) struct ComponentProperty<T> {
    pub equal: fn(&T, &T) -> bool,
}

impl<T: Component<Mutability = Mutable> + Clone + Default> Property for ComponentProperty<T> {
    fn capture(&self, world: &World, entity: Entity) -> Option<PropertyValue> {
        let entity = world.get_entity(entity).ok()?;
        let value = entity.get_ref::<T>()?;
        Some(Box::new(((*value).clone(), value.last_changed())))
    }

    fn remember(
        &self,
        world: &World,
        entry: &mut MountedEntity,
        old: Option<PropertyValue>,
        authored: &[ComponentId],
    ) {
        if world
            .component_id::<T>()
            .is_some_and(|id| authored.contains(&id))
        {
            // Multiple declarations of a component form one final value. Retain
            // its runtime state from before the first write in this composition.
            entry.runtime.entry(TypeId::of::<T>()).or_insert(old);
        }
    }

    fn reconcile(&self, world: &mut World, entry: &mut MountedEntity) {
        let type_id = TypeId::of::<T>();
        let Some(old) = entry.runtime.remove(&type_id) else {
            return;
        };
        let Some(component) = world.get::<T>(entry.entity).cloned() else {
            entry.properties.remove(&type_id);
            return;
        };
        let previous = entry
            .properties
            .get(&type_id)
            .and_then(|value| value.downcast_ref::<T>());
        if previous.is_some_and(|previous| (self.equal)(previous, &component))
            && let Some(old) = old.and_then(|old| old.downcast::<(T, Tick)>().ok())
        {
            let (value, tick) = *old;
            let mut current = world
                .get_mut::<T>(entry.entity)
                .expect("authored component");
            *current.bypass_change_detection() = value;
            current.set_last_changed(tick);
        }
        entry.properties.insert(type_id, Box::new(component));
    }

    fn reset_required(
        &self,
        world: &mut World,
        entry: &mut MountedEntity,
        omitted: &[ComponentId],
    ) {
        if !world
            .component_id::<T>()
            .is_some_and(|id| omitted.contains(&id))
        {
            return;
        }
        let type_id = TypeId::of::<T>();
        let default = T::default();
        if entry
            .properties
            .get(&type_id)
            .and_then(|value| value.downcast_ref::<T>())
            .is_some_and(|previous| !(self.equal)(previous, &default))
        {
            world.entity_mut(entry.entity).insert(default.clone());
        }
        // Required components cannot be removed. Omitting an explicit value
        // resumes the default declaration, including Visibility::Inherited.
        entry.properties.insert(type_id, Box::new(default));
    }
}
