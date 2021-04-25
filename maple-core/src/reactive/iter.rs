use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use std::rc::Rc;

use super::*;

// Credits: Ported from TypeScript implementation in https://github.com/ryansolid/solid
pub fn map_keyed<T, U>(
    list: StateHandle<Vec<T>>,
    map_fn: impl Fn(&T) -> U + 'static,
) -> impl FnMut() -> Rc<Vec<U>>
where
    T: Eq + Clone + Hash,
    U: Clone + 'static,
{
    // Previous state used for diffing.
    let mut items = Vec::new();
    let mapped = Rc::new(RefCell::new(Vec::<U>::new()));
    let mut scopes: Vec<Option<Rc<ReactiveScope>>> = Vec::new();

    move || {
        let new_items = list.get(); // Subscribe to list.
        untrack(|| {
            if new_items.is_empty() && !items.is_empty() {
                // Fast path for removing all items.
                drop(mem::take(&mut scopes));
                *mapped.borrow_mut() = Vec::new();
            } else if !new_items.is_empty() && items.is_empty() {
                // Fast path for new create.
                for new_item in new_items.iter() {
                    let mut new_mapped = None;
                    let new_scope = create_root(|| {
                        new_mapped = Some(map_fn(new_item));
                    });
                    mapped.borrow_mut().push(new_mapped.unwrap());
                    scopes.push(Some(Rc::new(new_scope)));
                }
            } else {
                // Skip common prefix.
                let mut start = 0;
                let end = usize::min(items.len(), new_items.len());
                while start < end && items[start] == new_items[start] {
                    start += 1;
                }

                // Skip common suffix.
                let mut end = items.len() as isize - 1;
                let mut new_end = new_items.len() as isize - 1;
                #[allow(clippy::suspicious_operation_groupings)]
                // FIXME: make code clearer so that clippy won't complain
                while (start as isize) < end
                    && (start as isize) < new_end
                    && items[end as usize] == new_items[new_end as usize]
                {
                    end -= 1;
                    new_end -= 1;
                }

                // Prepare a map of indices in newItems. Scan backwards so we encounter them in
                // natural order.
                let mut new_indices = HashMap::new();
                if (start as isize) < new_end {
                    for i in (start..=new_end as usize).rev() {
                        let item = &new_items[i];
                        new_indices.insert(item, i);
                    }
                }

                // Step through old items and see if they can be found in new set; if so, mark them
                // as moved.
                let mut temp = HashMap::new();
                let mut temp_scopes = HashMap::new();
                if (start as isize) < end {
                    for i in start..=end as usize {
                        let item = &items[i];
                        if let Some(j) = new_indices.get(item).copied() {
                            temp.insert(j, mapped.borrow()[i].clone());
                            temp_scopes.insert(j, scopes.get(i).unwrap().as_ref().cloned());
                        } else {
                            scopes[i] = None;
                        }
                    }
                }

                // Set all the new values, pulling from the temp array if copied, otherwise entering
                // the new value.
                for i in start..new_items.len() {
                    if temp.get(&i).is_some() {
                        mapped.borrow_mut()[i] = temp.remove(&i).unwrap();
                        scopes[i] = temp_scopes.remove(&i).unwrap();
                    } else {
                        let mut new_mapped = None;
                        let new_scope = create_root(|| {
                            new_mapped = Some(map_fn(&new_items[i]));
                        });

                        if mapped.borrow().len() > i {
                            mapped.borrow_mut()[i] = new_mapped.unwrap();
                            scopes[i] = Some(Rc::new(new_scope));
                        } else {
                            mapped.borrow_mut().push(new_mapped.unwrap());
                            scopes.push(Some(Rc::new(new_scope)));
                        }
                    }
                }
            }

            items = (*new_items).clone();
            debug_assert!([items.len(), mapped.borrow().len(), scopes.len()]
                .iter()
                .all(|l| *l == new_items.len()));

            Rc::new((*mapped).clone().into_inner())
        })
    }
}

pub fn map_indexed<T, U>(
    list: StateHandle<Vec<T>>,
    map_fn: impl Fn(&T) -> U + 'static,
) -> impl FnMut() -> Rc<Vec<U>>
where
    T: PartialEq + Clone,
    U: Clone + 'static,
{
    // Previous state used for diffing.
    let mut items = Vec::new();
    let mapped = Rc::new(RefCell::new(Vec::new()));
    let mut scopes = Vec::new();

    move || {
        let new_items = list.get(); // Subscribe to list.
        untrack(|| {
            if new_items.is_empty() && !items.is_empty() {
                // Fast path for removing all items.
                drop(mem::take(&mut scopes));
                items = Vec::new();
                *mapped.borrow_mut() = Vec::new();
            } else {
                for (i, new_item) in new_items.iter().enumerate() {
                    let item = items.get(i);

                    if item.is_none() {
                        let mut new_mapped = None;
                        let new_scope = create_root(|| {
                            new_mapped = Some(map_fn(new_item));
                        });
                        mapped.borrow_mut().push(new_mapped.unwrap());
                        scopes.push(new_scope);
                    } else if item != Some(new_item) {
                        let mut new_mapped = None;
                        let new_scope = create_root(|| {
                            new_mapped = Some(map_fn(new_item));
                        });
                        mapped.borrow_mut()[i] = new_mapped.unwrap();
                        scopes[i] = new_scope;
                    }
                }

                if new_items.len() < items.len() {
                    for _i in new_items.len()..items.len() {
                        scopes.pop();
                    }
                }

                items = (*new_items).clone();
                debug_assert!([items.len(), mapped.borrow().len(), scopes.len()]
                    .iter()
                    .all(|l| *l == new_items.len()));
            }

            Rc::new((*mapped).clone().into_inner())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyed() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_keyed(a.handle(), |x| *x * 2);
        assert_eq!(*mapped(), vec![2, 4, 6]);

        a.set(vec![1, 2, 3, 4]);
        assert_eq!(*mapped(), vec![2, 4, 6, 8]);

        a.set(vec![2, 2, 3, 4]);
        assert_eq!(*mapped(), vec![4, 4, 6, 8]);
    }

    /// Test fast path for clearing Vec.
    #[test]
    fn keyed_clear() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_keyed(a.handle(), |x| *x * 2);

        a.set(Vec::new());
        assert_eq!(*mapped(), Vec::new());
    }

    #[test]
    fn indexed() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_indexed(a.handle(), |x| *x * 2);
        assert_eq!(*mapped(), vec![2, 4, 6]);

        a.set(vec![1, 2, 3, 4]);
        assert_eq!(*mapped(), vec![2, 4, 6, 8]);

        a.set(vec![2, 2, 3, 4]);
        assert_eq!(*mapped(), vec![4, 4, 6, 8]);
    }

    /// Test fast path for clearing Vec.
    #[test]
    fn indexed_clear() {
        let a = Signal::new(vec![1, 2, 3]);
        let mut mapped = map_indexed(a.handle(), |x| *x * 2);

        a.set(Vec::new());
        assert_eq!(*mapped(), Vec::new());
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
}
