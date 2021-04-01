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
use web_sys::{Element, HtmlElement};

use crate::internal::append;
use crate::prelude::*;
use crate::reactive::Owner;

/// Props for [`Keyed`].
pub struct KeyedProps<T: 'static, F, K, Key>
where
    F: Fn(T) -> TemplateResult,
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
pub fn Keyed<T, F: 'static, K: 'static, Key: 'static>(
    props: KeyedProps<T, F, K, Key>,
) -> TemplateResult
where
    F: Fn(T) -> TemplateResult,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq + std::fmt::Debug,
    T: Clone + PartialEq + std::fmt::Debug,
{
    let KeyedProps {
        iterable,
        template,
        key: key_fn,
    } = props;
    let iterable = Rc::new(iterable);
    let key_fn = Rc::new(key_fn);

    type TemplateValue<T> = (Owner, T, TemplateResult, usize /* index */);

    // A tuple with a value of type `T` and the `TemplateResult` produces by calling `props.template` with the first value.
    let templates: Rc<RefCell<HashMap<Key, TemplateValue<T>>>> =
        Rc::new(RefCell::new(HashMap::new()));

    let fragment = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_document_fragment();

    let marker = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_comment("");

    append(&fragment, &marker);

    create_effect({
        let iterable = Rc::clone(&iterable);
        let key_fn = Rc::clone(&key_fn);
        let templates = Rc::clone(&templates);
        let marker = marker.clone();
        move || {
            #[derive(Debug)]
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

            web_sys::console::log_1(&format!("iterable changed: {:?}", iterable).into());
            web_sys::console::log_1(&format!("previous_values: {:?}", previous_values).into());

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
                                .unchecked_ref::<HtmlElement>()
                                .before_with_node_1(&new_template.unwrap().node)
                                .unwrap();
                        } else {
                            marker
                                .before_with_node_1(&new_template.unwrap().node)
                                .unwrap();
                        }
                    } else {
                        marker
                            .before_with_node_1(&new_template.unwrap().node)
                            .unwrap();
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
                        next_node
                            .2
                            .node
                            .unchecked_ref::<HtmlElement>()
                            .before_with_node_1(&node)
                            .unwrap(); // Move to before next node
                    } else {
                        marker.before_with_node_1(&node).unwrap(); // Move to end.
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

            if templates.borrow().len() > iterable.get().len() {
                // remove extra templates

                let mut templates = templates.borrow_mut();
                let new_keys: HashSet<Key> =
                    iterable.get().iter().map(|item| key_fn(item)).collect();

                let excess_nodes = templates
                    .iter()
                    .filter(|item| new_keys.get(item.0).is_none())
                    .map(|x| (x.0.clone(), x.1 .2.clone()))
                    .collect::<Vec<_>>();

                for node in &excess_nodes {
                    templates.remove(&node.0);
                }

                for node in excess_nodes {
                    node.1.node.unchecked_into::<Element>().remove();
                }
            }
        }
    });

    for item in iterable.get().iter() {
        let key = key_fn(item);
        let template = templates.borrow().get(&key).unwrap().2.clone();

        marker.before_with_node_1(&template.node).unwrap();
    }

    TemplateResult::new(fragment.into())
}

/// Props for [`Indexed`].
pub struct IndexedProps<T: 'static, F>
where
    F: Fn(T) -> TemplateResult,
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
pub fn Indexed<T, F: 'static>(props: IndexedProps<T, F>) -> TemplateResult
where
    T: Clone + PartialEq,
    F: Fn(T) -> TemplateResult,
{
    let templates: Rc<RefCell<Vec<(Owner, TemplateResult)>>> = Rc::new(RefCell::new(Vec::new()));

    // Previous values for diffing purposes.
    let previous_values = RefCell::new(Vec::new());

    let fragment = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_document_fragment();

    let marker = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_comment("");

    append(&fragment, &marker);

    create_effect({
        let templates = Rc::clone(&templates);
        let marker = marker.clone();
        move || {
            // Find values that changed by comparing to previous_values.
            for (i, item) in props.iterable.get().iter().enumerate() {
                let previous_values = previous_values.borrow();
                let previous_value = previous_values.get(i);

                if previous_value.is_none() || previous_value.unwrap() != item {
                    // value changed, re-render item

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
                        parent
                            .replace_child(&new_template.unwrap().node, &old_node.1.node)
                            .unwrap();
                    } else {
                        debug_assert!(templates.borrow().len() == i, "pushing new value scenario");

                        templates
                            .borrow_mut()
                            .push((owner, new_template.as_ref().unwrap().clone()));

                        marker
                            .before_with_node_1(&new_template.unwrap().node)
                            .unwrap();
                    }
                }
            }

            if templates.borrow().len() > props.iterable.get().len() {
                let mut templates = templates.borrow_mut();
                let excess_nodes = templates.drain(props.iterable.get().len()..);

                for node in excess_nodes {
                    node.1.node.unchecked_into::<Element>().remove();
                }
            }

            *previous_values.borrow_mut() = (*props.iterable.get()).clone();
        }
    });

    for template in templates.borrow().iter() {
        marker.before_with_node_1(&template.1.node).unwrap();
    }

    TemplateResult::new(fragment.into())
}
