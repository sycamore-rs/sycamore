use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use super::*;

pub fn map_keyed<T, U>(list: StateHandle<Vec<T>>, map_fn: impl Fn(T) -> U) -> StateHandle<Vec<U>> {
    todo!();
}

pub fn map_indexed<T, U>(
    list: StateHandle<Vec<T>>,
    map_fn: impl Fn(&T) -> U + 'static,
) -> Rc<RefCell<Vec<U>>>
where
    T: PartialEq + Clone,
    U: 'static,
{
    // Previous state used for diffing.
    let mut items: Vec<T> = Vec::new();
    let mapped = Rc::new(RefCell::new(Vec::new()));
    let mut scopes: Vec<ReactiveScope> = Vec::new();

    create_effect({
        let mapped = Rc::clone(&mapped);
        move || {
            let new_items = list.get();
            untrack(|| {
                if new_items.is_empty() && !items.is_empty() {
                    // Fast path for removing elements.
                    drop(mem::take(&mut scopes));
                    items = Vec::new();
                    *mapped.borrow_mut() = Vec::new();
                }

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
            })
        }
    });

    mapped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed() {
        let a = Signal::new(vec![1, 2, 3]);
        let mapped = map_indexed(a.handle(), |x| *x * 2);
        debug_assert_eq!(*mapped.borrow(), vec![2, 4, 6]);

        a.set(vec![1, 2, 3, 4]);
        debug_assert_eq!(*mapped.borrow(), vec![2, 4, 6, 8]);

        a.set(vec![2, 2, 3, 4]);
        debug_assert_eq!(*mapped.borrow(), vec![4, 4, 6, 8]);
    }
}
