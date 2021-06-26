//! Iteration utility components for [`template`](crate::template!).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::hash::Hash;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::prelude::*;
use crate::rx::{map_indexed, map_keyed};

/// Props for [`Keyed`].
pub struct KeyedProps<T: 'static, F, G: GenericNode, K, Key>
where
    F: Fn(T) -> Template<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
    pub key: K,
}

/// Keyed iteration. Use this instead of directly rendering an array of [`Template`]s.
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
/// let node = template! {
///     Keyed(KeyedProps {
///         iterable: count.handle(),
///         template: |item| template! {
///             li { (item) }
///         },
///         key: |item| *item,
///     })
/// };
/// # let _ : Template<DomNode> = node;
/// ```
#[component(Keyed<G>)]
pub fn keyed<T: 'static, F: 'static, K: 'static, Key: 'static>(
    props: KeyedProps<T, F, G, K, Key>,
) -> Template<G>
where
    F: Fn(T) -> Template<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + Eq + Hash,
{
    let KeyedProps {
        iterable,
        template,
        key,
    } = props;
    let template = Rc::new(template);

    let mut mapped = map_keyed(
        iterable,
        {
            let template = Rc::clone(&template);
            move |x| template(x.clone())
        },
        key,
    );
    Template::new_lazy(move || Template::new_fragment(mapped()))
}

/// Props for [`Indexed`].
pub struct IndexedProps<T: 'static, F, G: GenericNode>
where
    F: Fn(T) -> Template<G>,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`Template`]s. Using this will minimize re-renders instead of re-rendering every single
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
/// let node = template! {
///     Indexed(IndexedProps {
///         iterable: count.handle(),
///         template: |item| template! {
///             li { (item) }
///         },
///     })
/// };
/// # let _ : Template<DomNode> = node;
/// ```
#[component(Indexed<G>)]
pub fn indexed<T: 'static, F: 'static>(props: IndexedProps<T, F, G>) -> Template<G>
where
    T: Clone + PartialEq,
    F: Fn(T) -> Template<G>,
{
    let IndexedProps { iterable, template } = props;
    let template = Rc::new(template);

    let mut mapped = map_indexed(iterable, {
        let template = Rc::clone(&template);
        move |x| template(x.clone())
    });
    Template::new_lazy(move || Template::new_fragment(mapped()))
}
