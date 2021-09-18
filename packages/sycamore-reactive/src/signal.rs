use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;

use super::*;

/// A readonly [`Signal`].
///
/// Returned by functions that provide a handle to access state.
/// Use [`Signal::handle`] or [`Signal::into_handle`] to retrieve a handle from a [`Signal`].
pub struct StateHandle<T: 'static>(Rc<RefCell<SignalInner<T>>>);

impl<T: 'static> StateHandle<T> {
    /// Get the current value of the state. When called inside a reactive scope, calling this will
    /// add itself to the scope's dependencies.
    ///
    /// # Example
    /// ```rust
    /// use sycamore_reactive::*;
    ///
    /// let state = Signal::new(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// ```
    pub fn get(&self) -> Rc<T> {
        // If inside an effect, add this signal to dependency list.
        // If running inside a destructor, do nothing.
        let _ = LISTENERS.try_with(|listeners| {
            if let Some(last_context) = listeners.borrow().last() {
                let signal = Rc::clone(&self.0);

                last_context
                    .upgrade()
                    .expect_throw("Running should be valid while inside reactive scope")
                    .borrow_mut()
                    .as_mut()
                    .unwrap_throw()
                    .dependencies
                    .insert(Dependency(signal));
            }
        });

        self.get_untracked()
    }

    /// Get the current value of the state, without tracking this as a dependency if inside a
    /// reactive context.
    ///
    /// # Example
    ///
    /// ```
    /// use sycamore_reactive::*;
    ///
    /// let state = Signal::new(1);
    ///
    /// let double = create_memo({
    ///     let state = state.clone();
    ///     move || *state.get_untracked() * 2
    /// });
    ///
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// // double value should still be old value because state was untracked
    /// assert_eq!(*double.get(), 2);
    /// ```
    #[inline]
    pub fn get_untracked(&self) -> Rc<T> {
        Rc::clone(&self.0.borrow().inner)
    }
}

impl<T: 'static> Clone for StateHandle<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T: fmt::Debug> fmt::Debug for StateHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StateHandle")
            .field(&self.get_untracked())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for StateHandle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_untracked().as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for StateHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Signal::new(T::deserialize(deserializer)?).handle())
    }
}

/// State that can be set.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let state = Signal::new(0);
/// assert_eq!(*state.get(), 0);
///
/// state.set(1);
/// assert_eq!(*state.get(), 1);
/// ```
pub struct Signal<T: 'static> {
    handle: StateHandle<T>,
}

impl<T: 'static> Signal<T> {
    /// Creates a new signal with the given value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// let state = Signal::new(0);
    /// # assert_eq!(*state.get(), 0);
    /// ```
    #[inline]
    pub fn new(initial: T) -> Self {
        Self {
            handle: StateHandle(Rc::new(RefCell::new(SignalInner::new(initial)))),
        }
    }

    /// Set the current value of the state.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    ///
    /// let state = Signal::new(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// ```
    pub fn set(&self, new_value: T) {
        self.handle.0.borrow_mut().update(new_value);

        self.trigger_subscribers();
    }

    /// Get the [`StateHandle`] associated with this signal.
    ///
    /// This is a shortcut for `(*signal).clone()`.
    #[inline]
    pub fn handle(&self) -> StateHandle<T> {
        self.handle.clone()
    }

    /// Consumes this signal and returns its underlying [`StateHandle`].
    #[inline]
    pub fn into_handle(self) -> StateHandle<T> {
        self.handle
    }

    /// Calls all the subscribers without modifying the state.
    /// This can be useful when using patterns such as inner mutability where the state updated will
    /// not be automatically triggered. In the general case, however, it is preferable to use
    /// [`Signal::set`] instead.
    pub fn trigger_subscribers(&self) {
        // Clone subscribers to prevent modifying list when calling callbacks.
        let subscribers = self.handle.0.borrow().subscribers.clone();

        // Reverse order of subscribers to trigger outer effects before inner effects.
        for subscriber in subscribers.values().rev() {
            // subscriber might have already been destroyed in the case of nested effects
            if let Some(callback) = subscriber.try_callback() {
                // Might already be inside a callback, if infinite loop.
                // Do nothing if infinite loop.
                if let Ok(mut callback) = callback.try_borrow_mut() {
                    callback()
                }
            }
        }
    }
}

impl<T: 'static> Deref for Signal<T> {
    type Target = StateHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for Signal<T> {
    fn eq(&self, other: &Signal<T>) -> bool {
        self.get_untracked().eq(&other.get_untracked())
    }
}

impl<T: Eq> Eq for Signal<T> {}

impl<T: Hash> Hash for Signal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_untracked().hash(state);
    }
}

impl<T: fmt::Debug> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Signal")
            .field(&self.get_untracked())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Signal<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_untracked().as_ref().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Signal<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Signal::new(T::deserialize(deserializer)?))
    }
}

pub(super) struct SignalInner<T> {
    inner: Rc<T>,
    subscribers: IndexMap<CallbackPtr, Callback>,
}

impl<T> SignalInner<T> {
    fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
            subscribers: IndexMap::new(),
        }
    }

    /// Adds a handler to the subscriber list. If the handler is already a subscriber, does nothing.
    fn subscribe(&mut self, handler: Callback) {
        self.subscribers.insert(handler.as_ptr(), handler);
    }

    /// Removes a handler from the subscriber list. If the handler is not a subscriber, does
    /// nothing.
    fn unsubscribe(&mut self, handler: CallbackPtr) {
        self.subscribers.remove(&handler);
    }

    /// Updates the inner value. This does **NOT** call the subscribers.
    /// You will have to do so manually with `trigger_subscribers`.
    fn update(&mut self, new_value: T) {
        self.inner = Rc::new(new_value);
    }
}

/// Trait for any [`SignalInner`], regardless of type param `T`.
pub(super) trait AnySignalInner {
    /// Wrapper around [`SignalInner::subscribe`].
    fn subscribe(&self, handler: Callback);
    /// Wrapper around [`SignalInner::unsubscribe`].
    fn unsubscribe(&self, handler: CallbackPtr);
}

impl<T> AnySignalInner for RefCell<SignalInner<T>> {
    fn subscribe(&self, handler: Callback) {
        self.borrow_mut().subscribe(handler);
    }

    fn unsubscribe(&self, handler: CallbackPtr) {
        self.borrow_mut().unsubscribe(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals() {
        let state = Signal::new(0);
        assert_eq!(*state.get(), 0);

        state.set(1);
        assert_eq!(*state.get(), 1);
    }

    #[test]
    fn signal_composition() {
        let state = Signal::new(0);

        let double = || *state.get() * 2;

        assert_eq!(double(), 0);

        state.set(1);
        assert_eq!(double(), 2);
    }

    #[test]
    fn state_handle() {
        let state = Signal::new(0);
        let readonly = state.handle();

        assert_eq!(*readonly.get(), 0);

        state.set(1);
        assert_eq!(*readonly.get(), 1);
    }
}
