use std::sync::atomic::{AtomicUsize, Ordering};

/// Create a unique ID that's stable between SSR and hydration.
///
/// Returns the current component ID and the local hook index in hydration contexts,
/// or a global hook index outside of a hydration context.
/// Use this to generate stable IDs for things like accessiblity attributes.
///
/// # Example
///
/// ```
/// # use sycamore::prelude::create_unique_id;
/// let (component_id, id) = create_unique_id();
/// ```
pub fn create_unique_id() -> (Option<usize>, usize) {
    static LAST_COMPONENT_ID: AtomicUsize = AtomicUsize::new(0);
    static CURRENT_HOOK_ID: AtomicUsize = AtomicUsize::new(0);

    #[cfg(feature = "hydrate")]
    if let Some((component_id, _)) = crate::hydrate::get_current_id() {
        return if component_id == LAST_COMPONENT_ID.swap(component_id, Ordering::Relaxed) {
            (
                Some(component_id),
                CURRENT_HOOK_ID.fetch_add(1, Ordering::Relaxed),
            )
        } else {
            CURRENT_HOOK_ID.store(1, Ordering::Relaxed);
            (Some(component_id), 0)
        };
    }

    // Unhydrated IDs don't have a component ID
    static CURRENT_ID: AtomicUsize = AtomicUsize::new(0);
    let id = CURRENT_ID.fetch_add(1, Ordering::Relaxed);
    (None, id)
}

#[cfg(test)]
mod test {
    use super::create_unique_id;

    #[test]
    pub fn ids_increment() {
        let (_, id1) = create_unique_id();
        let (_, id2) = create_unique_id();
        let (_, id3) = create_unique_id();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }

    #[cfg(feature = "hydrate")]
    #[test]
    pub fn id_is_stable() {
        use crate::hydrate::{hydrate_component, with_hydration_context};

        let (id1, id2, id3) = with_hydration_context(|| {
            let id1 = create_unique_id();
            let (id2, id3) = hydrate_component(|| (create_unique_id(), create_unique_id()));
            (id1, id2, id3)
        });

        assert_eq!(id1, (Some(0), 0));
        assert_eq!(id2, (Some(1), 0));
        assert_eq!(id3, (Some(1), 1));

        with_hydration_context(|| {
            assert_eq!(id1, create_unique_id());
            hydrate_component(|| {
                assert_eq!(id2, create_unique_id());
                assert_eq!(id3, create_unique_id());
            });
        });
    }
}
