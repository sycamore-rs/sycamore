//! Iteration utility components for [view!](crate::view!).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::hash::Hash;

use crate::prelude::*;

/// Props for [`Keyed`].
#[derive(Props, Debug)]
pub struct KeyedProps<T, F, G: GenericNode, K, Key>
where
    F: Fn(T) -> View<G> + 'static,
    K: Fn(&T) -> Key + 'static,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq + 'static,
{
    iterable: ReadSignal<Vec<T>>,
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
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// #[derive(Clone, PartialEq)]
/// struct AnimalInfo {
///     // The name of the animal.
///     name: &'static str,
///     // An unique id to identify the animal.
///     id: u32,
/// }
///
/// # fn App<G: Html>() -> View<G> {
/// let animals = create_signal(vec![
///     AnimalInfo { name: "Dog", id: 1 },
///     AnimalInfo { name: "Cat", id: 2 },
///     AnimalInfo { name: "Fish", id: 3 },
/// ]);
/// view! {
///     ul {
///         Keyed(
///             iterable=*animals,
///             view=|animal| view! {
///                 li { (animal.name) }
///             },
///             key=|animal| animal.id,
///         )
///     }
/// }
/// # }
/// ```
#[component]
pub fn Keyed<G: GenericNode, T, F, K, Key>(props: KeyedProps<T, F, G, K, Key>) -> View<G>
where
    F: Fn(T) -> View<G> + 'static,
    K: Fn(&T) -> Key + 'static,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    let KeyedProps {
        iterable,
        view,
        key,
    } = props;

    let mapped = map_keyed(iterable, view, key);
    View::new_dyn(move || View::new_fragment(mapped.get_clone()))
}

/// Props for [`Indexed`].
#[derive(Props, Debug)]
pub struct IndexedProps<G: GenericNode, T: 'static, F>
where
    F: Fn(T) -> View<G> + 'static,
{
    iterable: ReadSignal<Vec<T>>,
    /// The map function that renders a [`View`] for each element in `iterable`.
    view: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # fn App<G: Html>() -> View<G> {
/// let fib = create_signal(vec![0, 1, 1, 2, 3, 5, 8]);
/// view! {
///     ul {
///         Indexed(
///             iterable=*fib,
///             view=|x| view! {
///                 li { (x) }
///             },
///         )
///     }
/// }
/// # }
/// ```
#[component]
pub fn Indexed<G: GenericNode, T, F>(props: IndexedProps<G, T, F>) -> View<G>
where
    T: Clone + PartialEq,
    F: Fn(T) -> View<G> + 'static,
{
    let IndexedProps { iterable, view } = props;

    let mapped = map_indexed(iterable, view);
    View::new_dyn(move || View::new_fragment(mapped.get_clone()))
}
