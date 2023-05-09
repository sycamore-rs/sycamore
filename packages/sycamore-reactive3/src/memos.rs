use std::ops::Deref;

use crate::signals::{create_signal, Signal};
use crate::Scope;

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

pub fn create_memo<T>(cx: Scope, mut f: impl FnMut() -> T + 'static) -> Memo<T> {
    let (initial, tracker) = cx.root.tracked_scope(|| f());
    let signal = create_signal(cx, initial);
    tracker.create_dependency_links(cx.root, signal.id);

    signal.get_data_mut(move |data| {
        // Set the signal update callback as f.
        data.update = Some(Box::new(move |any| {
            let new = f();
            *any = Box::new(new);
        }));
    });

    Memo { signal }
}
