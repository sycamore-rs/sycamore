//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::Deref;

use slotmap::new_key_type;

use crate::{Root, Scope};

new_key_type! { pub(crate) struct SignalId; }

/// Stores al the data associated with a signal.
pub(crate) struct SignalState {
    pub value: RefCell<Box<dyn Any>>,
    /// List of signals whose value this signal depends on.
    ///
    /// If any of the dependency signals are updated, this signal will automatically be updated as
    /// well.
    pub dependencies: Vec<SignalId>,
    /// List of signals which depend on the value of this signal.
    ///
    /// If this signal updates, any dependent signal will automatically be updated as well.
    pub dependents: Vec<SignalId>,
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
/// # create_root(|cx| {
/// let signal: Signal<i32> = create_signal(cx, 123);
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
/// # create_root(|cx| {
/// let signal = create_signal(cx, 1);
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
/// # create_root(|cx| {
/// let signal = create_signal(cx, 1);
/// // Note that we are accessing signal inside a closure in the line below. This will cause it to
/// // be automatically tracked and update our double value whenever signal is changed.
/// let double = create_memo(cx, move || signal.get() * 2);
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
pub fn create_signal<T>(cx: Scope, value: T) -> Signal<T> {
    let data = SignalState {
        value: RefCell::new(Box::new(value)),
        dependencies: Vec::new(),
        dependents: Vec::new(),
        update: None,
        changed_in_last_update: false,
        mark: Mark::None,
    };
    let key = cx.root.signals.borrow_mut().insert(data);
    // Add the signal the scope signal list so that it is properly dropped when the scope is
    // dropped.
    cx.get_data(|cx| cx.signals.push(key));
    Signal(ReadSignal {
        id: key,
        root: cx.root,
        _phantom: PhantomData,
    })
}

impl<T> ReadSignal<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data<U>(self, f: impl FnOnce(&SignalState) -> U) -> U {
        f(&mut self
            .root
            .signals
            .borrow()
            .get(self.id)
            .expect("signal is disposed"))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data_mut<U>(self, f: impl FnOnce(&mut SignalState) -> U) -> U {
        f(&mut self
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
    /// # create_root(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(state.get(), 1);
    ///
    /// // The signal is automatically tracked in the line below.
    /// let doubled = create_memo(cx, move || state.get());
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
    /// # create_root(|cx| {
    /// let greeting = create_signal(cx, "Hello".to_string());
    /// assert_eq!(greeting.get_clone(), "Hello".to_string());
    ///
    /// // The signal is automatically tracked in the line below.
    /// let hello_world = create_memo(cx, move || format!("{} World!", greeting.get_clone()));
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
                .downcast_ref::<T>()
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

    /// Update the value of the signal silently. This will not trigger any updates in dependent
    /// signals. As such, this is generally not recommended as it can easily lead to state
    /// inconsistencies.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn update_silent<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        self.0.get_data(|signal| {
            f(signal
                .value
                .borrow_mut()
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
}

/// We manually implement `Clone` + `Copy` for `Signal` so that we don't get extra bounds on `T`.
impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            root: self.root,
            _phantom: self._phantom,
        }
    }
}
impl<T> Copy for ReadSignal<T> {}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self(self.0)
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
