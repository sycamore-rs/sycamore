//! Iteration utility components for [`template`](crate::template).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;
use std::rc::Rc;

use crate::generic_node::GenericNode;
use crate::prelude::*;
use crate::reactive::ReactiveScope;

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
    pub key: K,
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
    T: Clone + PartialEq,
{
    let KeyedProps {
        iterable,
        template,
        key: key_fn,
    } = props;
    let iterable = Rc::new(iterable);
    let key_fn = Rc::new(key_fn);

    type TemplateValue<T, G> = (ReactiveScope, T, TemplateResult<G>, usize /* index */);

    // A tuple with a value of type `T` and the `TemplateResult` produces by calling
    // `props.template` with the first value.
    let templates: Rc<RefCell<HashMap<Key, TemplateValue<T, G>>>> = Default::default();

    let fragment = G::fragment();

    let marker = G::marker();

    fragment.append_child(&marker);

    create_effect({
        let iterable = Rc::clone(&iterable);
        let key_fn = Rc::clone(&key_fn);
        let templates = Rc::clone(&templates);
        move || {
            // Fast path for empty array. Remove all nodes from DOM in templates.
            if iterable.get().is_empty() {
                for (_, (scope, _value, template, _i)) in templates.borrow_mut().drain() {
                    drop(scope); // destroy old scope
                    for node in &template {
                        node.remove_self()
                    }
                }
                return;
            }

            // Remove old nodes not in iterable.
            {
                let mut templates = templates.borrow_mut();
                let new_keys: HashSet<Key> =
                    iterable.get().iter().map(|item| key_fn(item)).collect();

                let excess_nodes = templates
                    .iter()
                    .filter(|item| new_keys.get(item.0).is_none())
                    .map(|x| (x.0.clone(), (x.1 .2.clone(), x.1 .3)))
                    .collect::<Vec<_>>();

                for template in &excess_nodes {
                    let removed_index = template.1 .1;
                    templates.remove(&template.0);

                    // Offset indexes of other templates by 1.
                    for (_, _, _, i) in templates.values_mut() {
                        if *i > removed_index {
                            *i -= 1;
                        }
                    }
                }

                for template in excess_nodes {
                    for node in template.1 .0 {
                        node.remove_self();
                    }
                }
            }

            struct PreviousData<T> {
                value: T,
                index: usize,
            }

            let previous_values: HashMap<_, PreviousData<T>> = {
                let templates = templates.borrow();
                templates
                    .iter()
                    .map(|x| {
                        (
                            (*x.0).clone(),
                            PreviousData {
                                value: x.1 .1.clone(),
                                index: x.1 .3,
                            },
                        )
                    })
                    .collect()
            };

            // Find values that changed by comparing to previous_values.
            for (i, item) in iterable.get().iter().enumerate() {
                let key = key_fn(item);

                let previous_value = previous_values.get(&key);

                if previous_value.is_none() {
                    // Create new DOM node.

                    let mut new_template = None;
                    let scope = create_root(|| new_template = Some(template(item.clone())));

                    templates.borrow_mut().insert(
                        key.clone(),
                        (scope, item.clone(), new_template.clone().unwrap(), i),
                    );

                    if let Some(next_item) = iterable.get().get(i + 1) {
                        let templates = templates.borrow();
                        if let Some(next_template) = templates.get(&key_fn(next_item)) {
                            for node in &new_template.unwrap() {
                                next_template.2.first_node().insert_sibling_before(node);
                            }
                        } else {
                            for node in &new_template.unwrap() {
                                marker.insert_sibling_before(node);
                            }
                        }
                    } else {
                        for node in &new_template.unwrap() {
                            marker.insert_sibling_before(node);
                        }
                    }
                } else if match previous_value {
                    Some(prev) => prev.index,
                    _ => unreachable!(),
                } != i
                {
                    // Location changed, move from old location to new location
                    // Node was moved in the DOM. Move node to new index.

                    {
                        let templates = templates.borrow();
                        let template = &templates.get(&key).unwrap().2;

                        if let Some(next_item) = iterable.get().get(i + 1) {
                            let next_node = templates.get(&key_fn(next_item)).unwrap();
                            for node in template {
                                // Move to before next node.
                                next_node.2.first_node().insert_sibling_before(node);
                            }
                        } else {
                            for node in template {
                                marker.insert_sibling_before(node); // Move to end.
                            }
                        }
                    }

                    templates.borrow_mut().get_mut(&key).unwrap().3 = i;
                } else if match previous_value {
                    Some(prev) => &prev.value,
                    _ => unreachable!(),
                } != item
                {
                    // Value changed. Re-render node (with same previous key and index).

                    // Destroy old reactive scope.
                    let mut templates = templates.borrow_mut();
                    let (old_scope, _, _, _) = templates
                        .get_mut(&key)
                        .expect("previous value is different but must be valid");
                    let old_scope = mem::replace(old_scope, ReactiveScope::new() /* placeholder */);
                    drop(old_scope);

                    let mut new_template = None;
                    let scope = create_root(|| new_template = Some(template(item.clone())));

                    let (_, _, old_template, _) = mem::replace(
                        templates.get_mut(&key).unwrap(),
                        (scope, item.clone(), new_template.clone().unwrap(), i),
                    );

                    let parent = old_template.first_node().parent_node().unwrap();

                    for new_node in &new_template.unwrap() {
                        parent.insert_child_before(new_node, Some(old_template.first_node()));
                    }
                    for old_node in &old_template {
                        parent.remove_child(old_node);
                    }
                }
            }
        }
    });

    TemplateResult::new_node(fragment)
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
    type TemplateData<G> = (ReactiveScope, TemplateResult<G>);
    let templates: Rc<RefCell<Vec<TemplateData<G>>>> = Default::default();

    // Previous values for diffing purposes.
    let previous_values = RefCell::new(Vec::new());

    let fragment = G::fragment();

    let marker = G::marker();

    fragment.append_child(&marker);

    create_effect({
        let templates = Rc::clone(&templates);
        move || {
            // Fast path for empty array. Remove all nodes from DOM in templates.
            if props.iterable.get().is_empty() {
                for (scope, template) in templates.borrow_mut().drain(..) {
                    drop(scope); // destroy old scope
                    for node in template {
                        node.remove_self();
                    }
                }
                return;
            }

            // Find values that changed by comparing to previous_values.
            for (i, item) in props.iterable.get().iter().enumerate() {
                let previous_values = previous_values.borrow();
                let previous_value = previous_values.get(i);

                if previous_value.is_none() || previous_value.unwrap() != item {
                    // Value changed, re-render item.

                    templates.borrow_mut().get_mut(i).and_then(|(scope, _)| {
                        // destroy old scope
                        let old_scope = mem::replace(scope, ReactiveScope::new() /* placeholder */);
                        drop(old_scope);
                        None::<()>
                    });

                    let mut new_template = None;
                    let scope = create_root(|| new_template = Some((props.template)(item.clone())));

                    if templates.borrow().get(i).is_some() {
                        let old_template = mem::replace(
                            &mut templates.borrow_mut()[i],
                            (scope, new_template.as_ref().unwrap().clone()),
                        );

                        let parent = old_template.1.first_node().parent_node().unwrap();
                        for node in &new_template.unwrap() {
                            parent.insert_child_before(node, Some(old_template.1.first_node()));
                        }
                        for old_node in &old_template.1 {
                            parent.remove_child(old_node);
                        }
                    } else {
                        debug_assert!(templates.borrow().len() == i, "pushing new value scenario");

                        templates
                            .borrow_mut()
                            .push((scope, new_template.as_ref().unwrap().clone()));

                        for node in &new_template.unwrap() {
                            marker.insert_sibling_before(node);
                        }
                    }
                }
            }

            if templates.borrow().len() > props.iterable.get().len() {
                let mut templates = templates.borrow_mut();
                let excess_nodes = templates.drain(props.iterable.get().len()..);

                for template in excess_nodes {
                    for node in &template.1 {
                        node.remove_self();
                    }
                }
            }

            *previous_values.borrow_mut() = (*props.iterable.get()).clone();
        }
    });

    TemplateResult::new_node(fragment)
}
