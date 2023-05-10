use std::cell::RefCell;
use std::ops::Deref;

use crate::signals::{create_signal, Signal};
use crate::{DependencyTracker, Scope};

pub struct Memo<T: 'static> {
    signal: Signal<T>,
}

// TODO: Don't do this.
impl<T> Deref for Memo<T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal,
        }
    }
}
impl<T> Copy for Memo<T> {}

pub(crate) fn create_updated_signal<T>(
    cx: Scope,
    initial: T,
    initial_deps: DependencyTracker,
    mut f: impl FnMut(&mut T) -> bool + 'static,
) -> Signal<T> {
    let signal = create_signal(cx, initial);
    initial_deps.create_dependency_links(cx.root, signal.id);

    // Set the signal update callback as f.
    signal.get_data_mut(move |data| {
        data.update = Some(Box::new(move |any| {
            f(any.downcast_mut().expect("could not downcast memo value"))
        }))
    });

    signal
}

pub fn create_memo<T>(cx: Scope, mut f: impl FnMut() -> T + 'static) -> Memo<T> {
    let (initial, tracker) = cx.root.tracked_scope(&mut f);
    let signal = create_updated_signal(cx, initial, tracker, move |value| {
        *value = f();
        true
    });

    Memo { signal }
}

pub fn create_selector_with<T>(
    cx: Scope,
    mut f: impl FnMut() -> T + 'static,
    mut eq: impl FnMut(&T, &T) -> bool + 'static,
) -> Memo<T> {
    let (initial, tracker) = cx.root.tracked_scope(&mut f);
    let signal = create_updated_signal(cx, initial, tracker, move |value| {
        let new = f();
        if eq(&new, &value) {
            false
        } else {
            *value = new;
            true
        }
    });

    Memo { signal }
}

pub fn create_selector<T>(cx: Scope, f: impl FnMut() -> T + 'static) -> Memo<T>
where
    T: PartialEq,
{
    create_selector_with(cx, f, PartialEq::eq)
}

pub fn create_reducer<T, Msg>(
    cx: Scope,
    initial: T,
    reduce: impl FnMut(&T, Msg) -> T,
) -> (Memo<T>, impl Fn(Msg)) {
    let reduce = RefCell::new(reduce);
    let signal = create_signal(cx, initial);
    let dispatch = move |msg| signal.update(|value| *value = reduce.borrow_mut()(&value, msg));
    (Memo { signal }, dispatch)
}

pub fn create_effect(cx: Scope, f: impl FnMut() + 'static) {
    let _effect = create_memo(cx, f);
}
