//! Keyed iteration in [`template`](crate::template).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;
use std::rc::Rc;

use wasm_bindgen::*;
use web_sys::Element;

use crate::internal::append;
use crate::prelude::*;

pub struct KeyedProps<T: 'static, F, K, Key>
where
    F: Fn(T) -> TemplateResult,
    K: Fn(&T) -> Key,
    Key: Hash + Eq,
{
    pub iterable: Signal<Vec<T>>,
    pub template: F,
    pub key: K,
}

pub fn Keyed<T, F: 'static, K: 'static, Key: 'static>(
    props: KeyedProps<T, F, K, Key>,
) -> TemplateResult
where
    F: Fn(T) -> TemplateResult,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    // A tuple with a value of type `T` and the `TemplateResult` produces by calling `props.template` with the first value.
    let templates: Rc<RefCell<HashMap<Key, (T, Option<TemplateResult>)>>> =
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
        let templates = Rc::clone(&templates);
        let marker = marker.clone();
        move || {
            let previous_values = (*templates.borrow()).clone();

            // Find values that changed by comparing to previous_values.
            for (i, item) in props.iterable.get().iter().enumerate() {
                let key = (props.key)(item);

                let previous_value = previous_values.get(&key);

                if previous_value.is_none() || &previous_value.unwrap().0 != item {
                    // value changed, re-render item

                    if templates.borrow().get(&key).is_some() {
                        let new_node = (props.template)(item.clone());
                        let (_, old_node) = mem::replace(
                            templates.borrow_mut().get_mut(&key).unwrap(),
                            (item.clone(), Some(new_node.clone())),
                        );

                        let parent = old_node.as_ref().unwrap().node.parent_node().unwrap();
                        parent
                            .replace_child(&new_node.node, &old_node.unwrap().node)
                            .unwrap();
                    } else {
                        debug_assert!(templates.borrow().len() == i, "pushing new value scenario");

                        templates.borrow_mut().insert(
                            key.clone(),
                            (item.clone(), Some((props.template)(item.clone()))),
                        );

                        marker
                            .before_with_node_1(
                                &templates
                                    .borrow()
                                    .get(&key)
                                    .as_ref()
                                    .unwrap()
                                    .1
                                    .as_ref()
                                    .unwrap()
                                    .node,
                            )
                            .unwrap();
                    }
                }
            }

            if templates.borrow().len() > props.iterable.get().len() {
                let mut templates = templates.borrow_mut();
                let new_keys: HashSet<Key> = props
                    .iterable
                    .get()
                    .iter()
                    .map(|item| (props.key)(item))
                    .collect();

                let excess_nodes = templates
                    .iter()
                    .filter(|item| new_keys.get(item.0).is_none())
                    .map(|x| (x.0.clone(), x.1.clone()))
                    .collect::<Vec<_>>();

                for node in &excess_nodes {
                    templates.remove(&node.0);
                }

                for node in excess_nodes {
                    node.1
                         .1
                        .as_ref()
                        .unwrap()
                        .clone()
                        .node
                        .unchecked_into::<Element>()
                        .remove();
                }
            }

            debug_assert!(
                templates.borrow().values().all(|item| item.1.is_some()),
                "templates should all be Some"
            );
        }
    });

    for template in templates.borrow().values() {
        let template = template.1.as_ref().unwrap().clone();
        marker.before_with_node_1(&template.node).unwrap();
    }

    TemplateResult::new(fragment.into())
}

pub struct IndexedProps<T: 'static, F>
where
    F: Fn(T) -> TemplateResult,
{
    pub iterable: Signal<Vec<T>>,
    pub template: F,
}

pub fn Indexed<T, F: 'static>(props: IndexedProps<T, F>) -> TemplateResult
where
    T: Clone + PartialEq,
    F: Fn(T) -> TemplateResult,
{
    let templates: Rc<RefCell<Vec<Option<TemplateResult>>>> = Rc::new(RefCell::new(Vec::new()));

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

                    if templates.borrow().get(i).is_some() {
                        let new_node = (props.template)(item.clone());
                        let old_node =
                            mem::replace(&mut templates.borrow_mut()[i], Some(new_node.clone()))
                                .unwrap();

                        let parent = old_node.node.parent_node().unwrap();
                        parent
                            .replace_child(&new_node.node, &old_node.node)
                            .unwrap();
                    } else {
                        debug_assert!(templates.borrow().len() == i, "pushing new value scenario");

                        templates
                            .borrow_mut()
                            .push(Some((props.template)(item.clone())));

                        marker
                            .before_with_node_1(&templates.borrow()[i].as_ref().unwrap().node)
                            .unwrap();
                    }
                }
            }

            if templates.borrow().len() > props.iterable.get().len() {
                let mut templates = templates.borrow_mut();
                let excess_nodes = templates.drain(props.iterable.get().len()..);

                for node in excess_nodes {
                    node.unwrap().node.unchecked_into::<Element>().remove();
                }
            }

            *previous_values.borrow_mut() = (*props.iterable.get()).clone();

            debug_assert!(
                templates.borrow().iter().all(|item| item.is_some()),
                "templates should all be Some"
            );
        }
    });

    for template in templates.borrow().iter() {
        let template = template.as_ref().unwrap().clone();
        marker.before_with_node_1(&template.node).unwrap();
    }

    TemplateResult::new(fragment.into())
}
