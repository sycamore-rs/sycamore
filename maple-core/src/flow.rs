//! Keyed iteration in [`template`](crate::template).

use std::cell::RefCell;
use std::collections::HashMap;
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
    Key: Hash + Eq,
    T: Clone,
{
    let map = RefCell::new(HashMap::new());

    let _old_keys: Vec<_> = props
        .iterable
        .get()
        .iter()
        .map(|item| (props.key)(item))
        .collect();

    create_effect(move || {
        for item in props.iterable.get().iter() {
            let key = (props.key)(item);

            if !map.borrow().contains_key(&key) {
                map.borrow_mut().insert(key, (props.template)(item.clone()));
            }
        }
    });

    todo!();
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
    // A list of indexes that need to be re-rendered.
    let changed = Rc::new(RefCell::new(Vec::new()));

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
            *changed.borrow_mut() = Vec::new(); // reset changed

            let mut changed_tmp = Vec::new();

            // Find values that changed by comparing to previous_values.
            for (i, item) in props.iterable.get().iter().enumerate() {
                let previous_values = previous_values.borrow();
                let previous_value = previous_values.get(i);

                if previous_value.is_none() || previous_value.unwrap() != item {
                    changed_tmp.push(i);
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

            *changed.borrow_mut() = changed_tmp;

            debug_assert!(
                templates.borrow_mut().iter().all(|item| item.is_some()),
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
