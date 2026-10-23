use super::UiComposer;
use bevy::prelude::*;
use std::any::TypeId;

#[cfg(test)]
mod tests;

struct SubtreeRevision<R>(R);

impl UiComposer<'_, '_> {
    /// Retain an unchanged subtree, including its live animation and scroll state.
    /// The caller's revision must cover every input used to build its contents.
    pub fn cached_child<R: PartialEq + Send + Sync + 'static>(
        &mut self,
        parent: Entity,
        key: impl Into<String>,
        revision: R,
        bundle: impl Bundle,
    ) -> (Entity, bool) {
        let entity = self.entity_for(parent, key);
        let identity = self.registry.key_for(entity).expect("mounted subtree");
        let entry = self.registry.entities.get_mut(&identity).expect("subtree");
        let cached = entry
            .properties
            .get(&TypeId::of::<SubtreeRevision<R>>())
            .and_then(|value| value.downcast_ref::<SubtreeRevision<R>>())
            .is_some_and(|previous| previous.0 == revision)
            && self.alive.contains(entity);
        entry.properties.insert(
            TypeId::of::<SubtreeRevision<R>>(),
            Box::new(SubtreeRevision(revision)),
        );
        if cached {
            let (owner, frame) = self.registry.scope(parent);
            for child in [entity]
                .into_iter()
                .chain(self.children.iter_descendants(entity))
            {
                if let Some(entry) = self
                    .registry
                    .key_for(child)
                    .and_then(|key| self.registry.entities.get_mut(&key))
                    && entry.owner == owner
                {
                    entry.frame = frame;
                    if child == entity {
                        entry.next_authored.extend(entry.authored.iter().copied());
                    } else {
                        entry.retained = true;
                    }
                }
            }
        }
        self.entity(entity).insert(bundle).add_child_to(parent);
        (entity, cached)
    }
}
