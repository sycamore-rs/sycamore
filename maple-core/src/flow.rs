//! Keyed iteration in [`template`](crate::template).

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

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
