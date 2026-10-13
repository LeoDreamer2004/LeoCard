use super::UiComposer;
use bevy::prelude::*;
use std::any::TypeId;

struct MaterialBinding<T: Asset> {
    declared: T,
    handle: Handle<T>,
}

impl UiComposer<'_, '_> {
    /// Keep a material's runtime uniforms until its declared appearance changes.
    pub(crate) fn material<T: Asset + Clone + PartialEq>(
        &mut self,
        entity: Entity,
        assets: &mut Assets<T>,
        declared: T,
    ) -> Handle<T> {
        let Some(entry) = self
            .registry
            .key_for(entity)
            .and_then(|key| self.registry.entities.get_mut(&key))
        else {
            return assets.add(declared);
        };
        let value = entry
            .properties
            .entry(TypeId::of::<MaterialBinding<T>>())
            .or_insert_with(|| {
                Box::new(MaterialBinding {
                    handle: assets.add(declared.clone()),
                    declared: declared.clone(),
                })
            });
        let binding = value
            .downcast_mut::<MaterialBinding<T>>()
            .expect("material binding type");
        if binding.declared != declared {
            *assets
                .get_mut(&binding.handle)
                .expect("mounted material is retained") = declared.clone();
            binding.declared = declared;
        }
        binding.handle.clone()
    }
}
