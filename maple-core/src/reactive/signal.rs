use super::*;

/// Returned by functions that provide a handle to access state.
pub struct StateHandle<T: 'static>(Rc<RefCell<SignalInner<T>>>);

impl<T: 'static> StateHandle<T> {
    /// Get the current value of the state.
    pub fn get(&self) -> Rc<T> {
        // if inside an effect, add this signal to dependency list
        CONTEXTS.with(|contexts| {
            if !contexts.borrow().is_empty() {
                let signal = Rc::downgrade(&self.0.clone());

                contexts
                    .borrow()
                    .last()
                    .unwrap()
                    .upgrade()
                    .expect("Running should be valid while inside reactive scope")
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
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
    /// use maple_core::prelude::*;
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
    pub fn get_untracked(&self) -> Rc<T> {
        self.0.borrow().inner.clone()
    }
}

impl<T: 'static> Clone for StateHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// State that can be set.
pub struct Signal<T: 'static> {
    handle: StateHandle<T>,
}

impl<T: 'static> Signal<T> {
    /// Creates a new signal.
    ///
    /// # Examples
    ///
    /// ```
    /// use maple_core::prelude::*;
    ///
    /// let state = Signal::new(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            handle: StateHandle(Rc::new(RefCell::new(SignalInner::new(value)))),
        }
    }

    /// Set the current value of the state.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    pub fn set(&self, new_value: T) {
        self.handle.0.borrow_mut().update(new_value);

        // Clone subscribers to prevent modifying list when calling callbacks.
        let subscribers = self.handle.0.borrow().subscribers.clone();

        for subscriber in subscribers {
            subscriber.0();
        }
    }

    /// Get the [`StateHandle`] associated with this signal.
    ///
    /// This is a shortcut for `(*signal).clone()`.
    pub fn handle(&self) -> StateHandle<T> {
        self.handle.clone()
    }

    /// Convert this signal into its underlying handle.
    pub fn into_handle(self) -> StateHandle<T> {
        self.handle
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

pub(super) struct SignalInner<T> {
    inner: Rc<T>,
    subscribers: HashSet<Callback>,
}

impl<T> SignalInner<T> {
    fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
            subscribers: HashSet::new(),
        }
    }

    /// Adds a handler to the subscriber list. If the handler is already a subscriber, does nothing.
    fn subscribe(&mut self, handler: Callback) {
        self.subscribers.insert(handler);
    }

    /// Removes a handler from the subscriber list. If the handler is not a subscriber, does nothing.
    fn unsubscribe(&mut self, handler: &Callback) {
        self.subscribers.remove(handler);
    }

    /// Updates the inner value. This does **NOT** call the subscribers.
    /// You will have to do so manually with `trigger_subscribers`.
    fn update(&mut self, new_value: T) {
        self.inner = Rc::new(new_value);
    }
}

/// Trait for any [`SignalInner`], regardless of type param `T`.
pub(super) trait AnySignalInner {
    fn subscribe(&self, handler: Callback);
    fn unsubscribe(&self, handler: &Callback);
}

impl<T> AnySignalInner for RefCell<SignalInner<T>> {
    fn subscribe(&self, handler: Callback) {
        self.borrow_mut().subscribe(handler);
    }

    fn unsubscribe(&self, handler: &Callback) {
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
}
