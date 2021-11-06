//! Iteration utility components for [view!](crate::view!).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::hash::Hash;

use crate::generic_node::GenericNode;
use crate::prelude::*;
use crate::reactive::{map_indexed, map_keyed};

/// Props for [`Keyed`].
pub struct KeyedProps<T: 'static, F, G: GenericNode, K, Key>
where
    F: Fn(T) -> View<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
    pub key: K,
}

/// Keyed iteration. Use this instead of directly rendering an array of [`View`]s.
/// Using this will minimize re-renders instead of re-rendering every single node on every state
/// change.
///
/// For non keyed iteration, see [`Indexed`].
///
/// # Example
/// ```no_run
/// use sycamore::prelude::*;
///
/// let count = Signal::new(vec![1, 2]);
///
/// let node = view! {
///     Keyed(KeyedProps {
///         iterable: count.handle(),
///         template: |item| view! {
///             li { (item) }
///         },
///         key: |item| *item,
///     })
/// };
/// # let _ : View<DomNode> = node;
/// ```
#[component(Keyed<G>)]
pub fn keyed<T: 'static, F: 'static, K: 'static, Key: 'static>(
    props: KeyedProps<T, F, G, K, Key>,
) -> View<G>
where
    F: Fn(T) -> View<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + Eq,
{
    let KeyedProps {
        iterable,
        template,
        key,
    } = props;

    let mut mapped = map_keyed(iterable, move |x| template(x.clone()), key);
    View::new_dyn(move || View::new_fragment(mapped()))
}

/// Props for [`Indexed`].
pub struct IndexedProps<T: 'static, F, G: GenericNode>
where
    F: Fn(T) -> View<G>,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
///
/// # Example
/// ```no_run
/// use sycamore::prelude::*;
///
/// let count = Signal::new(vec![1, 2]);
///
/// let node = view! {
///     Indexed(IndexedProps {
///         iterable: count.handle(),
///         template: |item| view! {
///             li { (item) }
///         },
///     })
/// };
/// # let _ : View<DomNode> = node;
/// ```
#[component(Indexed<G>)]
pub fn indexed<T: 'static, F: 'static>(props: IndexedProps<T, F, G>) -> View<G>
where
    T: Clone + PartialEq,
    F: Fn(T) -> View<G>,
{
    let IndexedProps { iterable, template } = props;

    let mut mapped = map_indexed(iterable, move |x| template(x.clone()));
    View::new_dyn(move || View::new_fragment(mapped()))
}
