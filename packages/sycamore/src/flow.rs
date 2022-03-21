//! Iteration utility components for [view!](crate::view!).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::hash::Hash;

use crate::prelude::*;

/// Props for [`Keyed`].
#[derive(Prop)]
pub struct KeyedProps<'a, T, F, G: GenericNode, K, Key>
where
    F: Fn(BoundedScope<'_, 'a>, T) -> View<G> + 'a,
    K: Fn(&T) -> Key + 'a,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    iterable: &'a ReadSignal<Vec<T>>,
    /// The map function that renders a [`View`] for each element in `iterable`.
    view: F,
    /// The key function that assigns each element in `iterable` an unique key.
    key: K,
}

/// Keyed iteration. Use this instead of directly rendering an array of [`View`]s.
/// Using this will minimize re-renders instead of re-rendering every view node on every
/// state change.
///
/// For non keyed iteration, see [`Indexed`].
#[component]
pub fn Keyed<'a, G: GenericNode, T, F, K, Key>(
    cx: Scope<'a>,
    props: KeyedProps<'a, T, F, G, K, Key>,
) -> View<G>
where
    F: Fn(BoundedScope<'_, 'a>, T) -> View<G> + 'a,
    K: Fn(&T) -> Key + 'a,
    Key: Clone + Hash + Eq,
    T: Clone + Eq,
{
    let KeyedProps {
        iterable,
        view,
        key,
    } = props;

    let mapped = map_keyed(cx, iterable, view, key);
    View::new_dyn(cx, || View::new_fragment(mapped.get().as_ref().clone()))
}

/// Props for [`Indexed`].
#[derive(Prop)]
pub struct IndexedProps<'a, G: GenericNode, T, F>
where
    F: Fn(BoundedScope<'_, 'a>, T) -> View<G> + 'a,
{
    iterable: &'a ReadSignal<Vec<T>>,
    /// The map function that renders a [`View`] for each element in `iterable`.
    view: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
#[component]
pub fn Indexed<'a, G: GenericNode, T, F>(cx: Scope<'a>, props: IndexedProps<'a, G, T, F>) -> View<G>
where
    T: Clone + PartialEq,
    F: Fn(BoundedScope<'_, 'a>, T) -> View<G> + 'a,
{
    let IndexedProps { iterable, view } = props;

    let mapped = map_indexed(cx, iterable, view);
    View::new_dyn(cx, || View::new_fragment(mapped.get().as_ref().clone()))
}
