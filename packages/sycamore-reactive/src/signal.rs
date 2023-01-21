//! Signals - The building blocks of reactivity.

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{AddAssign, Deref, DerefMut, DivAssign, MulAssign, SubAssign};

use crate::effect::EFFECTS;
use crate::*;

type WeakEffectCallback = Weak<RefCell<dyn FnMut()>>;
type EffectCallbackPtr = *const RefCell<dyn FnMut()>;

pub(crate) type SignalEmitterInner = RefCell<IndexMap<EffectCallbackPtr, WeakEffectCallback>>;

/// A struct for managing subscriptions to signals.
#[derive(Default, Clone)]
pub struct SignalEmitter(pub(crate) Rc<SignalEmitterInner>);
impl std::fmt::Debug for SignalEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalEmitter").finish()
    }
}

#[derive(Default, Clone)]
pub(crate) struct WeakSignalEmitter(pub Weak<SignalEmitterInner>);

impl WeakSignalEmitter {
    pub fn upgrade(&self) -> Option<SignalEmitter> {
        self.0.upgrade().map(SignalEmitter)
    }
}

impl SignalEmitter {
    pub(crate) fn downgrade(&self) -> WeakSignalEmitter {
        WeakSignalEmitter(Rc::downgrade(&self.0))
    }

    /// Adds a callback to the subscriber list. If the callback is already a subscriber, does
    /// nothing.
    pub(crate) fn subscribe(&self, cb: WeakEffectCallback) {
        self.0.borrow_mut().insert(cb.as_ptr(), cb);
    }

    /// Removes a callback from the subscriber list. If the callback is not a subscriber, does
    /// nothing.
    pub(crate) fn unsubscribe(&self, cb: EffectCallbackPtr) {
        self.0.borrow_mut().remove(&cb);
    }

    /// Track the current signal in the effect scope.
    pub fn track(&self) {
        EFFECTS.with(|effects| {
            if let Some(last) = effects.borrow().last() {
                // SAFETY: See guarantee on EffectState within EFFECTS.
                let last = unsafe { &mut **last };
                last.add_dependency(self.downgrade());
            }
        });
    }

    /// Calls all the subscribers without modifying the state.
    /// This can be useful when using patterns such as inner mutability where the state updated will
    /// not be automatically triggered. In the general case, however, it is preferable to use
    /// [`Signal::set()`] instead.
    ///
    /// This will also re-compute all the subscribers of this signal by calling all the dependency
    /// callbacks.
    pub fn trigger_subscribers(&self) {
        // Reset subscribers to prevent modifying the subscriber list while it is being read from.
        // We can completely wipe out the subscriber list because it will be constructed again when
        // each callback is called.
        let subscribers = self.0.take().into_values();
        // Subscriber order is reversed because effects attach subscribers at the end of the
        // effect scope. This will ensure that outer effects re-execute before inner effects,
        // preventing inner effects from running twice.
        for subscriber in subscribers.rev() {
            // subscriber might have already been destroyed in the case of nested effects.
            if let Some(callback) = subscriber.upgrade() {
                // Call the callback.
                callback.borrow_mut()();
            }
        }
    }
}

/// A read-only [`Signal`].
///
/// Unlike Rust's shared-reference (`&T`), the underlying data is not immutable. The data can be
/// updated with the corresponding [`Signal`] (which has mutable access) and will show up in the
/// `ReadSignal` as well.
///
/// A `ReadSignal` can be simply obtained by dereferencing a [`Signal`]. In fact, every [`Signal`]
/// is a `ReadSignal` with additional write abilities!
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// # create_scope_immediate(|cx| {
/// let signal: &Signal<i32> = create_signal(cx, 123);
/// let read_signal: &ReadSignal<i32> = &*signal;
/// # });
/// ```
pub struct ReadSignal<T> {
    value: RefCell<Rc<T>>,
    emitter: SignalEmitter,
}

impl<T> ReadSignal<T> {
    /// Get the current value of the state. When called inside a reactive scope, calling this will
    /// add itself to the scope's dependencies.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    #[must_use = "to only subscribe the signal without using the value, use .track() instead"]
    pub fn get(&self) -> Rc<T> {
        self.emitter.track();
        self.value.borrow().clone()
    }

    /// Get the current value of the state, without tracking this as a dependency if inside a
    /// reactive context.
    ///
    /// # Example
    ///
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, 1);
    /// let double = create_memo(cx, || *state.get_untracked() * 2);
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// // double value should still be old value because state was untracked
    /// assert_eq!(*double.get(), 2);
    /// # });
    /// ```
    #[must_use = "discarding the returned value does nothing"]
    pub fn get_untracked(&self) -> Rc<T> {
        self.value.borrow().clone()
    }

    /// Creates a mapped [`ReadSignal`]. This is equivalent to using
    /// [`create_memo`].
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, 1);
    /// let double = state.map(cx, |&x| x * 2);
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// assert_eq!(*double.get(), 4);
    /// # });
    /// ```
    #[must_use]
    pub fn map<'a, U: 'static>(
        &'a self,
        cx: Scope<'a>,
        mut f: impl FnMut(&T) -> U + 'a,
    ) -> &'a ReadSignal<U> {
        create_memo(cx, move || f(&self.get()))
    }

    /// When called inside a reactive scope, calling this will add itself to the scope's
    /// dependencies.
    ///
    /// To both track and get the value of the signal, use [`ReadSignal::get`] instead.
    pub fn track(&self) {
        self.emitter.track();
    }
}

/// A container of reactive state that can be updated and subscribed to.
///
/// # Example
/// Creating a `Signal` requires a reactive [`Scope`]. Generally, you can use the `cx` parameter
/// obtained from your component or from inside a [`create_effect_scoped`].
/// ```
/// # use sycamore_reactive::*;
/// # create_scope_immediate(|cx| {
/// let signal = create_signal(cx, 123);    // A signal of type `i32`.
/// let signal = create_signal(cx, true);   // A signal of type `bool`.
/// let signal = create_signal(cx, "abc");  // A signal of type `&str`.
/// # });
/// ```
pub struct Signal<T>(ReadSignal<T>);

impl<T> Signal<T> {
    /// Create a new `Signal` with the specified value.
    ///
    /// This method is internal because it should not be possible to create a signal without a
    /// reactive scope.
    pub(crate) fn new(value: T) -> Self {
        Self(ReadSignal {
            value: RefCell::new(Rc::new(value)),
            emitter: Default::default(),
        })
    }

    /// Set the current value of the state.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    pub fn set(&self, value: T) {
        self.set_silent(value);
        self.trigger_subscribers();
    }

    /// Set the value of the state using a function that receives the current value.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set_fn(|n| n + 1);
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    pub fn set_fn<F: Fn(&T) -> T>(&self, f: F) {
        self.set(f(&self.get_untracked()));
    }

    /// Set the value of the state using a function that can mutate the value.
    ///
    /// # Panics
    ///
    /// This method doesn't require a clone, but it panics if there are existing
    /// borrows of the underlying value. That shouldn't happen in practice, since
    /// this borrow ends at the end of the method call.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, vec![]);
    /// assert_eq!(*state.get(), vec![]);
    ///
    /// state.set_fn_mut(|v| v.push(1));
    /// assert_eq!(*state.get(), vec![1]);
    /// # });
    /// ```
    pub fn set_fn_mut<F: Fn(&mut T)>(&self, f: F) {
        self.set_fn_mut_silent(f);
        self.trigger_subscribers();
    }

    /// Set the current value of the state wrapped in a [`Rc`]. Unlike [`Signal::set()`], this
    /// method accepts the value wrapped in a [`Rc`] because the underlying storage is already using
    /// [`Rc`], thus preventing an unnecessary clone.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use std::rc::Rc;
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set_rc(Rc::new(1));
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    pub fn set_rc(&self, value: Rc<T>) {
        self.set_rc_silent(value);
        self.trigger_subscribers();
    }

    /// Set the current value of the state _without_ triggering subscribers.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn set_silent(&self, value: T) {
        self.set_rc_silent(Rc::new(value));
    }

    /// Set the value of the state using a function that receives the current value _without_
    /// triggering subscribers.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn set_fn_silent<F: Fn(&T) -> T>(&self, f: F) {
        self.set_silent(f(&self.get_untracked()));
    }

    /// Set the value of the state using a function that can mutate the value _without_
    /// triggering subscribers.
    ///
    /// # Panics
    ///
    /// This method doesn't require a clone, but it panics if there are existing
    /// borrows of the underlying value. That shouldn't happen in practice, since
    /// this borrow ends at the end of the method call.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn set_fn_mut_silent<F: Fn(&mut T)>(&self, f: F) {
        f(Rc::get_mut(self.0.value.borrow_mut().deref_mut()).unwrap());
    }

    /// Set the current value of the state wrapped in a [`Rc`] _without_ triggering subscribers.
    ///
    /// See the documentation for [`Signal::set_rc()`] for more information.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn set_rc_silent(&self, value: Rc<T>) {
        *self.0.value.borrow_mut() = value;
    }

    /// Split a signal into getter and setter handles.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let (state, set_state) = create_signal(cx, 0).split();
    /// assert_eq!(*state(), 0);
    ///
    /// set_state(1);
    /// assert_eq!(*state(), 1);
    /// # });
    /// ```
    pub fn split(&self) -> (impl Fn() -> Rc<T> + Copy + '_, impl Fn(T) + Copy + '_) {
        let getter = move || self.get();
        let setter = move |x| self.set(x);
        (getter, setter)
    }

    /// Calls all the subscribers without modifying the state.
    /// This can be useful when using patterns such as inner mutability where the state updated will
    /// not be automatically triggered. In the general case, however, it is preferable to use
    /// [`Signal::set()`] instead.
    ///
    /// This will also re-compute all the subscribers of this signal by calling all the dependency
    /// callbacks.
    pub fn trigger_subscribers(&self) {
        self.0.emitter.trigger_subscribers()
    }
}

/// A mutable reference for modifying a [`Signal`].
///
/// Construct this using the [`Signal::modify()`] method.
#[derive(Debug)]
pub struct Modify<'a, T>(Option<T>, &'a Signal<T>);

impl<'a, T> Deref for Modify<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
impl<'a, T> DerefMut for Modify<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

/// When the mutable handle is dropped, update the [`Signal`].
impl<T> Drop for Modify<'_, T> {
    fn drop(&mut self) {
        self.1.set(self.0.take().unwrap())
    }
}

impl<T: Clone> Signal<T> {
    /// Return a mutable handle to make it easier to mutate the inner value.
    /// This requires the inner type to implement [`Clone`].
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|cx| {
    /// let state = create_signal(cx, "Hello ".to_string());
    /// state.modify().push_str("World!");
    /// assert_eq!(*state.get(), "Hello World!");
    /// # });
    /// ```
    pub fn modify(&self) -> Modify<T> {
        Modify(Some(self.value.borrow().as_ref().clone()), self)
    }
}

impl<T: Default> Signal<T> {
    /// Take the current value out and replace it with the default value.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    pub fn take(&self) -> Rc<T> {
        let ret = self.0.value.take();
        self.trigger_subscribers();
        ret
    }

    /// Take the current value out and replace it with the default value _without_ triggering
    /// subscribers.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn take_silent(&self) -> Rc<T> {
        self.0.value.take()
    }
}

impl<T> Deref for Signal<T> {
    type Target = ReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AddAssign + Copy> AddAssign<T> for &Signal<T> {
    fn add_assign(&mut self, other: T) {
        let mut value = **self.0.value.borrow();
        value += other;
        self.set(value);
    }
}
impl<T: SubAssign + Copy> SubAssign<T> for &Signal<T> {
    fn sub_assign(&mut self, other: T) {
        let mut value = **self.0.value.borrow();
        value -= other;
        self.set(value);
    }
}
impl<T: MulAssign + Copy> MulAssign<T> for &Signal<T> {
    fn mul_assign(&mut self, other: T) {
        let mut value = **self.0.value.borrow();
        value *= other;
        self.set(value);
    }
}
impl<T: DivAssign + Copy> DivAssign<T> for &Signal<T> {
    fn div_assign(&mut self, other: T) {
        let mut value = **self.0.value.borrow();
        value /= other;
        self.set(value);
    }
}

/// A trait that is implemented for all [`ReadSignal`]s regardless of the type parameter.
pub trait AnyReadSignal<'a> {
    /// Call the [`ReadSignal::track`] method.
    fn track(&self);
}
impl<'a, T> AnyReadSignal<'a> for RcSignal<T> {
    fn track(&self) {
        self.deref().deref().track();
    }
}
impl<'a, T> AnyReadSignal<'a> for Signal<T> {
    fn track(&self) {
        self.deref().track();
    }
}
impl<'a, T> AnyReadSignal<'a> for ReadSignal<T> {
    fn track(&self) {
        self.track();
    }
}

/// Create a new [`Signal`] under the current [`Scope`].
/// The created signal lasts as long as the scope and cannot be used outside of the scope.
///
/// # Signal lifetime
///
/// The lifetime of the returned signal is the same as the [`Scope`].
/// As such, the signal cannot escape the [`Scope`].
///
/// ```compile_fail
/// # use sycamore_reactive::*;
/// let mut outer = None;
/// create_scope_immediate(|cx| {
///     let signal = create_signal(cx, 0);
///     outer = Some(signal);
/// });
/// ```
pub fn create_signal<T: 'static>(cx: Scope, value: T) -> &Signal<T> {
    let signal = Signal::new(value);
    create_ref(cx, signal)
}

/// Create a new [`Signal`] under the current [`Scope`] but with an initial value wrapped in a
/// [`Rc`]. This is useful to avoid having to clone a value that is already wrapped in a [`Rc`] when
/// creating a new signal. Otherwise, this is identical to [`create_signal`].
pub fn create_signal_from_rc<T: 'static>(cx: Scope, value: Rc<T>) -> &Signal<T> {
    let signal = Signal(ReadSignal {
        value: RefCell::new(value),
        emitter: Default::default(),
    });
    create_ref(cx, signal)
}

/// A signal that is not bound to a [`Scope`].
///
/// Sometimes, it is useful to have a signal that can escape the enclosing [reactive scope](Scope).
/// However, this cannot be achieved simply with [`create_signal`] because the resulting
/// [`Signal`] is tied to the [`Scope`] by it's lifetime. The [`Signal`] can only live as long as
/// the [`Scope`].
///
/// With [`RcSignal`] on the other hand, the lifetime is not tied to a [`Scope`]. Memory is managed
/// using a reference-counted smart pointer ([`Rc`]). What this means is that [`RcSignal`] cannot
/// implement the [`Copy`] trait and therefore needs to be manually cloned into all closures where
/// it is used.
///
/// In general, [`create_signal`] should be preferred, both for performance and ergonomics.
///
/// # Usage
///
/// To create a [`RcSignal`], use the [`create_rc_signal`] function.
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// let mut outer = None;
///
/// create_scope_immediate(|cx| {
/// // Even though the RcSignal is created inside a reactive scope, it can escape out of it.
/// let rc_state = create_rc_signal(0);
/// let rc_state_cloned = rc_state.clone();
/// let double = create_memo(cx, move || *rc_state_cloned.get() * 2);
/// assert_eq!(*double.get(), 0);
///
/// rc_state.set(1);
/// assert_eq!(*double.get(), 2);
///
/// // This isn't possible with simply create_signal(_)
/// outer = Some(rc_state);
/// });
/// ```
pub struct RcSignal<T>(Rc<Signal<T>>);

impl<T> Deref for RcSignal<T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> Clone for RcSignal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Create a new [`RcSignal`] with the specified initial value.
///
/// For more details, check the documentation for [`RcSignal`].
pub fn create_rc_signal<T>(value: T) -> RcSignal<T> {
    RcSignal(Rc::new(Signal::new(value)))
}

/// Create a new [`RcSignal`] with the specified initial value wrapped in a [`Rc`].
///
/// For more details, check the documentation for [`RcSignal`].
pub fn create_rc_signal_from_rc<T>(value: Rc<T>) -> RcSignal<T> {
    RcSignal(Rc::new(Signal(ReadSignal {
        value: RefCell::new(value),
        emitter: Default::default(),
    })))
}

/* Display implementations */

impl<T: Display> Display for RcSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
impl<T: Display> Display for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
impl<T: Display> Display for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

/* Debug implementations */

impl<T: Debug> Debug for RcSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RcSignal").field(&self.get()).finish()
    }
}
impl<T: Debug> Debug for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Signal").field(&self.get()).finish()
    }
}
impl<T: Debug> Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ReadSignal").field(&self.get()).finish()
    }
}

/* Default implementations */

impl<T: Default> Default for RcSignal<T> {
    fn default() -> Self {
        create_rc_signal(T::default())
    }
}

/* PartialEq, Eq, Hash implementations */

impl<T: PartialEq> PartialEq for RcSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}
impl<T: PartialEq> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}
impl<T: PartialEq> PartialEq for ReadSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}

impl<T: Eq> Eq for RcSignal<T> {}
impl<T: Eq> Eq for Signal<T> {}
impl<T: Eq> Eq for ReadSignal<T> {}

impl<T: Hash> Hash for RcSignal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state)
    }
}
impl<T: Hash> Hash for Signal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state)
    }
}
impl<T: Hash> Hash for ReadSignal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state)
    }
}

/* Serde implementations */

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for RcSignal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get().serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for RcSignal<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(create_rc_signal(T::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 0);
            assert_eq!(*state.get(), 0);

            state.set(1);
            assert_eq!(*state.get(), 1);

            state.set_fn(|n| n + 1);
            assert_eq!(*state.get(), 2);

            let state = create_signal(cx, vec![]);
            state.set_fn_mut(|v| v.push("yo"));
            assert_eq!(*state.get(), vec!["yo"]);
        });
    }

    #[test]
    fn signal_composition() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 0);
            let double = || *state.get() * 2;

            assert_eq!(double(), 0);
            state.set(1);
            assert_eq!(double(), 2);
        });
    }

    #[test]
    fn set_silent_signal() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 0);
            let double = state.map(cx, |&x| x * 2);
            assert_eq!(*double.get(), 0);

            state.set_silent(1);
            assert_eq!(*state.get(), 1);
            assert_eq!(*double.get(), 0); // double value is unchanged.

            state.set_fn_silent(|n| n + 1);
            assert_eq!(*state.get(), 2);
            assert_eq!(*double.get(), 0); // double value is unchanged.

            state.set_fn_mut_silent(|n| *n = 5);
            assert_eq!(*state.get(), 5);
            assert_eq!(*double.get(), 0); // double value is unchanged.
        });
    }

    #[test]
    fn read_signal() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 0);
            let readonly: &ReadSignal<i32> = state.deref();

            assert_eq!(*readonly.get(), 0);
            state.set(1);
            assert_eq!(*readonly.get(), 1);
        });
    }

    #[test]
    fn map_signal() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 0);
            let double = state.map(cx, |&x| x * 2);

            assert_eq!(*double.get(), 0);
            state.set(1);
            assert_eq!(*double.get(), 2);
        });
    }

    #[test]
    fn take_signal() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 123);

            let x = state.take();
            assert_eq!(*x, 123);
            assert_eq!(*state.get(), 0);
        });
    }

    #[test]
    fn take_silent_signal() {
        create_scope_immediate(|cx| {
            let state = create_signal(cx, 123);
            let double = state.map(cx, |&x| x * 2);

            // Do not trigger subscribers.
            state.take_silent();
            assert_eq!(*state.get(), 0);
            assert_eq!(*double.get(), 246);
        });
    }

    #[test]
    fn signal_split() {
        create_scope_immediate(|cx| {
            let (state, set_state) = create_signal(cx, 0).split();
            assert_eq!(*state(), 0);

            set_state(1);
            assert_eq!(*state(), 1);
        });
    }

    #[test]
    fn rc_signal() {
        let mut outer = None;
        create_scope_immediate(|cx| {
            let rc_state = create_rc_signal(0);
            let rc_state_cloned = rc_state.clone();
            let double = create_memo(cx, move || *rc_state_cloned.get() * 2);
            assert_eq!(*double.get(), 0);

            rc_state.set(1);
            assert_eq!(*double.get(), 2);

            outer = Some(rc_state);
        });
        assert_eq!(*outer.unwrap().get(), 1);
    }

    #[test]
    fn signal_display() {
        create_scope_immediate(|cx| {
            let signal = create_signal(cx, 0);
            assert_eq!(format!("{signal}"), "0");
            let read_signal: &ReadSignal<_> = signal;
            assert_eq!(format!("{read_signal}"), "0");
            let rcsignal = create_rc_signal(0);
            assert_eq!(format!("{rcsignal}"), "0");
        });
    }

    #[test]
    fn signal_debug() {
        create_scope_immediate(|cx| {
            let signal = create_signal(cx, 0);
            assert_eq!(format!("{signal:?}"), "Signal(0)");
            let read_signal: &ReadSignal<_> = signal;
            assert_eq!(format!("{read_signal:?}"), "ReadSignal(0)");
            let rcsignal = create_rc_signal(0);
            assert_eq!(format!("{rcsignal:?}"), "RcSignal(0)");
        });
    }

    #[test]
    fn signal_add_assign_update() {
        create_scope_immediate(|cx| {
            let mut signal = create_signal(cx, 0);
            let counter = create_signal(cx, 0);
            create_effect(cx, || {
                signal.track();
                counter.set(*counter.get_untracked() + 1);
            });
            signal += 1;
            signal -= 1;
            signal *= 1;
            signal /= 1;
            assert_eq!(*counter.get(), 5);
        });
    }

    #[test]
    fn signal_modify() {
        create_scope_immediate(|cx| {
            let signal = create_signal(cx, "Hello ".to_string());
            let counter = create_signal(cx, 0);
            create_effect(cx, || {
                signal.track();
                counter.set(*counter.get_untracked() + 1);
            });
            signal.modify().push_str("World!");
            assert_eq!(*signal.get(), "Hello World!");
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn create_signals_from_rc_value() {
        create_scope_immediate(|cx| {
            let _signal: &Signal<i32> = create_signal_from_rc(cx, Rc::new(0));
            let _rc_signal: RcSignal<i32> = create_rc_signal_from_rc(Rc::new(0));
        });
    }
}
