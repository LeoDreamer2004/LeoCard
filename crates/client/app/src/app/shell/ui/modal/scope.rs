use super::{ModalDismiss, ModalInputBlocker};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::ComputedStackIndex;

#[derive(SystemParam)]
pub(crate) struct ModalInputScope<'w, 's> {
    blockers: Query<
        'w,
        's,
        (
            Entity,
            &'static ComputedStackIndex,
            &'static InheritedVisibility,
            Option<&'static ModalDismiss>,
        ),
        With<ModalInputBlocker>,
    >,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl ModalInputScope<'_, '_> {
    pub(crate) fn top(&self) -> Option<(Entity, Option<&ModalDismiss>)> {
        self.blockers
            .iter()
            .filter(|(_, _, visibility, _)| visibility.get())
            .max_by_key(|(_, order, _, _)| order.0)
            .map(|(entity, _, _, dismiss)| (entity, dismiss))
    }

    pub(crate) fn allows(&self, target: Entity) -> bool {
        // Raw wheel messages reach every reader, regardless of picking.
        self.top().is_none_or(|(modal, _)| {
            modal == target
                || self
                    .parents
                    .iter_ancestors(target)
                    .any(|entity| entity == modal)
        })
    }
}
