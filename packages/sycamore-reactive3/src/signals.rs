//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{AddAssign, Deref, DivAssign, MulAssign, RemAssign, SubAssign};

use slotmap::new_key_type;

use crate::{create_memo, EffectId, Memo, Root};

new_key_type! { pub(crate) struct SignalId; }

/// Stores al the data associated with a signal.
pub(crate) struct SignalState {
    /// The value of the signal. This is wrapped inside an [`Option`] because this will allow us to
    /// temporarily take the value out while we run signal updates so that we do not have to hold
    /// on mutably to `root.signals`.
    pub value: RefCell<Option<Box<dyn Any>>>,
    /// List of signals whose value this signal depends on.
    ///
    /// If any of the dependency signals are updated, this signal will automatically be updated as
    /// well.
    pub dependencies: Vec<SignalId>,
    /// List of signals which depend on the value of this signal.
    ///
    /// If this signal updates, any dependent signal will automatically be updated as well.
    pub dependents: Vec<SignalId>,
    pub effect_dependents: Vec<EffectId>,
    /// A callback that automatically updates the value of the signal when one of its dependencies
    /// updates.
    ///
    /// A signal created using [`create_signal`] can be thought of as a signal which is never
    /// autoamtically updated. A signal created using [`create_memo`] can be thought of as a signal
    /// that is always automatically updated.
    ///
    /// Note that the update function takes a `&mut dyn Any`. The update function should only ever
    /// set this value to the same type as the signal.
    ///
    /// The return value of the update function is a `bool`. This should represent whether the
    /// value has been changed or not. If `true` is returned, then dependent signals will also be
    /// updated.
    #[allow(clippy::type_complexity)]
    pub update: Option<Box<dyn FnMut(&mut Box<dyn Any>) -> bool>>,
    /// An internal state used by `propagate_updates`. This should be `true` if the signal has been
    /// updated in the last call to `propagate_updates` and was reacheable from the start node.
    /// This is to stop propagation to dependents if this value is `false`.
    pub changed_in_last_update: bool,
    /// An internal state used by `propagate_updates`. This is used in DFS to detect cycles.
    pub mark: Mark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    /// Mark when DFS reaches node.
    Temp,
    /// Mark when DFS is done with node.
    Permanent,
    /// No mark.
    None,
}

/// A read-only reactive value.
///
/// Unlike the difference between Rust's shared and mutable-references (`&T` and `&mut`), the
/// underlying data is not immutable. The data can be updated with the corresponding [`Signal`]
/// (which has mutable access) and will show up in the `ReadSignal` as well.
///
/// A `ReadSignal` can be simply obtained by dereferencing a [`Signal`]. In fact, every [`Signal`]
/// is a `ReadSignal` with additional write abilities!
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|| {
/// let signal: Signal<i32> = create_signal(123);
/// let read_signal: ReadSignal<i32> = *signal;
/// assert_eq!(read_signal.get(), 123);
/// signal.set(456);
/// assert_eq!(read_signal.get(), 456);
/// // read_signal.set(789); // <-- This is not allowed!
/// # });
/// ```
///
/// See [`create_signal`] for more information.
pub struct ReadSignal<T: 'static> {
    pub(crate) id: SignalId,
    root: &'static Root,
    _phantom: PhantomData<T>,
}

/// A reactive value that can be read and written to.
///
/// This is the writable analog of [`ReadSignal`].
///
/// See [`create_signal`] for more information.
pub struct Signal<T: 'static>(pub(crate) ReadSignal<T>);

/// Create a new [`Signal`].
///
/// Signals are reactive atoms, pieces of state that can be read and written to and which will
/// automatically update anything which depend on them.
///
/// # Usage
/// The simplest way to use a signal is by using [`.get()`](ReadSignal::get) and
/// [`.set(...)`](Signal::set). However, this only works if the value implements [`Copy`]. If
/// we wanted to store something that doesn't implement [`Copy`] but implements [`Clone`] instead,
/// say a [`String`], we can use [`.get_clone()`](ReadSignal::get_clone) which will automatically
/// clone the value for us.
///
/// ```rust
/// # use sycamore_reactive3::*;
/// # create_root(|| {
/// let signal = create_signal(1);
/// signal.get(); // Should return 1.
/// signal.set(2);
/// signal.get(); // Should return 2.
/// # });
/// ```
///
/// There are many other ways of getting and setting signals, such as
/// [`.with(...)`](ReadSignal::with) and [`.update(...)`](Signal::update) which can access the
/// signal even if it does not implement [`Clone`] or if you simply don't want to pay the
/// performance overhead of cloning your value everytime you read it.
///
/// # Reactivity
/// What makes signals so powerful, as opposed to some other wrapper type like [`RefCell`] is the
/// automatic dependency tracking. This means that accessing a signal will automatically add it as
/// a dependency in certain contexts (such as inside a [`create_memo`](crate::create_memo)) which
/// allows us to update related state whenever the signal is changed.
///
/// ```rust
/// # use sycamore_reactive3::*;
/// # create_root(|| {
/// let signal = create_signal(1);
/// // Note that we are accessing signal inside a closure in the line below. This will cause it to
/// // be automatically tracked and update our double value whenever signal is changed.
/// let double = create_memo(move || signal.get() * 2);
/// double.get(); // Should return 2.
/// signal.set(2);
/// double.get(); // Should return 4. Notice how this value was updated automatically when we
///               // modified signal. This way, we can rest assured that all our state will be
///               // consistent at all times!
/// # });
/// ```
///
/// # Ownership
/// Signals are always associated with a [`Scope`]. This is what performs the memory management for
/// the actual value of the signal. What is returned from this function is just a handle/reference
/// to the signal allocted in the [`Scope`]. This allows us to freely copy this handle around and
/// use it in closures and event handlers without worrying about ownership of the signal.
///
/// This is why in the above example, we could access `signal` even after it was moved in to the
/// closure of the `create_memo`.
#[cfg_attr(debug_assertions, track_caller)]
pub fn create_signal<T>(value: T) -> Signal<T> {
    let root = Root::get_global();
    let data = SignalState {
        value: RefCell::new(Some(Box::new(value))),
        dependencies: Vec::new(),
        effect_dependents: Vec::new(),
        dependents: Vec::new(),
        update: None,
        changed_in_last_update: false,
        mark: Mark::None,
    };
    let key = root.signals.borrow_mut().insert(data);
    // Add the signal the scope signal list so that it is properly dropped when the scope is
    // dropped.
    root.scopes.borrow_mut()[root.current_scope.get()]
        .signals
        .push(key);
    Signal(ReadSignal {
        id: key,
        root,
        _phantom: PhantomData,
    })
}

impl<T> ReadSignal<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data<U>(self, f: impl FnOnce(&SignalState) -> U) -> U {
        f(self
            .root
            .signals
            .borrow()
            .get(self.id)
            .expect("signal is disposed"))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data_mut<U>(self, f: impl FnOnce(&mut SignalState) -> U) -> U {
        f(self
            .root
            .signals
            .borrow_mut()
            .get_mut(self.id)
            .expect("signal is disposed"))
    }

    /// Get the value of the signal without tracking it. The type must implement [`Copy`]. If this
    /// is not the case, use [`ReadSignal::get_clone_untracked`] or [`ReadSignal::with_untracked`]
    /// instead.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_untracked(self) -> T
    where
        T: Copy,
    {
        self.with_untracked(|value| *value)
    }

    /// Get the value of the signal without tracking it. The type is [`Clone`]-ed automatically.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_clone_untracked(self) -> T
    where
        T: Clone,
    {
        self.with_untracked(Clone::clone)
    }

    /// Get the value of the signal. The type must implement [`Copy`]. If this is not the case, use
    /// [`ReadSignal::get_clone_untracked`] or [`ReadSignal::with_untracked`] instead.
    ///
    /// When called inside a reactive scope, the signal will be automatically tracked.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive3::*;
    /// # create_root(|| {
    /// let state = create_signal(0);
    /// assert_eq!(state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(state.get(), 1);
    ///
    /// // The signal is automatically tracked in the line below.
    /// let doubled = create_memo(move || state.get());
    /// # });
    /// ```
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get(self) -> T
    where
        T: Copy,
    {
        self.track();
        self.get_untracked()
    }

    /// Get the value of the signal. The type is [`Clone`]-ed automatically.
    ///
    /// When called inside a reactive scope, the signal will be automatically tracked.
    ///
    /// If the value implements [`Copy`], you should use [`ReadSignal::get`] instead.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive3::*;
    /// # create_root(|| {
    /// let greeting = create_signal("Hello".to_string());
    /// assert_eq!(greeting.get_clone(), "Hello".to_string());
    ///
    /// // The signal is automatically tracked in the line below.
    /// let hello_world = create_memo(move || format!("{} World!", greeting.get_clone()));
    /// assert_eq!(hello_world.get_clone(), "Hello World!");
    ///
    /// greeting.set("Goodbye".to_string());
    /// assert_eq!(greeting.get_clone(), "Goodbye".to_string());
    /// assert_eq!(hello_world.get_clone(), "Goodbye World!");
    /// # });
    /// ```
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_clone(self) -> T
    where
        T: Clone,
    {
        self.track();
        self.get_clone_untracked()
    }

    /// Get a value from the signal without tracking it.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn with_untracked<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.get_data(|signal| {
            f(signal
                .value
                .borrow()
                .as_ref()
                .expect("cannot get value while updating")
                .downcast_ref()
                .expect("wrong signal type in slotmap"))
        })
    }

    /// Get a value from the signal.
    ///
    /// When called inside a reactive scope, the signal will be automatically tracked.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn with<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.track();
        self.with_untracked(f)
    }

    /// Track the signal in the current reactive scope. This is done automatically when calling
    /// [`ReadSignal::get`] and other similar methods.
    pub fn track(self) {
        if let Some(tracker) = &mut *self.root.tracker.borrow_mut() {
            tracker.dependencies.push(self.id);
        }
    }
}

impl<T> Signal<T> {
    /// Silently set a new value for the signal. This will not trigger any updates in dependent
    /// signals. As such, this is generally not recommended as it can easily lead to state
    /// inconsistencies.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_silent(self, new: T) -> T {
        self.update_silent(|val| std::mem::replace(val, new))
    }

    /// Set a new value for the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(self, new: T) -> T {
        self.update(|val| std::mem::replace(val, new))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn take_silent(self) -> T
    where
        T: Default,
    {
        self.set_silent(T::default())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn take(self) -> T
    where
        T: Default,
    {
        self.set(T::default())
    }

    /// Update the value of the signal silently. This will not trigger any updates in dependent
    /// signals. As such, this is generally not recommended as it can easily lead to state
    /// inconsistencies.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn update_silent<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        self.0.get_data(|signal| {
            f(signal
                .value
                .borrow_mut()
                .as_mut()
                .expect("cannot update while updating")
                .downcast_mut()
                .expect("wrong signal type in slotmap"))
        })
    }

    /// Update the value of the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn update<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        let ret = self.update_silent(f);
        self.0.root.propagate_updates(self.0.id);
        ret
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_fn_silent(self, f: impl FnOnce(&T) -> T) {
        self.update_silent(move |val| *val = f(val));
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_fn(self, f: impl FnOnce(&T) -> T) {
        self.update_silent(move |val| *val = f(val));
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn map<U>(self, mut f: impl FnMut(&T) -> U + 'static) -> Memo<U> {
        create_memo(move || self.with(&mut f))
    }

    pub fn split(self) -> (ReadSignal<T>, impl Fn(T) -> T) {
        (*self, move |value| self.set(value))
    }
}

/// We manually implement `Clone` + `Copy` for `Signal` so that we don't get extra bounds on `T`.
impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ReadSignal<T> {}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Signal<T> {}

impl<T> Deref for Signal<T> {
    type Target = ReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Formatting implementations for `ReadSignal` and `Signal`.
impl<T: fmt::Debug> fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}
impl<T: fmt::Debug> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}

impl<T: fmt::Display> fmt::Display for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}
impl<T: fmt::Display> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}

#[cfg(feature = "nightly")]
impl<T: Copy> FnOnce<()> for ReadSignal<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get()
    }
}

impl<T: AddAssign<Rhs>, Rhs> AddAssign<Rhs> for Signal<T> {
    fn add_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this += rhs);
    }
}
impl<T: SubAssign<Rhs>, Rhs> SubAssign<Rhs> for Signal<T> {
    fn sub_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this -= rhs);
    }
}
impl<T: MulAssign<Rhs>, Rhs> MulAssign<Rhs> for Signal<T> {
    fn mul_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this *= rhs);
    }
}
impl<T: DivAssign<Rhs>, Rhs> DivAssign<Rhs> for Signal<T> {
    fn div_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this /= rhs);
    }
}
impl<T: RemAssign<Rhs>, Rhs> RemAssign<Rhs> for Signal<T> {
    fn rem_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this %= rhs);
    }
}

/// We need to implement this again for `Signal` despite `Signal` deref-ing to `ReadSignal` since
/// we also have another implementation of `FnOnce` for `Signal`.
#[cfg(feature = "nightly")]
impl<T: Copy> FnOnce<()> for Signal<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        self.get()
    }
}

#[cfg(feature = "nightly")]
impl<T: Copy> FnOnce<(T,)> for Signal<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, (val,): (T,)) -> Self::Output {
        self.set(val)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn signal() {
        create_root(|| {
            let state = create_signal(0);
            assert_eq!(state.get(), 0);

            state.set(1);
            assert_eq!(state.get(), 1);

            state.set_fn(|n| *n + 1);
            assert_eq!(state.get(), 2);
        });
    }

    #[test]
    fn signal_composition() {
        create_root(|| {
            let state = create_signal(0);
            let double = || state.get() * 2;

            assert_eq!(double(), 0);
            state.set(1);
            assert_eq!(double(), 2);
        });
    }

    #[test]
    fn set_silent_signal() {
        create_root(|| {
            let state = create_signal(0);
            let double = state.map(|&x| x * 2);

            assert_eq!(double.get(), 0);
            state.set_silent(1);
            assert_eq!(double.get(), 0); // double value is unchanged.

            state.set_fn_silent(|n| n + 1);
            assert_eq!(double.get(), 0); // double value is unchanged.
        });
    }

    #[test]
    fn read_signal() {
        create_root(|| {
            let state = create_signal(0);
            let readonly: ReadSignal<i32> = *state;

            assert_eq!(readonly.get(), 0);
            state.set(1);
            assert_eq!(readonly.get(), 1);
        });
    }

    #[test]
    fn map_signal() {
        create_root(|| {
            let state = create_signal(0);
            let double = state.map(|&x| x * 2);

            assert_eq!(double.get(), 0);
            state.set(1);
            assert_eq!(double.get(), 2);
        });
    }

    #[test]
    fn take_signal() {
        create_root(|| {
            let state = create_signal(123);

            let x = state.take();
            assert_eq!(x, 123);
            assert_eq!(state.get(), 0);
        });
    }

    #[test]
    fn take_silent_signal() {
        create_root(|| {
            let state = create_signal(123);
            let double = state.map(|&x| x * 2);

            // Do not trigger subscribers.
            state.take_silent();
            assert_eq!(state.get(), 0);
            assert_eq!(double.get(), 246);
        });
    }

    #[test]
    fn signal_split() {
        create_root(|| {
            let (state, set_state) = create_signal(0).split();
            assert_eq!(state.get(), 0);

            set_state(1);
            assert_eq!(state.get(), 1);
        });
    }

    #[test]
    fn signal_display() {
        create_root(|| {
            let signal = create_signal(0);
            assert_eq!(format!("{signal}"), "0");
            let read_signal: ReadSignal<_> = *signal;
            assert_eq!(format!("{read_signal}"), "0");
            let memo = create_memo(|| 0);
            assert_eq!(format!("{memo}"), "0");
        });
    }

    #[test]
    fn signal_debug() {
        create_root(|| {
            let signal = create_signal(0);
            assert_eq!(format!("{signal:?}"), "0");
            let read_signal: ReadSignal<_> = *signal;
            assert_eq!(format!("{read_signal:?}"), "0");
            let memo = create_memo(|| 0);
            assert_eq!(format!("{memo:?}"), "0");
        });
    }

    #[test]
    fn signal_add_assign_update() {
        create_root(|| {
            let mut signal = create_signal(0);
            let counter = create_signal(0);
            create_effect(move || {
                signal.track();
                counter.set(counter.get_untracked() + 1);
            });
            signal += 1;
            signal -= 1;
            signal *= 1;
            signal /= 1;
            assert_eq!(counter.get(), 5);
        });
    }

    #[test]
    fn signal_update() {
        create_root(|| {
            let signal = create_signal("Hello ".to_string());
            let counter = create_signal(0);
            create_effect(move || {
                signal.track();
                counter.set(counter.get_untracked() + 1);
            });
            signal.update(|value| value.push_str("World!"));
            assert_eq!(signal.get_clone(), "Hello World!");
            assert_eq!(counter.get(), 2);
        });
    }
}
