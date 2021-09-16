//! Reactive utilities for dealing with lists.

use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use ahash::AHashMap;
use wasm_bindgen::prelude::*;

use super::*;

/// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
/// computed, meaning that it's value will only be updated when requested. Modifications to the
/// input `Vec` are diffed using keys to prevent recomputing values that have not changed.
///
/// This function is the underlying utility behind `Keyed`.
///
/// # Params
/// * `list` - The list to be mapped. The list must be a [`StateHandle`] (obtained from a
///   [`Signal`]) and therefore reactive.
/// * `map_fn` - A closure that maps from the input type to the output type.
/// * `key_fn` - A closure that returns an _unique_ key to each entry.
///
///  _Credits: Based on TypeScript implementation in <https://github.com/solidjs/solid>_
pub fn map_keyed<T, K, U>(
    list: StateHandle<Vec<T>>,
    map_fn: impl Fn(&T) -> U + 'static,
    key_fn: impl Fn(&T) -> K + 'static,
) -> impl FnMut() -> Vec<U>
where
    T: Eq + Clone,
    K: Eq + Hash,
    U: Clone + 'static,
{
    // Previous state used for diffing.
    let mut items = Rc::new(Vec::new());
    let mapped = Rc::new(RefCell::new(Vec::new()));
    let mut scopes: Vec<Option<Rc<ReactiveScope>>> = Vec::new();

    move || {
        let new_items = list.get(); // Subscribe to list.
        untrack(|| {
            if new_items.is_empty() {
                // Fast path for removing all items.
                scopes = Vec::new();
                *mapped.borrow_mut() = Vec::new();
            } else if items.is_empty() {
                // Fast path for new create.
                for new_item in new_items.iter() {
                    let new_scope = create_root(|| {
                        mapped.borrow_mut().push(map_fn(new_item));
                    });
                    scopes.push(Some(Rc::new(new_scope)));
                }
            } else {
                debug_assert!(
                    !new_items.is_empty() && !items.is_empty(),
                    "new_items.is_empty() and items.is_empty() are special cased"
                );

                let mut temp = vec![None; new_items.len()];
                let mut temp_scopes = vec![None; new_items.len()];

                // Skip common prefix.
                let min_len = usize::min(items.len(), new_items.len());
                let start = items
                    .iter()
                    .zip(new_items.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(min_len);
                debug_assert!(
                    (items.get(start).is_none() && new_items.get(start).is_none())
                        || (items.get(start) != new_items.get(start)),
                    "start is the first index where items[start] != new_items[start]"
                );

                // Skip common suffix.
                let mut end = items.len();
                let mut new_end = new_items.len();
                #[allow(clippy::suspicious_operation_groupings)]
                // FIXME: make code clearer so that clippy won't complain
                while end > start && new_end > start && items[end - 1] == new_items[new_end - 1] {
                    end -= 1;
                    new_end -= 1;
                    temp[new_end] = Some(mapped.borrow()[end].clone());
                    temp_scopes[new_end] = scopes[end].clone();
                }
                debug_assert!(
                    if end != 0 && new_end != 0 {
                        (end == items.len() && new_end == new_items.len())
                            || (items[end - 1] != new_items[new_end - 1])
                    } else {
                        true
                    },
                    "end and new_end are the last indexes where items[end - 1] != new_items[new_end - 1]"
                );

                // 0) Prepare a map of indices in newItems. Scan backwards so we encounter them in
                // natural order.
                let mut new_indices = AHashMap::with_capacity(new_end - start);

                // Indexes for new_indices_next are shifted by start because values at 0..start are
                // always None.
                let mut new_indices_next = vec![None; new_end - start];
                for j in (start..new_end).rev() {
                    let item = &new_items[j];
                    let i = new_indices.get(&key_fn(item));
                    new_indices_next[j - start] = i.copied();
                    new_indices.insert(key_fn(item), j);
                }

                // 1) Step through old items and see if they can be found in new set; if so, mark
                // them as moved.
                for i in start..end {
                    let item = &items[i];
                    if let Some(j) = new_indices.get(&key_fn(item)).copied() {
                        // Moved. j is index of item in new_items.
                        temp[j] = Some(mapped.borrow()[i].clone());
                        temp_scopes[j] = scopes[i].clone();
                        new_indices_next[j - start]
                            .and_then(|j| new_indices.insert(key_fn(item), j));
                    } else {
                        // Create new.
                        scopes[i] = None;
                    }
                }

                // 2) Set all the new values, pulling from the moved array if copied, otherwise
                // entering the new value.
                for j in start..new_items.len() {
                    if matches!(temp.get(j), Some(Some(_))) {
                        // Pull from moved array.
                        if j >= mapped.borrow().len() {
                            debug_assert_eq!(mapped.borrow().len(), j);
                            mapped.borrow_mut().push(temp[j].clone().unwrap_throw());
                            scopes.push(temp_scopes[j].clone());
                        } else {
                            mapped.borrow_mut()[j] = temp[j].clone().unwrap_throw();
                            scopes[j] = temp_scopes[j].clone();
                        }
                    } else {
                        // Create new value.
                        let mut new_mapped = None;
                        let new_scope = create_root(|| {
                            new_mapped = Some(map_fn(&new_items[j]));
                        });

                        if mapped.borrow().len() > j {
                            mapped.borrow_mut()[j] = new_mapped.unwrap_throw();
                            scopes[j] = Some(Rc::new(new_scope));
                        } else {
                            mapped.borrow_mut().push(new_mapped.unwrap_throw());
                            scopes.push(Some(Rc::new(new_scope)));
                        }
                    }
                }
            }

            // 3) In case the new set is shorter than the old, set the length of the mapped array.
            mapped.borrow_mut().truncate(new_items.len());
            scopes.truncate(new_items.len());

            // 4) save a copy of the mapped items for the next update.
            items = Rc::clone(&new_items);
            debug_assert!([items.len(), mapped.borrow().len(), scopes.len()]
                .iter()
                .all(|l| *l == new_items.len()));

            mapped.borrow().clone()
        })
    }
}

/// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
/// computed, meaning that it's value will only be updated when requested. Modifications to the
/// input `Vec` are diffed by index to prevent recomputing values that have not changed.
///
/// Generally, it is preferred to use [`map_keyed`] instead when a key function is available.
///
/// This function is the underlying utility behind `Indexed`.
///
/// # Params
/// * `list` - The list to be mapped. The list must be a [`StateHandle`] (obtained from a
///   [`Signal`]) and therefore reactive.
/// * `map_fn` - A closure that maps from the input type to the output type.
pub fn map_indexed<T, U>(
    list: StateHandle<Vec<T>>,
    map_fn: impl Fn(&T) -> U + 'static,
) -> impl FnMut() -> Vec<U>
where
    T: PartialEq + Clone,
    U: Clone + 'static,
{
    // Previous state used for diffing.
    let mut items = Rc::new(Vec::new());
    let mapped = Rc::new(RefCell::new(Vec::new()));
    let mut scopes = Vec::new();

    move || {
        let new_items = list.get(); // Subscribe to list.
        untrack(|| {
            if new_items.is_empty() {
                // Fast path for removing all items.
                scopes = Vec::new();
                items = Rc::new(Vec::new());
                *mapped.borrow_mut() = Vec::new();
            } else {
                // Pre-allocate space needed
                if new_items.len() > items.len() {
                    let new_count = new_items.len() - items.len();
                    mapped.borrow_mut().reserve(new_count);
                    scopes.reserve(new_count);
                }

                for (i, new_item) in new_items.iter().enumerate() {
                    let item = items.get(i);

                    if item.is_none() {
                        let new_scope = create_root(|| {
                            mapped.borrow_mut().push(map_fn(new_item));
                        });
                        scopes.push(new_scope);
                    } else if item != Some(new_item) {
                        let new_scope = create_root(|| {
                            mapped.borrow_mut()[i] = map_fn(new_item);
                        });
                        scopes[i] = new_scope;
                    }
                }

                if new_items.len() < items.len() {
                    for _i in new_items.len()..items.len() {
                        scopes.pop();
                    }
                }

                // In case the new set is shorter than the old, set the length of the mapped array.
                mapped.borrow_mut().truncate(new_items.len());
                scopes.truncate(new_items.len());

                // save a copy of the mapped items for the next update.
                items = Rc::clone(&new_items);
                debug_assert!([items.len(), mapped.borrow().len(), scopes.len()]
                    .iter()
                    .all(|l| *l == new_items.len()));
            }

            mapped.borrow().clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn keyed() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_keyed(a.handle(), |x| *x * 2, |x| *x);
        assert_eq!(mapped(), vec![2, 4, 6]);

        a.set(vec![1, 2, 3, 4]);
        assert_eq!(mapped(), vec![2, 4, 6, 8]);

        a.set(vec![2, 2, 3, 4]);
        assert_eq!(mapped(), vec![4, 4, 6, 8]);
    }

    #[test]
    fn keyed_recompute_everything() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_keyed(a.handle(), |x| *x * 2, |x| *x);
        assert_eq!(mapped(), vec![2, 4, 6]);

        a.set(vec![4, 5, 6]);
        assert_eq!(mapped(), vec![8, 10, 12]);
    }

    /// Test fast path for clearing Vec.
    #[test]
    fn keyed_clear() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_keyed(a.handle(), |x| *x * 2, |x| *x);

        a.set(Vec::new());
        assert_eq!(mapped(), Vec::<i32>::new());
    }

    /// Test that using [`map_keyed`] will reuse previous computations.
    #[test]
    fn keyed_use_previous_computation() {
        let a = Signal::new(vec![1, 2, 3]);
        let counter = Rc::new(Cell::new(0));
        let mut mapped = map_keyed(
            a.handle(),
            {
                let counter = Rc::clone(&counter);
                move |_| {
                    counter.set(counter.get() + 1);
                    counter.get()
                }
            },
            |x| *x,
        );
        assert_eq!(mapped(), vec![1, 2, 3]);

        a.set(vec![1, 2]);
        assert_eq!(mapped(), vec![1, 2]);

        a.set(vec![1, 2, 4]);
        assert_eq!(mapped(), vec![1, 2, 4]);

        a.set(vec![1, 2, 3, 4]);
        assert_eq!(mapped(), vec![1, 2, 5, 4]);
    }

    #[test]
    fn indexed() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_indexed(a.handle(), |x| *x * 2);
        assert_eq!(mapped(), vec![2, 4, 6]);

        a.set(vec![1, 2, 3, 4]);
        assert_eq!(mapped(), vec![2, 4, 6, 8]);

        a.set(vec![2, 2, 3, 4]);
        assert_eq!(mapped(), vec![4, 4, 6, 8]);
    }

    /// Test fast path for clearing Vec.
    #[test]
    fn indexed_clear() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_indexed(a.handle(), |x| *x * 2);

        a.set(Vec::new());
        assert_eq!(mapped(), Vec::<i32>::new());
    }

    /// Test that result of mapped function can be listened to.
    #[test]
    fn indexed_react() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_indexed(a.handle(), |x| *x * 2);

        let counter = Signal::new(0);
        create_effect({
            let counter = counter.clone();
            move || {
                counter.set(*counter.get_untracked() + 1);
                mapped(); // Subscribe to mapped.
            }
        });

        assert_eq!(*counter.get(), 1);
        a.set(vec![1, 2, 3, 4]);
        assert_eq!(*counter.get(), 2);
    }

    /// Test that using [`map_indexed`] will reuse previous computations.
    #[test]
    fn indexed_use_previous_computation() {
        let a = Signal::new(vec![1, 2, 3]);
        let counter = Rc::new(Cell::new(0));
        let mut mapped = map_indexed(a.handle(), {
            let counter = Rc::clone(&counter);
            move |_| {
                counter.set(counter.get() + 1);
                counter.get()
            }
        });
        assert_eq!(mapped(), vec![1, 2, 3]);

        a.set(vec![1, 2]);
        assert_eq!(mapped(), vec![1, 2]);

        a.set(vec![1, 2, 4]);
        assert_eq!(mapped(), vec![1, 2, 4]);

        a.set(vec![1, 3, 4]);
        assert_eq!(mapped(), vec![1, 5, 4]);
    }
}
