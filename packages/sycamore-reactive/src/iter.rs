//! Reactive utilities for dealing with lists and iterables.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::rc::Rc;

use crate::*;

impl<'a> Scope<'a> {
    /// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
    /// computed, meaning that it's value will only be updated when requested. Modifications to the
    /// input `Vec` are diffed using keys to prevent recomputing values that have not changed.
    ///
    /// This function is the underlying utility behind `Keyed`.
    ///
    /// # Params
    /// * `list` - The list to be mapped. The list must be a [`ReadSignal`] (obtained from a
    ///   [`Signal`]) and therefore reactive.
    /// * `map_fn` - A closure that maps from the input type to the output type.
    /// * `key_fn` - A closure that returns an _unique_ key to each entry.
    ///
    ///  _Credits: Based on TypeScript implementation in <https://github.com/solidjs/solid>_
    pub fn map_keyed<T, K, U>(
        &'a self,
        list: &'a ReadSignal<Vec<T>>,
        map_fn: impl for<'child_lifetime> Fn(BoundedScopeRef<'child_lifetime, 'a>, T) -> U + 'a,
        key_fn: impl Fn(&T) -> K + 'a,
    ) -> &'a ReadSignal<Vec<U>>
    where
        T: Eq + Clone + 'a,
        K: Eq + Hash,
        U: Clone + 'a,
    {
        let map_fn = Rc::new(map_fn);

        // Previous state used for diffing.
        let mut items = Rc::new(Vec::new());
        let mut mapped: Vec<U> = Vec::new();
        let mut disposers: Vec<Option<Rc<ScopeDisposer<'a>>>> = Vec::new();

        let signal = self.create_signal(Vec::new());

        // Diff and update signal each time list is updated.
        self.create_effect(move || {
            let new_items = list.get();
            if new_items.is_empty() {
                // Fast path for removing all items.
                disposers = Vec::new();
                mapped = Vec::new();
            } else if items.is_empty() {
                // Fast path for new create.
                // TODO: do not clone T
                for new_item in new_items.iter().cloned() {
                    let tmp = Rc::new(RefCell::new(None));
                    let new_disposer = self.create_child_scope({
                        let tmp = Rc::clone(&tmp);
                        let map_fn = Rc::clone(&map_fn);
                        move |ctx| {
                            // SAFETY: f takes the same parameter as the argument to
                            // self.create_child_scope(_).
                            *tmp.borrow_mut() = Some(map_fn(unsafe { std::mem::transmute(ctx) }, new_item));
                        }
                    });
                    mapped.push(tmp.borrow().clone().unwrap());
                    disposers.push(Some(Rc::new(new_disposer)));
                }
            } else {
                debug_assert!(
                    !new_items.is_empty() && !items.is_empty(),
                    "new_items.is_empty() and items.is_empty() are special cased"
                );

                let mut temp = vec![None; new_items.len()];
                let mut temp_disposers = vec![None; new_items.len()];

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
                    temp[new_end] = Some(mapped[end].clone());
                    temp_disposers[new_end] = disposers[end].clone();
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
                let mut new_indices = HashMap::with_capacity(new_end - start);

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
                        temp[j] = Some(mapped[i].clone());
                        temp_disposers[j] = disposers[i].clone();
                        new_indices_next[j - start]
                            .and_then(|j| new_indices.insert(key_fn(item), j));
                    } else {
                        // Create new.
                        disposers[i] = None;
                    }
                }

                // 2) Set all the new values, pulling from the moved array if copied, otherwise
                // entering the new value.
                for j in start..new_items.len() {
                    if matches!(temp.get(j), Some(Some(_))) {
                        // Pull from moved array.
                        if j >= mapped.len() {
                            debug_assert_eq!(mapped.len(), j);
                            mapped.push(temp[j].clone().unwrap());
                            disposers.push(temp_disposers[j].clone());
                        } else {
                            mapped[j] = temp[j].clone().unwrap();
                            disposers[j] = temp_disposers[j].clone();
                        }
                    } else {
                        // Create new value.
                        let tmp = Rc::new(RefCell::new(None));
                        let new_disposer = self.create_child_scope({
                            let tmp = Rc::clone(&tmp);
                            let map_fn = Rc::clone(&map_fn);
                            let new_item = new_items[j].clone();
                            move |ctx| {
                                // SAFETY: f takes the same parameter as the argument to
                                // self.create_child_scope(_).
                                *tmp.borrow_mut() = Some(map_fn(unsafe { std::mem::transmute(ctx) }, new_item));
                            }
                        });

                        if mapped.len() > j {
                            mapped[j] = tmp.borrow().clone().unwrap();
                            disposers[j] = Some(Rc::new(new_disposer));
                        } else {
                            mapped.push(tmp.borrow().clone().unwrap());
                            disposers.push(Some(Rc::new(new_disposer)));
                        }
                    }
                }
            }

            // 3) In case the new set is shorter than the old, set the length of the mapped array.
            mapped.truncate(new_items.len());
            disposers.truncate(new_items.len());

            // 4) Save a copy of the mapped items for the next update.
            items = Rc::clone(&new_items);
            debug_assert!([items.len(), mapped.len(), disposers.len()]
                .iter()
                .all(|l| *l == new_items.len()));

            // 5) Update signal to trigger updates.
            signal.set(mapped.clone());
        });

        signal
    }

    /// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
    /// computed, meaning that it's value will only be updated when requested. Modifications to the
    /// input `Vec` are diffed by index to prevent recomputing values that have not changed.
    ///
    /// Generally, it is preferred to use [`map_keyed`](Self::map_keyed) instead when a key function
    /// is available.
    ///
    /// This function is the underlying utility behind `Indexed`.
    ///
    /// # Params
    /// * `list` - The list to be mapped. The list must be a [`ReadSignal`] (obtained from a
    ///   [`Signal`]) and therefore reactive.
    /// * `map_fn` - A closure that maps from the input type to the output type.
    pub fn map_indexed<T, U>(
        &'a self,
        list: &'a ReadSignal<Vec<T>>,
        map_fn: impl for<'child_lifetime> Fn(BoundedScopeRef<'child_lifetime, 'a>, T) -> U + 'a,
    ) -> &'a ReadSignal<Vec<U>>
    where
        T: PartialEq + Clone,
        U: Clone + 'a,
    {
        let map_fn = Rc::new(map_fn);

        // Previous state used for diffing.
        let mut items = Rc::new(Vec::new());
        let mut mapped = Vec::new();
        let mut disposers: Vec<Box<ScopeDisposer<'a>>> = Vec::new();

        let signal = self.create_signal(Vec::new());

        // Diff and update signal each time list is updated.
        self.create_effect(move || {
            let new_items = list.get();

            if new_items.is_empty() {
                // Fast path for removing all items.
                disposers = Vec::new();
                items = Rc::new(Vec::new());
                mapped = Vec::new();
            } else {
                // Pre-allocate space needed
                if new_items.len() > items.len() {
                    let new_count = new_items.len() - items.len();
                    mapped.reserve(new_count);
                    disposers.reserve(new_count);
                }

                for (i, new_item) in new_items.iter().enumerate() {
                    let new_item = new_item.clone();
                    let item = items.get(i);
                    // We lift the equality out of the else if branch to satisfy borrow checker.
                    let eqs = item != Some(&new_item);

                    let mut tmp = MaybeUninit::<U>::zeroed();
                    let ptr = &mut tmp as *mut MaybeUninit<U>;
                    if item.is_none() || eqs {
                        let new_disposer = self.create_child_scope({
                            let map_fn = Rc::clone(&map_fn);
                            move |ctx| unsafe {
                                // SAFETY: callback is called immediately in
                                // self.create_child_scope.
                                // ptr is still accessible after self.create_child_scope and
                                // therefore lives long enough.

                                // SAFETY: f takes the same parameter as the argument to
                                // self.create_child_scope(_).
                                (*ptr).write(map_fn(std::mem::transmute(ctx), new_item));
                            }
                        });
                        if item.is_none() {
                            // SAFETY: tmp is written in self.create_child_scope
                            mapped.push(unsafe { tmp.assume_init() });
                            disposers.push(Box::new(new_disposer));
                        } else if eqs {
                            // SAFETY: tmp is written in self.create_child_scope
                            mapped[i] = unsafe { tmp.assume_init() };
                            disposers[i] = Box::new(new_disposer);
                        }
                    }
                }

                if new_items.len() < items.len() {
                    for _i in new_items.len()..items.len() {
                        disposers.pop();
                    }
                }

                // In case the new set is shorter than the old, set the length of the mapped array.
                mapped.truncate(new_items.len());
                disposers.truncate(new_items.len());

                // save a copy of the mapped items for the next update.
                items = Rc::clone(&new_items);
                debug_assert!([items.len(), mapped.len(), disposers.len()]
                    .iter()
                    .all(|l| *l == new_items.len()));
            }

            // Update signal to trigger updates.
            signal.set(mapped.clone());
        });

        signal
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn keyed() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let mapped = ctx.map_keyed(a, |_, x| x * 2, |x| *x);
            assert_eq!(*mapped.get(), vec![2, 4, 6]);

            a.set(vec![1, 2, 3, 4]);
            assert_eq!(*mapped.get(), vec![2, 4, 6, 8]);

            a.set(vec![2, 2, 3, 4]);
            assert_eq!(*mapped.get(), vec![4, 4, 6, 8]);
        });
    }

    #[test]
    fn keyed_recompute_everything() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let mapped = ctx.map_keyed(a, |_, x| x * 2, |x| *x);
            assert_eq!(*mapped.get(), vec![2, 4, 6]);

            a.set(vec![4, 5, 6]);
            assert_eq!(*mapped.get(), vec![8, 10, 12]);
        });
    }

    /// Test fast path for clearing Vec.
    #[test]
    fn keyed_clear() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let mapped = ctx.map_keyed(a, |_, x| x * 2, |x| *x);

            a.set(Vec::new());
            assert_eq!(*mapped.get(), Vec::<i32>::new());
        });
    }

    /// Test that using [`Scope::map_keyed`] will reuse previous computations.
    #[test]
    fn keyed_use_previous_computation() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let counter = Rc::new(Cell::new(0));
            let mapped = ctx.map_keyed(
                a,
                {
                    let counter = Rc::clone(&counter);
                    move |_, _| {
                        counter.set(counter.get() + 1);
                        counter.get()
                    }
                },
                |x| *x,
            );
            assert_eq!(*mapped.get(), vec![1, 2, 3]);

            a.set(vec![1, 2]);
            assert_eq!(*mapped.get(), vec![1, 2]);

            a.set(vec![1, 2, 4]);
            assert_eq!(*mapped.get(), vec![1, 2, 4]);

            a.set(vec![1, 2, 3, 4]);
            assert_eq!(*mapped.get(), vec![1, 2, 5, 4]);
        });
    }

    #[test]
    fn indexed() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let mapped = ctx.map_indexed(a, |_, x| x * 2);
            assert_eq!(*mapped.get(), vec![2, 4, 6]);

            a.set(vec![1, 2, 3, 4]);
            assert_eq!(*mapped.get(), vec![2, 4, 6, 8]);

            a.set(vec![2, 2, 3, 4]);
            assert_eq!(*mapped.get(), vec![4, 4, 6, 8]);
        });
    }

    /// Test fast path for clearing Vec.
    #[test]
    fn indexed_clear() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let mapped = ctx.map_indexed(a, |_, x| x * 2);

            a.set(Vec::new());
            assert_eq!(*mapped.get(), Vec::<i32>::new());
        });
    }

    /// Test that result of mapped function can be listened to.
    #[test]
    fn indexed_react() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let mapped = ctx.map_indexed(a, |_, x| x * 2);

            let counter = ctx.create_signal(0);
            ctx.create_effect(|| {
                counter.set(*counter.get_untracked() + 1);
                mapped.track();
            });

            assert_eq!(*counter.get(), 1);
            a.set(vec![1, 2, 3, 4]);
            assert_eq!(*counter.get(), 2);
        });
    }

    /// Test that using [`map_indexed`] will reuse previous computations.
    #[test]
    fn indexed_use_previous_computation() {
        create_scope_immediate(|ctx| {
            let a = ctx.create_signal(vec![1, 2, 3]);
            let counter = Rc::new(Cell::new(0));
            let mapped = ctx.map_indexed(a, {
                let counter = Rc::clone(&counter);
                move |_, _| {
                    counter.set(counter.get() + 1);
                    counter.get()
                }
            });
            assert_eq!(*mapped.get(), vec![1, 2, 3]);

            a.set(vec![1, 2]);
            assert_eq!(*mapped.get(), vec![1, 2]);

            a.set(vec![1, 2, 4]);
            assert_eq!(*mapped.get(), vec![1, 2, 4]);

            a.set(vec![1, 3, 4]);
            assert_eq!(*mapped.get(), vec![1, 5, 4]);
        });
    }
}
