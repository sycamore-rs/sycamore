//! Signals - The building blocks of reactivity.

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use crate::effect::EFFECTS;
use crate::*;

type WeakEffectCallback = Weak<RefCell<dyn FnMut()>>;
type EffectCallbackPtr = *const RefCell<dyn FnMut()>;

/// A struct for managing subscriptions to signals.
#[derive(Default)]
pub struct SignalEmitter(RefCell<IndexMap<EffectCallbackPtr, WeakEffectCallback>>);

impl SignalEmitter {
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
                // SAFETY: `last` necessarily lasts longer than self.
                last.add_dependency(unsafe { std::mem::transmute(self) });
            }
        });
    }

    /// Calls all the subscribers without modifying the state.
    /// This can be useful when using patterns such as inner mutability where the state updated will
    /// not be automatically triggered. In the general case, however, it is preferable to use
    /// [`Signal::set()`] instead.
    pub fn trigger_subscribers(&self) {
        // Clone subscribers to prevent modifying list when calling callbacks.
        let subscribers = self.0.borrow().clone();
        // Subscriber order is reversed because effects attach subscribers at the end of the
        // effect scope. This will ensure that outer effects re-execute before inner effects,
        // preventing inner effects from running twice.
        for subscriber in subscribers.values().rev() {
            // subscriber might have already been destroyed in the case of nested effects
            if let Some(callback) = subscriber.upgrade() {
                // Call the callback.
                callback.borrow_mut()();
            }
        }
    }
}

/// A read-only [`Signal`].
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
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
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
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(1);
    /// let double = ctx.create_memo(|| *state.get_untracked() * 2);
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
    /// [`create_memo`](Scope::create_memo).
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(1);
    /// let double = state.map(&ctx, |&x| x * 2);
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// assert_eq!(*double.get(), 4);
    /// # });
    /// ```
    #[must_use]
    pub fn map<'a, U>(
        &'a self,
        ctx: ScopeRef<'a>,
        mut f: impl FnMut(&T) -> U + 'a,
    ) -> &'a ReadSignal<U> {
        ctx.create_memo(move || f(&self.get()))
    }

    /// When called inside a reactive scope, calling this will add itself to the scope's
    /// dependencies.
    ///
    /// To both track and get the value of the signal, use [`ReadSignal::get`] instead.
    pub fn track(&self) {
        self.emitter.track();
    }
}

/// Reactive state that can be updated and subscribed to.
pub struct Signal<T>(ReadSignal<T>);

impl<T> Signal<T> {
    /// Create a new [`Signal`] with the specified value.
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
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    pub fn set(&self, value: T) {
        *self.0.value.borrow_mut() = Rc::new(value);
        self.0.emitter.trigger_subscribers();
    }

    /// Set the current value of the state _without_ triggering subscribers.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn set_silent(&self, value: T) {
        *self.0.value.borrow_mut() = Rc::new(value);
    }

    /// Split a signal into getter and setter handles.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let (state, set_state) = ctx.create_signal(0).split();
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
}

impl<T: Default> Signal<T> {
    /// Take the current value out and replace it with the default value.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    pub fn take(&self) -> Rc<T> {
        let ret = self.0.value.take();
        self.0.emitter.trigger_subscribers();
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

impl<'a, T> Deref for Signal<T> {
    type Target = ReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

/// A signal that is not bound to a [`Scope`].
///
/// Sometimes, it is useful to have a signal that can escape the enclosing [reactive scope](Scope).
/// However, this cannot be achieved simply with [`Scope::create_signal`] because the resulting
/// [`Signal`] is tied to the [`Scope`] by it's lifetime. The [`Signal`] can only live as long as
/// the [`Scope`].
///
/// With [`RcSignal`] on the other hand, the lifetime is not tied to a [`Scope`]. Memory is managed
/// using a reference-counted smart pointer ([`Rc`]). What this means is that [`RcSignal`] cannot
/// implement the [`Copy`] trait and therefore needs to be manually cloned into all closures where
/// it is used.
///
/// In general, [`Scope::create_signal`] should be preferred, both for performance and ergonomics.
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
/// create_scope_immediate(|ctx| {
/// // Even though the RcSignal is created inside a reactive scope, it can escape out of it.
/// let rc_state = create_rc_signal(0);
/// let rc_state_cloned = rc_state.clone();
/// let double = ctx.create_memo(move || *rc_state_cloned.get() * 2);
/// assert_eq!(*double.get(), 0);
///
/// rc_state.set(1);
/// assert_eq!(*double.get(), 2);
///
/// // This isn't possible with simply ctx.create_signal()
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
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            assert_eq!(*state.get(), 0);

            state.set(1);
            assert_eq!(*state.get(), 1);
        });
    }

    #[test]
    fn signal_composition() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = || *state.get() * 2;

            assert_eq!(double(), 0);
            state.set(1);
            assert_eq!(double(), 2);
        });
    }

    #[test]
    fn set_silent_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = state.map(ctx, |&x| x * 2);

            assert_eq!(*double.get(), 0);
            state.set_silent(1);
            assert_eq!(*double.get(), 0); // double value is unchanged.
        });
    }

    #[test]
    fn read_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let readonly: &ReadSignal<i32> = state.deref();

            assert_eq!(*readonly.get(), 0);
            state.set(1);
            assert_eq!(*readonly.get(), 1);
        });
    }

    #[test]
    fn map_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = state.map(ctx, |&x| x * 2);

            assert_eq!(*double.get(), 0);
            state.set(1);
            assert_eq!(*double.get(), 2);
        });
    }

    #[test]
    fn take_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(123);

            let x = state.take();
            assert_eq!(*x, 123);
            assert_eq!(*state.get(), 0);
        });
    }

    #[test]
    fn take_silent_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(123);
            let double = state.map(ctx, |&x| x * 2);

            // Do not trigger subscribers.
            state.take_silent();
            assert_eq!(*state.get(), 0);
            assert_eq!(*double.get(), 246);
        });
    }

    #[test]
    fn rc_signal() {
        let mut outer = None;
        create_scope_immediate(|ctx| {
            let rc_state = create_rc_signal(0);
            let rc_state_cloned = rc_state.clone();
            let double = ctx.create_memo(move || *rc_state_cloned.get() * 2);
            assert_eq!(*double.get(), 0);

            rc_state.set(1);
            assert_eq!(*double.get(), 2);

            outer = Some(rc_state);
        });
        assert_eq!(*outer.unwrap().get(), 1);
    }
}
