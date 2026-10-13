use super::property::{ComponentProperty, Property};
use super::registry::{MountedEntity, UiSceneRegistry};
use bevy::ecs::change_detection::Tick;
use bevy::ecs::component::{ComponentId, Mutable};
use bevy::ecs::error::Result;
use bevy::prelude::*;

/// Keep runtime state when the author-supplied value has not changed. Changes to
/// declared layout/text still propagate, while animation progress and scroll survive.
#[derive(Resource, Default)]
pub(crate) struct UiSceneProperties {
    properties: Vec<Box<dyn Property>>,
}

impl UiSceneProperties {
    pub(crate) fn preserve<T: Component<Mutability = Mutable> + Clone + PartialEq + Default>(
        &mut self,
    ) {
        self.preserve_with::<T>(PartialEq::eq);
    }

    pub(crate) fn preserve_with<T: Component<Mutability = Mutable> + Clone + Default>(
        &mut self,
        equal: fn(&T, &T) -> bool,
    ) {
        self.properties
            .push(Box::new(ComponentProperty::<T> { equal }));
    }

    pub(super) fn reconcile(&self, world: &mut World, entry: &mut MountedEntity) {
        for property in &self.properties {
            property.reconcile(world, entry);
        }
    }

    pub(super) fn reset_required(
        &self,
        world: &mut World,
        entry: &mut MountedEntity,
        omitted: &[ComponentId],
    ) {
        for property in &self.properties {
            property.reset_required(world, entry, omitted);
        }
    }
}

pub(super) fn authored_components(world: &World, entity: Entity, tick: Tick) -> Vec<ComponentId> {
    // Hierarchy is reconciled separately. Relationship hooks can change Children
    // during a scene write, but that does not make it an authored property.
    let children = world.component_id::<Children>();
    let parent = world.component_id::<ChildOf>();
    let entity = world.entity(entity);
    entity
        .archetype()
        .components()
        .iter()
        .copied()
        .filter(|id| Some(*id) != children && Some(*id) != parent)
        .filter(|id| {
            entity
                .get_change_ticks_by_id(*id)
                .is_some_and(|ticks| ticks.changed == tick)
        })
        .collect()
}

pub(crate) fn image_node_eq(left: &ImageNode, right: &ImageNode) -> bool {
    left.color == right.color
        && left.image == right.image
        && left.texture_atlas == right.texture_atlas
        && left.flip_x == right.flip_x
        && left.flip_y == right.flip_y
        && left.rect == right.rect
        && left.image_mode == right.image_mode
        && left.visual_box == right.visual_box
}

/// Apply authored changes with one shared preservation path for bundles and BSN.
pub(super) fn author(
    world: &mut World,
    registry: &mut UiSceneRegistry,
    entities: &[Entity],
    write: impl FnOnce(&mut World) -> Result,
) -> Result {
    world.resource_scope(|world, properties: Mut<UiSceneProperties>| {
        let saved: Vec<_> = entities
            .iter()
            .map(|entity| {
                properties
                    .properties
                    .iter()
                    .map(|property| property.capture(world, *entity))
                    .collect::<Vec<_>>()
            })
            .collect();
        // The dedicated tick distinguishes authored components from runtime state.
        world.increment_change_tick();
        let tick = world.read_change_tick();
        write(world)?;
        for (entity, saved) in entities.iter().zip(saved) {
            let Some(entry) = registry
                .key_for(*entity)
                .and_then(|key| registry.entities.get_mut(&key))
            else {
                continue;
            };
            let authored = authored_components(world, *entity, tick);
            entry.next_authored.extend(authored.iter().copied());
            for (property, old) in properties.properties.iter().zip(saved) {
                property.remember(world, entry, old, &authored);
            }
            if entry.frame == 0 {
                properties.reconcile(world, entry);
            }
        }
        Ok(())
    })
}
