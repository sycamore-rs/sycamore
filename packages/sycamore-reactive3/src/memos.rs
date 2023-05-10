//! Memos (aka. eager derived signals).

use std::cell::RefCell;
use std::fmt::{self, Formatter};
use std::ops::Deref;

use crate::signals::{create_signal, Signal};
use crate::{DependencyTracker, ReadSignal, Scope};

/// A memoized derived signal.
///
/// Usually created using [`create_memo`], [`create_selector`], and [`create_selector_with`].
pub struct Memo<T: 'static> {
    signal: Signal<T>,
}

impl<T> Memo<T> {
    /// Get the inner [`Signal`] that is backing this memo.
    ///
    /// Be careful when using this! Normally, you should not be able to update a memo manually
    /// because that is already being done automatically. However, you can use this to create a
    /// "writable memo", one which can be both updated manually and automatically.
    pub fn inner_signal(self) -> Signal<T> {
        self.signal
    }
}

impl<T> Deref for Memo<T> {
    type Target = ReadSignal<T>;

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

impl<T: fmt::Debug> fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}
impl<T: fmt::Display> fmt::Display for Memo<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}
/// Create a new [`Signal`] from an initial value, an initial list of dependencies, and an update
/// function. Used in the implementation of [`create_memo`] and friends.
pub(crate) fn create_updated_signal<T>(
    cx: Scope,
    initial: T,
    initial_deps: DependencyTracker,
    mut f: impl FnMut(&mut T) -> bool + 'static,
) -> Signal<T> {
    let signal = create_signal(cx, initial);
    initial_deps.create_dependency_links(cx.root, signal.0.id);

    // Set the signal update callback as f.
    signal.0.get_data_mut(move |data| {
        data.update = Some(Box::new(move |any| {
            f(any.downcast_mut().expect("could not downcast memo value"))
        }))
    });

    signal
}

/// Creates a memoized computation from some signals.
/// The output is derived from all the signals that are used within the memo closure.
/// If any of the tracked signals are updated, the memo is also updated.
///
/// # Difference from derived signals
///
/// Derived signals (functions referencing signals) are lazy and do not keep track of the result
/// of the computation. This means that the computation will not be executed until needed.
/// This also means that calling the derived signal twice will result in the same computation
/// twice.
///
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
/// let double = || state.get() * 2;
///
/// let _ = double();
/// // Here, the closure named double is called again.
/// // If the computation is expensive enough, this would be wasted work!
/// let _ = double();
/// # });
/// ```
///
/// Memos, on the other hand, are eagerly evaluated and will only run the computation when one
/// of its dependencies change.
///
/// Memos also incur a slightly higher performance penalty than simple derived signals, so unless
/// there is some computation involved, it will likely be faster to just use a derived signal.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
/// let double = create_memo(cx, move || state.get() * 2);
///
/// assert_eq!(double.get(), 0);
/// state.set(1);
/// assert_eq!(double.get(), 2);
/// # });
/// ```
pub fn create_memo<T>(cx: Scope, mut f: impl FnMut() -> T + 'static) -> Memo<T> {
    let (initial, tracker) = cx.root.tracked_scope(&mut f);
    let signal = create_updated_signal(cx, initial, tracker, move |value| {
        *value = f();
        true
    });

    Memo { signal }
}

/// Creates a memoized value from some signals.
/// Unlike [`create_memo`], this function will not notify dependents of a
/// change if the output is the same.
///
/// It takes a comparison function to compare the old and new value, which returns `true` if
/// they are the same and `false` otherwise.
///
/// To use the type's [`PartialEq`] implementation instead of a custom function, use
/// [`create_selector`].
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

/// Creates a memoized value from some signals.
/// Unlike [`create_memo`], this function will not notify dependents of a hange if the output is the
/// same. That is why the output of the function must implement [`PartialEq`].
///
/// To specify a custom comparison function, use [`create_selector_with`].
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 1);
/// let squared = create_selector(cx, move || state.get() * state.get());
/// assert_eq!(squared.get(), 1);
///
/// create_effect(cx, move || println!("x^2 = {}", squared.get()));
///
/// state.set(2); // Triggers the effect.
/// assert_eq!(squared.get(), 4);
///
/// state.set(-2); // Does not trigger the effect.
/// assert_eq!(squared.get(), 4);
/// # });
/// ```
pub fn create_selector<T>(cx: Scope, f: impl FnMut() -> T + 'static) -> Memo<T>
where
    T: PartialEq,
{
    create_selector_with(cx, f, PartialEq::eq)
}

/// An alternative to [`create_signal`] that uses a reducer to get the next
/// value.
///
/// It uses a reducer function that takes the previous value and a message and returns the next
/// value.
///
/// Returns a [`Memo`] and a dispatch function to send messages to the reducer.
///
/// # Params
/// * `initial` - The initial value of the state.
/// * `reducer` - A function that takes the previous value and a message and returns the next value.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// enum Msg {
///     Increment,
///     Decrement,
/// }
///
/// # create_root(|cx| {
/// let (state, dispatch) = create_reducer(cx, 0, |&state, msg: Msg| match msg {
///     Msg::Increment => state + 1,
///     Msg::Decrement => state - 1,
/// });
///
/// assert_eq!(state.get(), 0);
/// dispatch(Msg::Increment);
/// assert_eq!(state.get(), 1);
/// dispatch(Msg::Decrement);
/// assert_eq!(state.get(), 0);
/// # });
/// ```
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

/// Creates an effect on signals used inside the effect closure.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
///
/// create_effect(cx, move || {
///     println!("State changed. New state value = {}", state.get());
/// });
/// // Prints "State changed. New state value = 0"
///
/// state.set(1);
/// // Prints "State changed. New state value = 1"
/// # });
/// ```
///
/// `create_effect` should only be used for creating **side-effects**. It is generally not
/// recommended to update signal states inside an effect. You probably should be using a
/// [`create_memo`] instead.
pub fn create_effect(cx: Scope, f: impl FnMut() + 'static) {
    let _ = create_memo(cx, f);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn memo() {
        create_root(|cx| {
            let state = create_signal(cx, 0);
            let double = create_memo(cx, move || state.get() * 2);

            assert_eq!(double.get(), 0);
            state.set(1);
            assert_eq!(double.get(), 2);
            state.set(2);
            assert_eq!(double.get(), 4);
        });
    }

    /// Make sure value is memoized rather than executed on demand.
    #[test]
    fn memo_only_run_once() {
        create_root(|cx| {
            let state = create_signal(cx, 0);

            let counter = create_signal(cx, 0);
            let double = create_memo(cx, move || {
                counter.set(counter.get_untracked() + 1);
                state.get() * 2
            });

            assert_eq!(counter.get(), 1); // once for calculating initial derived state
            state.set(2);
            assert_eq!(counter.get(), 2);
            assert_eq!(double.get(), 4);
            assert_eq!(counter.get(), 2); // should still be 2 after access
        });
    }

    #[test]
    fn dependency_on_memo() {
        create_root(|cx| {
            let state = create_signal(cx, 0);
            let double = create_memo(cx, move || state.get() * 2);
            let quadruple = create_memo(cx, move || double.get() * 2);

            assert_eq!(quadruple.get(), 0);
            state.set(1);
            assert_eq!(quadruple.get(), 4);
        });
    }

    #[test]
    fn untracked_memo() {
        create_root(|cx| {
            let state = create_signal(cx, 1);
            let double = create_memo(cx, move || state.get_untracked() * 2);

            assert_eq!(double.get(), 2);
            state.set(2);
            assert_eq!(double.get(), 2); // double value should still be true because state.get()
                                         // was
                                         // inside untracked
        });
    }

    #[test]
    fn selector() {
        create_root(|cx| {
            let state = create_signal(cx, 0);
            let double = create_selector(cx, move || state.get() * 2);

            let counter = create_signal(cx, 0);
            create_effect(cx, move || {
                counter.set(counter.get_untracked() + 1);

                double.track();
            });
            assert_eq!(double.get(), 0);
            assert_eq!(counter.get(), 1);

            state.set(0);
            assert_eq!(double.get(), 0);
            assert_eq!(counter.get(), 1); // calling set_state should not trigger the effect

            state.set(2);
            assert_eq!(double.get(), 4);
            assert_eq!(counter.get(), 2);
        });
    }

    #[test]
    fn reducer() {
        create_root(|cx| {
            enum Msg {
                Increment,
                Decrement,
            }

            let (state, dispatch) = create_reducer(cx, 0, |state, msg: Msg| match msg {
                Msg::Increment => *state + 1,
                Msg::Decrement => *state - 1,
            });

            assert_eq!(state.get(), 0);
            dispatch(Msg::Increment);
            assert_eq!(state.get(), 1);
            dispatch(Msg::Decrement);
            assert_eq!(state.get(), 0);
            dispatch(Msg::Increment);
            dispatch(Msg::Increment);
            assert_eq!(state.get(), 2);
        });
    }

    #[test]
    fn memo_reducer() {
        create_root(|cx| {
            enum Msg {
                Increment,
                Decrement,
            }

            let (state, dispatch) = create_reducer(cx, 0, |state, msg: Msg| match msg {
                Msg::Increment => *state + 1,
                Msg::Decrement => *state - 1,
            });
            let doubled = create_memo(cx, move || state.get() * 2);

            assert_eq!(doubled.get(), 0);
            dispatch(Msg::Increment);
            assert_eq!(doubled.get(), 2);
            dispatch(Msg::Decrement);
            assert_eq!(doubled.get(), 0);
        });
    }
}
