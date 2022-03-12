//! Derived and computed data.

use std::cell::Cell;

use crate::*;

impl<'a> Scope<'a> {
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
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    /// let double = || *state.get() * 2;
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
    /// Memos also incur a slightly higher performance penalty than simple derived signals.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    /// let double = ctx.create_memo(|| *state.get() * 2);
    ///
    /// assert_eq!(*double.get(), 0);
    /// state.set(1);
    /// assert_eq!(*double.get(), 2);
    /// # });
    /// ```
    pub fn create_memo<U: 'a>(self, f: impl FnMut() -> U + 'a) -> &'a ReadSignal<U> {
        self.create_selector_with(f, |_, _| false)
    }

    /// Creates a memoized value from some signals.
    /// Unlike [`create_memo`](Self::create_memo), this function will not notify dependents of a
    /// change if the output is the same. That is why the output of the function must implement
    /// [`PartialEq`].
    ///
    /// To specify a custom comparison function, use
    /// [`create_selector_with`](Self::create_selector_with).
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    /// let double = ctx.create_selector(|| *state.get() * 2);
    ///
    /// assert_eq!(*double.get(), 0);
    /// state.set(1);
    /// assert_eq!(*double.get(), 2);
    /// # });
    /// ```
    pub fn create_selector<U: PartialEq + 'a>(
        self,
        f: impl FnMut() -> U + 'a,
    ) -> &'a ReadSignal<U> {
        self.create_selector_with(f, PartialEq::eq)
    }

    /// Creates a memoized value from some signals.
    /// Unlike [`create_memo`](Self::create_memo), this function will not notify dependents of a
    /// change if the output is the same.
    ///
    /// It takes a comparison function to compare the old and new value, which returns `true` if
    /// they are the same and `false` otherwise.
    ///
    /// To use the type's [`PartialEq`] implementation instead of a custom function, use
    /// [`create_selector`](Self::create_selector).
    pub fn create_selector_with<U: 'a>(
        self,
        mut f: impl FnMut() -> U + 'a,
        eq_f: impl Fn(&U, &U) -> bool + 'a,
    ) -> &'a ReadSignal<U> {
        let signal: Rc<Cell<Option<&Signal<U>>>> = Default::default();

        self.create_effect({
            let signal = signal.clone();
            move || {
                let new = f();
                if let Some(signal) = signal.get() {
                    // Check if new value is different from old value.
                    if !eq_f(&new, &*signal.get_untracked()) {
                        signal.set(new)
                    }
                } else {
                    signal.set(Some(self.create_signal(new)))
                }
            }
        });

        signal.get().unwrap()
    }

    /// An alternative to [`create_signal`](Self::create_signal) that uses a reducer to get the next
    /// value.
    ///
    /// It uses a reducer function that takes the previous value and a message and returns the next
    /// value.
    ///
    /// Returns a [`ReadSignal`] and a dispatch function to send messages to the reducer.
    ///
    /// # Params
    /// * `initial` - The initial value of the state.
    /// * `reducer` - A function that takes the previous value and a message and returns the next
    ///   value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// enum Msg {
    ///     Increment,
    ///     Decrement,
    /// }
    ///
    /// # create_scope_immediate(|ctx| {
    /// let (state, dispatch) = ctx.create_reducer(0, |state, msg: Msg| match msg {
    ///     Msg::Increment => *state + 1,
    ///     Msg::Decrement => *state - 1,
    /// });
    ///
    /// assert_eq!(*state.get(), 0);
    /// dispatch(Msg::Increment);
    /// assert_eq!(*state.get(), 1);
    /// dispatch(Msg::Decrement);
    /// assert_eq!(*state.get(), 0);
    /// # });
    /// ```
    pub fn create_reducer<U, Msg>(
        self,
        initial: U,
        reduce: impl Fn(&U, Msg) -> U + 'a,
    ) -> (&'a ReadSignal<U>, impl Fn(Msg) + 'a) {
        let memo = self.create_signal(initial);

        let dispatcher = move |msg| {
            memo.set(reduce(&memo.get_untracked(), msg));
        };

        (&*memo, dispatcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = ctx.create_memo(|| *state.get() * 2);

            assert_eq!(*double.get(), 0);
            state.set(1);
            assert_eq!(*double.get(), 2);
            state.set(2);
            assert_eq!(*double.get(), 4);
        });
    }

    /// Make sure value is memoized rather than executed on demand.
    #[test]
    fn memo_only_run_once() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            let counter = ctx.create_signal(0);
            let double = ctx.create_memo(|| {
                counter.set(*counter.get_untracked() + 1);
                *state.get() * 2
            });

            assert_eq!(*counter.get(), 1); // once for calculating initial derived state
            state.set(2);
            assert_eq!(*counter.get(), 2);
            assert_eq!(*double.get(), 4);
            assert_eq!(*counter.get(), 2); // should still be 2 after access
        });
    }

    #[test]
    fn dependency_on_memo() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = ctx.create_memo(|| *state.get() * 2);
            let quadruple = ctx.create_memo(|| *double.get() * 2);

            assert_eq!(*quadruple.get(), 0);
            state.set(1);
            assert_eq!(*quadruple.get(), 4);
        });
    }

    #[test]
    fn untracked_memo() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(1);
            let double = ctx.create_memo(|| *state.get_untracked() * 2);

            assert_eq!(*double.get(), 2);
            state.set(2);
            assert_eq!(*double.get(), 2); // double value should still be true because state.get()
                                          // was
                                          // inside untracked
        });
    }

    #[test]
    fn selector() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = ctx.create_selector(|| *state.get() * 2);

            let counter = ctx.create_signal(0);
            ctx.create_effect(|| {
                counter.set(*counter.get_untracked() + 1);

                double.track();
            });
            assert_eq!(*double.get(), 0);
            assert_eq!(*counter.get(), 1);

            state.set(0);
            assert_eq!(*double.get(), 0);
            assert_eq!(*counter.get(), 1); // calling set_state should not trigger the effect

            state.set(2);
            assert_eq!(*double.get(), 4);
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn reducer() {
        create_scope_immediate(|ctx| {
            enum Msg {
                Increment,
                Decrement,
            }

            let (state, dispatch) = ctx.create_reducer(0, |state, msg: Msg| match msg {
                Msg::Increment => *state + 1,
                Msg::Decrement => *state - 1,
            });

            assert_eq!(*state.get(), 0);
            dispatch(Msg::Increment);
            assert_eq!(*state.get(), 1);
            dispatch(Msg::Decrement);
            assert_eq!(*state.get(), 0);
            dispatch(Msg::Increment);
            dispatch(Msg::Increment);
            assert_eq!(*state.get(), 2);
        });
    }

    #[test]
    fn memo_reducer() {
        create_scope_immediate(|ctx| {
            enum Msg {
                Increment,
                Decrement,
            }

            let (state, dispatch) = ctx.create_reducer(0, |state, msg: Msg| match msg {
                Msg::Increment => *state + 1,
                Msg::Decrement => *state - 1,
            });
            let doubled = ctx.create_memo(|| *state.get() * 2);

            assert_eq!(*doubled.get(), 0);
            dispatch(Msg::Increment);
            assert_eq!(*doubled.get(), 2);
            dispatch(Msg::Decrement);
            assert_eq!(*doubled.get(), 0);
        });
    }
}
