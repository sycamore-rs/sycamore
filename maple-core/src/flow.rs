//! Iteration utility components for [`template`](crate::template).
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_.
//! Use the [`Keyed`] and [`Indexed`] utility components respectively.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;
use std::rc::Rc;

use wasm_bindgen::*;
use web_sys::HtmlElement;

use crate::generic_node::GenericNode;
use crate::prelude::*;
use crate::reactive::Owner;

/// Props for [`Keyed`].
pub struct KeyedProps<T: 'static, F, G: GenericNode, K, Key>
where
    F: Fn(T) -> TemplateResult<G>,
    K: Fn(&T) -> Key,
    Key: Hash + Eq,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
    pub key: K,
}

/// Keyed iteration. Use this instead of directly rendering an array of [`TemplateResult`]s.
/// Using this will minimize re-renders instead of re-rendering every single node on every state change.
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
/// ```
pub fn Keyed<T, F: 'static, G: GenericNode, K: 'static, Key: 'static>(
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

    type TemplateValue<T, G> = (Owner, T, TemplateResult<G>, usize /* index */);

    // A tuple with a value of type `T` and the `TemplateResult` produces by calling `props.template` with the first value.
    let templates: Rc<RefCell<HashMap<Key, TemplateValue<T, G>>>> =
        Rc::new(RefCell::new(HashMap::new()));

    let fragment = G::fragment();

    let marker = G::marker();

    fragment.append_child(&marker);

    create_effect({
        let iterable = Rc::clone(&iterable);
        let key_fn = Rc::clone(&key_fn);
        let templates = Rc::clone(&templates);
        let marker = marker.clone();
        move || {
            // Fast path for empty array. Remove all nodes from DOM in templates.
            if iterable.get().is_empty() {
                for (_, (owner, _value, template, _i)) in templates.borrow_mut().drain() {
                    drop(owner); // destroy owner
                    template.node.remove_self();
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

                for node in &excess_nodes {
                    let removed_index = node.1 .1;
                    templates.remove(&node.0);

                    // Offset indexes of other templates by 1.
                    for (_, _, _, i) in templates.values_mut() {
                        if *i > removed_index {
                            *i -= 1;
                        }
                    }
                }

                for node in excess_nodes {
                    node.1 .0.node.remove_self();
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
                    let owner = create_root(|| new_template = Some(template(item.clone())));

                    templates.borrow_mut().insert(
                        key.clone(),
                        (owner, item.clone(), new_template.clone().unwrap(), i),
                    );

                    if let Some(next_item) = iterable.get().get(i + 1) {
                        let templates = templates.borrow();
                        if let Some(next_node) = templates.get(&key_fn(next_item)) {
                            next_node
                                .2
                                .node
                                .insert_before_self(&new_template.unwrap().node);
                        } else {
                            marker.insert_before_self(&new_template.unwrap().node);
                        }
                    } else {
                        marker.insert_before_self(&new_template.unwrap().node);
                    }
                } else if match previous_value {
                    Some(prev) => prev.index,
                    _ => unreachable!(),
                } != i
                {
                    // Location changed, move from old location to new location
                    // Node was moved in the DOM. Move node to new index.

                    let node = templates.borrow().get(&key).unwrap().2.node.clone();

                    if let Some(next_item) = iterable.get().get(i + 1) {
                        let templates = templates.borrow();
                        let next_node = templates.get(&key_fn(next_item)).unwrap();
                        next_node.2.node.insert_before_self(&node); // Move to before next node
                    } else {
                        marker.insert_before_self(&node); // Move to end.
                    }

                    templates.borrow_mut().get_mut(&key).unwrap().3 = i;
                } else if match previous_value {
                    Some(prev) => &prev.value,
                    _ => unreachable!(),
                } != item
                {
                    // Value changed. Re-render node (with same previous key and index).

                    // Destroy old template owner.
                    let mut templates = templates.borrow_mut();
                    let (old_owner, _, _, _) = templates
                        .get_mut(&key)
                        .expect("previous value is different but must be valid");
                    let old_owner = mem::replace(old_owner, Owner::new() /* placeholder */);
                    drop(old_owner);

                    let mut new_template = None;
                    let owner = create_root(|| new_template = Some(template(item.clone())));

                    let (_, _, old_node, _) = mem::replace(
                        templates.get_mut(&key).unwrap(),
                        (owner, item.clone(), new_template.clone().unwrap(), i),
                    );

                    let parent = old_node.node.parent_node().unwrap();
                    parent
                        .replace_child(&new_template.unwrap().node, &old_node.node)
                        .unwrap();
                }
            }
        }
    });

    for item in iterable.get().iter() {
        let key = key_fn(item);
        let template = templates.borrow().get(&key).unwrap().2.clone();

        marker.insert_sibling_before(&template.node);
    }

    TemplateResult::new(fragment.into())
}

/// Props for [`Indexed`].
pub struct IndexedProps<T: 'static, F, G: GenericNode>
where
    F: Fn(T) -> TemplateResult<G>,
{
    pub iterable: StateHandle<Vec<T>>,
    pub template: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of [`TemplateResult`]s.
/// Using this will minimize re-renders instead of re-rendering every single node on every state change.
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
/// ```
pub fn Indexed<T, F: 'static, G: GenericNode>(props: IndexedProps<T, F, G>) -> TemplateResult<G>
where
    T: Clone + PartialEq,
    F: Fn(T) -> TemplateResult<G>,
{
    let templates: Rc<RefCell<Vec<(Owner, TemplateResult<G>)>>> = Rc::new(RefCell::new(Vec::new()));

    // Previous values for diffing purposes.
    let previous_values = RefCell::new(Vec::new());

    let fragment = G::fragment();

    let marker = G::empty();

    fragment.append_child(&marker);

    create_effect({
        let templates = Rc::clone(&templates);
        let marker = marker.clone();
        move || {
            // Fast path for empty array. Remove all nodes from DOM in templates.
            if props.iterable.get().is_empty() {
                for (owner, template) in templates.borrow_mut().drain(..) {
                    drop(owner); // destroy owner
                    template.node.unchecked_into::<HtmlElement>().remove();
                }
                return;
            }

            // Find values that changed by comparing to previous_values.
            for (i, item) in props.iterable.get().iter().enumerate() {
                let previous_values = previous_values.borrow();
                let previous_value = previous_values.get(i);

                if previous_value.is_none() || previous_value.unwrap() != item {
                    // Value changed, re-render item.

                    templates.borrow_mut().get_mut(i).and_then(|(owner, _)| {
                        // destroy old owner
                        let old_owner = mem::replace(owner, Owner::new() /* placeholder */);
                        drop(old_owner);
                        None::<()>
                    });

                    let mut new_template = None;
                    let owner = create_root(|| new_template = Some((props.template)(item.clone())));

                    if templates.borrow().get(i).is_some() {
                        let old_node = mem::replace(
                            &mut templates.borrow_mut()[i],
                            (owner, new_template.as_ref().unwrap().clone()),
                        );

                        let parent = old_node.1.node.parent_node().unwrap();
                        parent.replace_child(&new_template.unwrap().node, &old_node.1.node);
                    } else {
                        debug_assert!(templates.borrow().len() == i, "pushing new value scenario");

                        templates
                            .borrow_mut()
                            .push((owner, new_template.as_ref().unwrap().clone()));

                        marker.insert_sibling_before(&new_template.unwrap().node);
                    }
                }
            }

            if templates.borrow().len() > props.iterable.get().len() {
                let mut templates = templates.borrow_mut();
                let excess_nodes = templates.drain(props.iterable.get().len()..);

                for node in excess_nodes {
                    node.1.node.remove_self();
                }
            }

            *previous_values.borrow_mut() = (*props.iterable.get()).clone();
        }
    });

    for template in templates.borrow().iter() {
        marker.insert_sibling_before(&template.1.node);
    }

    TemplateResult::new(fragment.into())
}
