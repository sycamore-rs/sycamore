//! Keyed iteration in [`template`](crate::template).

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::prelude::*;

pub struct KeyedProps<T: 'static, F, K, Key>
where
    F: Fn(&T) -> TemplateResult,
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
    F: Fn(&T) -> TemplateResult,
    K: Fn(&T) -> Key,
    Key: Hash + Eq,
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
                map.borrow_mut().insert(key, (props.template)(item));
            }
        }
    });

    todo!();
}

pub struct IndexedProps<T: 'static, F>
where
    F: Fn(&T) -> TemplateResult,
{
    pub iterable: Signal<Vec<T>>,
    pub template: F,
}

pub fn Indexed<T, F: 'static>(props: IndexedProps<T, F>) -> TemplateResult
where
    F: Fn(&T) -> TemplateResult,
    T: Clone + PartialEq,
{
    let templates = RefCell::new(Vec::new());

    // Previous values for diffing purposes.
    let previous_values = RefCell::new((*props.iterable.get()).clone());
    // A list of indexes that need to be re-rendered.
    let changed = Rc::new(RefCell::new(Vec::new()));

    create_effect(move || {
        *changed.borrow_mut() = Vec::new(); // reset changed

        let mut changed_tmp = Vec::new();

        // Find values that changed by comparing to previous_values.
        for (i, item) in props.iterable.get().iter().enumerate() {
            let previous_values = previous_values.borrow();
            let previous_value = previous_values.get(i);

            if previous_value.is_none() || previous_value.unwrap() != item {
                changed_tmp.push(i);
                // value changed, re-render item

                if let Some(template) = templates.borrow_mut().get_mut(i) {
                    *template = Some((props.template)(item));
                } else {
                    debug_assert!(templates.borrow().len() == i, "pushing new value scenario");

                    templates.borrow_mut().push(Some((props.template)(item)));
                }
            }
        }

        if templates.borrow().len() > props.iterable.get().len() {
            templates.borrow_mut().truncate(props.iterable.get().len());
        }

        *previous_values.borrow_mut() = (*props.iterable.get()).clone();
        *changed.borrow_mut() = changed_tmp;

        debug_assert!(
            templates.borrow_mut().iter().all(|item| item.is_some()),
            "templates should all be Some"
        );
    });

    todo!();
}
