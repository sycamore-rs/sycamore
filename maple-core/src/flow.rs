//! Iteration utility components for [`template`](crate::template).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::hash::Hash;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::prelude::*;
use crate::reactive::{map_indexed, map_keyed};

/// Props for [`Keyed`].
pub struct KeyedProps<T: 'static, F, G: GenericNode, K, Key>
where
    F: Fn(T) -> TemplateResult<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
    pub key: K, // TODO: make key optional
}

/// Keyed iteration. Use this instead of directly rendering an array of [`TemplateResult`]s.
/// Using this will minimize re-renders instead of re-rendering every single node on every state
/// change.
///
/// For non keyed iteration, see [`Indexed`].
///
/// # Example
/// ```no_run
/// use maple_core::prelude::*;
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
/// # let _ : TemplateResult<DomNode> = node;
/// ```
#[component(Keyed<G>)]
pub fn keyed<T: 'static, F: 'static, K: 'static, Key: 'static>(
    props: KeyedProps<T, F, G, K, Key>,
) -> TemplateResult<G>
where
    F: Fn(T) -> TemplateResult<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + Eq + Hash,
{
    let KeyedProps {
        iterable,
        template,
        key: _,
    } = props;
    let template = Rc::new(template);

    let template_result = create_memo(move || {
        let mapped = map_keyed(iterable.clone(), {
            let template = Rc::clone(&template);
            move |x| template(x.clone())
        })();
        TemplateResult::new_fragment((*mapped).clone())
    });
    TemplateResult::new_lazy(move || (*template_result.get()).clone())
}

/// Props for [`Indexed`].
pub struct IndexedProps<T: 'static, F, G: GenericNode>
where
    F: Fn(T) -> TemplateResult<G>,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`TemplateResult`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
///
/// # Example
/// ```no_run
/// use maple_core::prelude::*;
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
/// # let _ : TemplateResult<DomNode> = node;
/// ```
#[component(Indexed<G>)]
pub fn indexed<T: 'static, F: 'static>(props: IndexedProps<T, F, G>) -> TemplateResult<G>
where
    T: Clone + PartialEq,
    F: Fn(T) -> TemplateResult<G>,
{
    let IndexedProps { iterable, template } = props;
    let template = Rc::new(template);

    let template_result = create_memo(move || {
        let mapped = map_indexed(iterable.clone(), {
            let template = Rc::clone(&template);
            move |x| template(x.clone())
        })();
        TemplateResult::new_fragment((*mapped).clone())
    });
    TemplateResult::new_lazy(move || (*template_result.get()).clone())
}
