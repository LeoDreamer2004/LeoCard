use super::*;
use std::collections::HashSet;
#[test]
fn registry_ids_and_criteria_are_unique_and_requirements_reference_real_criteria() {
    let mut ids = HashSet::new();
    for definition in ACHIEVEMENT_REGISTRY {
        assert!(ids.insert(definition.id));
        assert!(definition.id.starts_with("leocard:"));
        let criteria = definition
            .criteria
            .iter()
            .map(|criterion| criterion.id)
            .collect::<HashSet<_>>();
        assert_eq!(criteria.len(), definition.criteria.len());
        assert!(!definition.requirements.is_empty());
        assert!(
            definition
                .criteria
                .iter()
                .all(|criterion| criterion.target > 0)
        );
        for group in definition.requirements {
            assert!(!group.is_empty());
            assert!(group.iter().all(|id| criteria.contains(id)));
        }
    }
}
