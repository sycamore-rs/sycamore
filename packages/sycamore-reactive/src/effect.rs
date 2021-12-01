use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::panic::Location;
use std::rc::{Rc, Weak};
use std::{mem, ptr};

use ahash::AHashSet;
use smallvec::SmallVec;
use wasm_bindgen::prelude::*;

use super::*;

/// The number of effects that are allocated on the stack before resorting to heap allocation in
/// [`ReactiveScope`].
const REACTIVE_SCOPE_EFFECTS_STACK_CAPACITY: usize = 2;

/// Initial capacity for [`CONTEXTS`].
const CONTEXTS_INITIAL_CAPACITY: usize = 10;
/// Initial capacity for [`SCOPES`].
const SCOPES_INITIAL_CAPACITY: usize = 4;

thread_local! {
    /// Listeners for the effect that is currently running. `None` if no effect is running.
    ///
    /// The [`Listener`] contains a list of [`Signal`]s that were accessed within the scope.
    pub(super) static LISTENERS: RefCell<Vec<Weak<RefCell<Option<Listener>>>>> =
        RefCell::new(Vec::with_capacity(CONTEXTS_INITIAL_CAPACITY));
    /// Explicit stack of [`ReactiveScope`]s.
    pub(super) static SCOPES: RefCell<Vec<ReactiveScope>> =
        RefCell::new(Vec::with_capacity(SCOPES_INITIAL_CAPACITY));
}

/// State of the current running effect.
/// When the state is dropped, all dependencies are removed (both links and backlinks).
///
/// The difference between [`Listener`] and [`ReactiveScope`] is that [`Listener`] is used for
/// dependency tracking whereas [`ReactiveScope`] is used for resource cleanup. Each [`Listener`]
/// contains a [`ReactiveScope`].
pub(super) struct Listener {
    /// Callback to run when the effect is recreated.
    pub(super) callback: Rc<RefCell<dyn FnMut()>>,
    /// A list of dependencies which trigger the effect.
    pub(super) dependencies: AHashSet<Dependency>,
    /// The reactive scope owns all effects created within it.
    scope: ReactiveScope,
}

impl Listener {
    /// Clears the dependencies (both links and backlinks).
    /// Should be called when re-executing an effect to recreate all dependencies.
    fn clear_dependencies(&mut self) {
        for dependency in &self.dependencies {
            dependency.signal().unsubscribe(Rc::as_ptr(&self.callback));
        }
        self.dependencies.clear();
    }
}

/// Internal representation for [`ReactiveScope`].
pub(crate) struct ReactiveScopeInner {
    /// Effects created in this scope.
    effects: SmallVec<[Rc<RefCell<Option<Listener>>>; REACTIVE_SCOPE_EFFECTS_STACK_CAPACITY]>,
    /// Callbacks to call when the scope is dropped.
    cleanup: Vec<Box<dyn FnOnce()>>,
    /// Contexts created in this scope.
    pub context: Option<Box<dyn ContextAny>>,
    pub parent: ReactiveScopeWeak,
    /// The source location where this scope was created.
    /// Only available when in debug mode.
    #[cfg(debug_assertions)]
    pub loc: &'static Location<'static>,
}

impl ReactiveScopeInner {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new() -> Self {
        Self {
            effects: SmallVec::new(),
            cleanup: Vec::new(),
            context: None,
            parent: ReactiveScopeWeak::default(),
            #[cfg(debug_assertions)]
            loc: Location::caller(),
        }
    }
}

impl Default for ReactiveScopeInner {
    #[cfg_attr(debug_assertions, track_caller)]
    fn default() -> Self {
        Self::new()
    }
}

/// Owns the effects created in the current reactive scope.
/// The effects are dropped and the cleanup callbacks are called when the [`ReactiveScope`] is
/// dropped.
///
/// A new [`ReactiveScope`] is usually created with [`create_root`]. A new [`ReactiveScope`] is also
/// created when a new effect is created with [`create_effect`] and other reactive utilities that
/// call it under the hood.
pub struct ReactiveScope(pub(crate) Rc<RefCell<ReactiveScopeInner>>);

impl ReactiveScope {
    /// Create a new empty [`ReactiveScope`].
    ///
    /// This should be rarely used and only serve as a placeholder. The scope created by this method
    /// is detached from the scope hierarchy, meaning that functionality such as contexts would not
    /// work through this scope.
    ///
    /// In general, prefer [`create_scope`] instead.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new() -> Self {
        // We call this first to make sure that track_caller can do its thing.
        let inner = ReactiveScopeInner::new();
        Self(Rc::new(RefCell::new(inner)))
    }

    /// Add an effect that is owned by this [`ReactiveScope`].
    pub(super) fn add_effect_state(&mut self, effect: Rc<RefCell<Option<Listener>>>) {
        self.0.borrow_mut().effects.push(effect);
    }

    /// Add a cleanup callback that will be called when the [`ReactiveScope`] is dropped.
    pub(super) fn add_cleanup(&mut self, cleanup: Box<dyn FnOnce()>) {
        self.0.borrow_mut().cleanup.push(cleanup);
    }

    /// Create a new [`ReactiveScopeWeak`] from this [`ReactiveScope`].
    pub(crate) fn downgrade(&self) -> ReactiveScopeWeak {
        ReactiveScopeWeak(Rc::downgrade(&self.0))
    }

    /// Runs the passed callback in the reactive scope pointed to by this handle.
    pub fn extend<U>(&self, f: impl FnOnce() -> U) -> U {
        SCOPES.with(|scopes| {
            scopes.borrow_mut().push(ReactiveScope(self.0.clone())); // We now have 2 references to the scope.
            let u = f();
            scopes.borrow_mut().pop().unwrap(); // Rationale: pop the scope we pushed above.
                                                // Since we have 2 references to the scope, this will not drop the scope.
            u
        })
    }

    /// Runs the passed future in the reactive scope pointed to by this handle.
    pub async fn extend_future<U>(&self, f: impl Future<Output = U>) -> U {
        SCOPES.with(|scopes| {
            scopes.borrow_mut().push(ReactiveScope(self.0.clone())); // We now have 2 references to
                                                                     // the scope.
        });
        let u = f.await;
        SCOPES.with(|scopes| {
            scopes.borrow_mut().pop().unwrap(); // Rationale: pop the scope we pushed above.
                                                // Since we have 2 references to the scope, this
                                                // will not drop the scope.
        });
        u
    }

    /// Returns the source code [`Location`] where this [`ReactiveScope`] was created.
    pub fn creation_loc(&self) -> Option<&'static Location<'static>> {
        #[cfg(debug_assertions)]
        return Some(self.0.borrow().loc);
        #[cfg(not(debug_assertions))]
        return None;
    }
}

impl Default for ReactiveScope {
    #[cfg_attr(debug_assertions, track_caller)]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ReactiveScope {
    fn drop(&mut self) {
        if Rc::strong_count(&self.0) == 1 {
            // This is the last reference to the scope. Drop all effects and call the cleanup
            // callbacks.
            for effect in &self.0.borrow().effects {
                effect
                    .borrow_mut()
                    .as_mut()
                    .unwrap_throw()
                    .clear_dependencies();
            }

            for cleanup in mem::take(&mut self.0.borrow_mut().cleanup) {
                untrack(cleanup);
            }
        }
    }
}

/// A weak reference to a [`ReactiveScope`]. This can be created by calling
/// [`ReactiveScope::downgrade`].
///
/// It is also possible to have a [`ReactiveScopeWeak`] that points to nowhere. This can be created
/// by using [`Default::default`].
#[derive(Default, Clone)]
pub struct ReactiveScopeWeak(pub(crate) Weak<RefCell<ReactiveScopeInner>>);

impl ReactiveScopeWeak {
    /// Runs the passed callback in the reactive scope pointed to by this handle.
    ///
    /// If the scope has already been destroyed, the callback is not run and `None` is returned.
    pub fn extend<U>(&self, f: impl FnOnce() -> U) -> Option<U> {
        // We only upgrade this temporarily for the duration of this
        // function call.
        if let Some(this) = self.0.upgrade() {
            SCOPES.with(|scopes| {
                scopes.borrow_mut().push(ReactiveScope(this)); // We now have 2 references to the scope.
                let u = f();
                scopes.borrow_mut().pop().unwrap(); // Rationale: pop the scope we pushed above.
                                                    // Since we have 2 references to the scope, this will not drop the scope.
                Some(u)
            })
        } else {
            None
        }
    }

    /// Runs the passed future in the reactive scope pointed to by this handle.
    ///
    /// If the scope has already been destroyed, the callback is not run and `None` is returned.
    pub async fn extend_future<U>(&self, f: impl Future<Output = U>) -> Option<U> {
        // We only upgrade this temporarily for the duration of this
        // function call.
        if let Some(this) = self.0.upgrade() {
            SCOPES.with(|scopes| {
                scopes.borrow_mut().push(ReactiveScope(this)); // We now have 2 references to the
                                                               // scope.
            });
            let u = f.await;
            SCOPES.with(|scopes| {
                scopes.borrow_mut().pop().unwrap(); // Rationale: pop the scope we pushed above.
                                                    // Since we have 2 references to the scope, this will not drop the scope.
                Some(u)
            })
        } else {
            None
        }
    }

    /// Returns `true` if the [`ReactiveScope`] pointed to by the weak reference is still valid.
    /// Returns `false` otherwise.
    pub fn is_valid(&self) -> bool {
        self.0.strong_count() != 0
    }
}

pub(super) type CallbackPtr = *const RefCell<dyn FnMut()>;

#[derive(Clone)]
pub(super) struct Callback(pub(super) Weak<RefCell<dyn FnMut()>>);

impl Callback {
    #[must_use = "returned value must be manually called"]
    pub fn try_callback(&self) -> Option<Rc<RefCell<dyn FnMut()>>> {
        self.0.upgrade()
    }

    pub fn as_ptr(&self) -> CallbackPtr {
        Weak::as_ptr(&self.0)
    }
}

/// A strong backlink to a [`Signal`] for any type `T`.
#[derive(Clone)]
pub(super) struct Dependency(pub(super) Rc<dyn AnySignalInner>);

impl Dependency {
    fn signal(&self) -> Rc<dyn AnySignalInner> {
        Rc::clone(&self.0)
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq::<()>(Rc::as_ptr(&self.0).cast(), Rc::as_ptr(&other.0).cast())
    }
}
impl Eq for Dependency {}

/// Creates an effect on signals used inside the effect closure.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let state = Signal::new(0);
///
/// create_effect(cloned!((state) => move || {
///     println!("State changed. New state value = {}", state.get());
/// })); // Prints "State changed. New state value = 0"
///
/// state.set(1); // Prints "State changed. New state value = 1"
/// ```
#[inline]
pub fn create_effect<F>(effect: F)
where
    F: FnMut() + 'static,
{
    _create_effect(Box::new(effect));
}

/// Internal implementation: use dynamic dispatch to reduce code bloat.
fn _create_effect(mut effect: Box<dyn FnMut()>) {
    let listener: Rc<RefCell<Option<Listener>>> = Rc::new(RefCell::new(None));

    // Callback for when the effect's dependencies are triggered.
    let callback: Rc<RefCell<dyn FnMut()>> = Rc::new(RefCell::new({
        let listener = Rc::downgrade(&listener);
        move || {
            LISTENERS.with(|listeners| {
                // Record initial context size to verify that it is the same after.
                let initial_context_size = listeners.borrow().len();

                // Upgrade running now to make sure running is valid for the whole duration of
                // the effect.
                let listener = listener.upgrade().unwrap_throw();

                // Push new reactive scope.
                listeners.borrow_mut().push(Rc::downgrade(&listener));

                let mut listener_mut = listener.borrow_mut();
                let listener_ref = listener_mut.as_mut().unwrap_throw();

                let old_dependencies = mem::take(&mut listener_ref.dependencies);

                // Get old scope's parent so that new scope does not change scope hierarchy.
                let parent = listener_ref.scope.0.borrow().parent.clone();

                // We want to destroy the old scope before creating the new one, so that cleanup
                // functions will be run before the effect closure is called again.
                let _ = mem::take(&mut listener_ref.scope);

                drop(listener_mut); // Drop the RefMut because Signals will access it inside the effect callback.
                let new_scope = create_child_scope_in(&parent, || {
                    // Run effect closure.
                    effect();
                });
                let mut listener_mut = listener.borrow_mut();
                let listener_ref = listener_mut.as_mut().unwrap_throw();
                listener_ref.scope = new_scope;

                // Unsubscribe from removed dependencies.
                // Removed dependencies are those that are in old dependencies but not in new
                // dependencies.
                for old_dependency in old_dependencies.difference(&listener_ref.dependencies) {
                    old_dependency
                        .signal()
                        .unsubscribe(listener_ref.callback.as_ref());
                }

                // Subscribe to new dependencies.
                // New dependencies are those that are in new dependencies but not in old
                // dependencies.
                for new_dependency in listener_ref.dependencies.difference(&old_dependencies) {
                    new_dependency.signal().subscribe(Callback(Rc::downgrade(
                        // Reference the same closure we are in right now.
                        // When the dependency changes, this closure will be called again.
                        &listener_ref.callback,
                    )));
                }

                // Remove reactive context.
                listeners.borrow_mut().pop();

                debug_assert_eq!(
                    initial_context_size,
                    listeners.borrow().len(),
                    "context size should not change before and after create_effect_initial"
                );
            });
        }
    }));

    *listener.borrow_mut() = Some(Listener {
        callback: Rc::clone(&callback),
        dependencies: AHashSet::new(),
        scope: ReactiveScope::new(), /* This is a placeholder and will be replaced when callback
                                      * is called. */
    });
    debug_assert_eq!(
        Rc::strong_count(&listener),
        1,
        "Running should be owned exclusively by ReactiveScope"
    );

    SCOPES.with(|scope| {
        if scope.borrow().last().is_some() {
            scope
                .borrow_mut()
                .last_mut()
                .unwrap_throw()
                .add_effect_state(listener);
        } else {
            thread_local! {
                static GLOBAL_SCOPE: RefCell<ReactiveScope> = RefCell::new(ReactiveScope::new());
            }
            GLOBAL_SCOPE.with(|global_scope| global_scope.borrow_mut().add_effect_state(listener));
        }
    });

    callback.borrow_mut()();
}

/// Creates a memoized value from some signals. Also know as "derived stores".
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let state = Signal::new(0);
///
/// let double = create_memo(cloned!((state) => move || *state.get() * 2));
/// assert_eq!(*double.get(), 0);
///
/// state.set(1);
/// assert_eq!(*double.get(), 2);
/// ```
#[inline]
pub fn create_memo<F, Out>(derived: F) -> ReadSignal<Out>
where
    F: FnMut() -> Out + 'static,
    Out: 'static,
{
    create_selector_with(derived, |_, _| false)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is
/// the same. That is why the output of the function must implement [`PartialEq`].
///
/// To specify a custom comparison function, use [`create_selector_with`].
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let state = Signal::new(0);
/// let double = create_selector(cloned!((state) => move || *state.get() * 2));
/// assert_eq!(*double.get(), 0);
///
/// state.set(1);
/// assert_eq!(*double.get(), 2);
/// ```
#[inline]
pub fn create_selector<F, Out>(derived: F) -> ReadSignal<Out>
where
    F: FnMut() -> Out + 'static,
    Out: PartialEq + 'static,
{
    create_selector_with(derived, PartialEq::eq)
}

/// Creates a memoized value from some signals. Also know as "derived stores".
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is
/// the same.
///
/// It takes a comparison function to compare the old and new value, which returns `true` if they
/// are the same and `false` otherwise.
///
/// To use the type's [`PartialEq`] implementation instead of a custom function, use
/// [`create_selector`].
pub fn create_selector_with<F, Out, C>(mut derived: F, comparator: C) -> ReadSignal<Out>
where
    F: FnMut() -> Out + 'static,
    Out: 'static,
    C: Fn(&Out, &Out) -> bool + 'static,
{
    let memo = Rc::new(RefCell::new(None::<Signal<Out>>));

    create_effect({
        let memo = Rc::clone(&memo);
        move || {
            if memo.borrow().as_ref().is_some() {
                let memo = memo.borrow();
                let memo = memo.as_ref().unwrap_throw();
                let new_value = derived();
                if !comparator(&memo.get_untracked(), &new_value) {
                    memo.set(new_value);
                }
            } else {
                *memo.borrow_mut() = Some(Signal::new(derived()));
            }
        }
    });

    let memo = memo.borrow();
    memo.as_ref().unwrap_throw().handle()
}

/// An alternative to [`Signal::new`] that uses a reducer to get the next value.
///
/// It uses a reducer function that takes the previous value and a message and returns the next
/// value.
///
/// Returns a [`ReadSignal`] and a dispatch function to send messages to the reducer.
///
/// # Params
/// * `initial` - The initial value of the state.
/// * `reducer` - A function that takes the previous value and a message and returns the next value.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// enum Msg {
///     Increment,
///     Decrement,
/// }
///
/// let (state, dispatch) = create_reducer(0, |state, msg: Msg| match msg {
///     Msg::Increment => *state + 1,
///     Msg::Decrement => *state - 1,
/// });
///
/// assert_eq!(*state.get(), 0);
/// dispatch(Msg::Increment);
/// assert_eq!(*state.get(), 1);
/// dispatch(Msg::Decrement);
/// assert_eq!(*state.get(), 0);
/// ```
pub fn create_reducer<F, Out, Msg>(initial: Out, reduce: F) -> (ReadSignal<Out>, Rc<impl Fn(Msg)>)
where
    F: Fn(&Out, Msg) -> Out,
{
    let memo = Signal::new(initial);

    let dispatcher = {
        let memo = memo.clone();
        move |msg| {
            memo.set(reduce(&memo.get_untracked(), msg));
        }
    };

    (memo.into_handle(), Rc::new(dispatcher))
}

/// Run the passed closure inside an untracked dependency scope.
///
/// This does **NOT** create a new [`ReactiveScope`].
///
/// See also [`StateHandle::get_untracked()`].
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
///     move || untrack(|| *state.get() * 2)
/// });
///
/// assert_eq!(*double.get(), 2);
///
/// state.set(2);
/// // double value should still be old value because state was untracked
/// assert_eq!(*double.get(), 2);
/// ```
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    let f = Rc::new(RefCell::new(Some(f)));
    let g = Rc::clone(&f);

    // Do not panic if running inside destructor.
    if let Ok(ret) = LISTENERS.try_with(|listeners| {
        let tmp = listeners.take();

        let ret = f.take().unwrap_throw()();

        *listeners.borrow_mut() = tmp;

        ret
    }) {
        ret
    } else {
        g.take().unwrap_throw()()
    }
}

/// Adds a callback function to the current reactive scope's cleanup.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// let cleanup_called = Signal::new(false);
///
/// let scope = create_root(cloned!((cleanup_called) => move || {
///     on_cleanup(move || {
///         cleanup_called.set(true);
///     })
/// }));
///
/// assert_eq!(*cleanup_called.get(), false);
///
/// drop(scope);
/// assert_eq!(*cleanup_called.get(), true);
/// ```
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    SCOPES.with(|scope| {
        if scope.borrow().last().is_some() {
            scope
                .borrow_mut()
                .last_mut()
                .unwrap_throw()
                .add_cleanup(Box::new(f));
        } else {
            #[cfg(all(target_arch = "wasm32", debug_assertions))]
            web_sys::console::warn_1(
                &"Cleanup callbacks created outside of a reactive root will never run.".into(),
            );
            #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
            eprintln!(
                "WARNING: Cleanup callbacks created outside of a reactive root will never run."
            );
        }
    });
}

/// Gets the number of dependencies of the current reactive scope.
///
/// If the function is called outside a reactive scope, it will return `None`.
///
/// # Example
/// ```
/// use sycamore_reactive::*;
///
/// assert_eq!(dependency_count(), None);
///
/// let state = Signal::new(1);
/// create_effect(move || {
///     assert_eq!(dependency_count(), Some(0));
///     state.get();
///     assert_eq!(dependency_count(), Some(1));
/// });
/// ```
pub fn dependency_count() -> Option<usize> {
    LISTENERS.with(|listeners| {
        listeners.borrow().last().map(|last_context| {
            last_context
                .upgrade()
                .expect_throw("Running should be valid while inside reactive scope")
                .borrow()
                .as_ref()
                .unwrap_throw()
                .dependencies
                .len()
        })
    })
}

/// Returns a [`ReactiveScopeWeak`] handle to the current reactive scope. If outside a scope,
/// returns a [`ReactiveScopeWeak`] that points to nothing.
pub fn current_scope() -> ReactiveScopeWeak {
    SCOPES.with(|scope| {
        scope
            .borrow()
            .last()
            .map(|last_context| last_context.downgrade())
            .unwrap_or_default()
    })
}

/// A struct that can be debug-printed to view the scope hierarchy at the location it was created.
pub struct DebugScopeHierarchy {
    scope: Option<Rc<RefCell<ReactiveScopeInner>>>,
    loc: &'static Location<'static>,
}

/// Returns a [`DebugScopeHierarchy`] which can be printed using [`std::fmt::Debug`] to debug the
/// scope hierarchy at the current level.
#[track_caller]
pub fn debug_scope_hierarchy() -> DebugScopeHierarchy {
    let loc = Location::caller();
    SCOPES.with(|scope| DebugScopeHierarchy {
        scope: scope.borrow().last().map(|x| x.0.clone()),
        loc,
    })
}

impl Debug for DebugScopeHierarchy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Reactive scope hierarchy at {}:", self.loc)?;
        if let Some(scope) = &self.scope {
            let mut s = Some(scope.clone());
            while let Some(x) = s {
                // Print scope.
                if let Some(loc) = ReactiveScope(x.clone()).creation_loc() {
                    write!(f, "\tScope created at {}", loc)?;
                } else {
                    write!(f, "\tScope")?;
                }
                // Print context.
                if let Some(context) = &x.borrow().context {
                    let type_name = context.get_type_name();
                    if let Some(type_name) = type_name {
                        write!(f, " with context (type = {})", type_name)?;
                    } else {
                        write!(f, " with context")?;
                    }
                }
                writeln!(f)?;
                // Set next iteration with scope parent.
                s = x.borrow().parent.0.upgrade();
            }
        } else {
            writeln!(f, "Not inside a reactive scope")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloned;

    #[test]
    fn effects() {
        let state = Signal::new(0);

        let double = Signal::new(-1);

        create_effect(cloned!((state, double) => move || {
            double.set(*state.get() * 2);
        }));
        assert_eq!(*double.get(), 0); // calling create_effect should call the effect at least once

        state.set(1);
        assert_eq!(*double.get(), 2);
        state.set(2);
        assert_eq!(*double.get(), 4);
    }

    #[test]
    fn effect_do_not_create_infinite_loop() {
        let state = Signal::new(0);
        create_effect(cloned!((state) => move || {
            state.get();
            state.set(0);
        }));
        state.set(0);
    }

    #[test]
    fn effect_should_subscribe_once() {
        let state = Signal::new(0);

        let counter = Signal::new(0);
        create_effect(cloned!((state, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            // call state.get() twice but should subscribe once
            state.get();
            state.get();
        }));

        assert_eq!(*counter.get(), 1);

        state.set(1);
        assert_eq!(*counter.get(), 2);
    }

    #[test]
    fn effect_should_recreate_dependencies() {
        let condition = Signal::new(true);

        let state1 = Signal::new(0);
        let state2 = Signal::new(1);

        let counter = Signal::new(0);
        create_effect(cloned!((condition, state1, state2, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            if *condition.get() {
                state1.get();
            } else {
                state2.get();
            }
        }));

        assert_eq!(*counter.get(), 1);

        state1.set(1);
        assert_eq!(*counter.get(), 2);

        state2.set(1);
        assert_eq!(*counter.get(), 2); // not tracked

        condition.set(false);
        assert_eq!(*counter.get(), 3);

        state1.set(2);
        assert_eq!(*counter.get(), 3); // not tracked

        state2.set(2);
        assert_eq!(*counter.get(), 4); // tracked after condition.set
    }

    #[test]
    fn nested_effects_should_recreate_inner() {
        let counter = Signal::new(0);

        let trigger = Signal::new(());

        create_effect(cloned!((trigger, counter) => move || {
            trigger.get(); // subscribe to trigger

            create_effect(cloned!((counter) => move || {
                counter.set(*counter.get_untracked() + 1);
            }));
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2); // old inner effect should be destroyed and thus not executed
    }

    #[test]
    fn nested_effects_trigger_outer_effect() {
        let trigger = Signal::new(());

        let outer_counter = Signal::new(0);
        let inner_counter = Signal::new(0);
        let inner_cleanup_counter = Signal::new(0);

        create_effect(
            cloned!((trigger, outer_counter, inner_counter, inner_cleanup_counter) => move || {
                trigger.get(); // subscribe to trigger
                outer_counter.set(*outer_counter.get_untracked() + 1);

                create_effect(cloned!((trigger, inner_counter, inner_cleanup_counter) => move || {
                    trigger.set(()); // update trigger which should recreate the outer effect
                    inner_counter.set(*inner_counter.get_untracked() + 1);

                    on_cleanup(cloned!((inner_cleanup_counter) => move || {
                        inner_cleanup_counter.set(*inner_cleanup_counter.get_untracked() + 1);
                    }));
                }));
            }),
        );

        assert_eq!(*outer_counter.get(), 1);
        assert_eq!(*inner_counter.get(), 1);
        assert_eq!(*inner_cleanup_counter.get(), 0);

        trigger.set(());

        assert_eq!(*outer_counter.get(), 2);
        assert_eq!(*inner_counter.get(), 2);
        assert_eq!(*inner_cleanup_counter.get(), 1);
    }

    #[test]
    fn create_nested_effect_from_outside() {
        let trigger = Signal::new(());
        let outer_counter = Signal::new(0);
        let inner_counter = Signal::new(0);

        let inner_effect: Signal<Option<Box<dyn Fn()>>> = Signal::new(None);

        create_effect(
            cloned!((trigger, outer_counter, inner_counter, inner_effect) => move || {
                trigger.get(); // subscribe to trigger
                outer_counter.set(*outer_counter.get_untracked() + 1);

                if inner_effect.get_untracked().is_none() {
                    inner_effect.set(Some(Box::new(cloned!((inner_counter) => move || {
                        inner_counter.set(*inner_counter.get_untracked() + 1);
                    }))));
                }
            }),
        );

        create_effect(move || (*inner_effect.get()).as_ref().unwrap()());

        assert_eq!(*outer_counter.get(), 1);
        assert_eq!(*inner_counter.get(), 1);

        trigger.set(());
        assert_eq!(*outer_counter.get(), 2);
        assert_eq!(*inner_counter.get(), 1);
    }

    #[test]
    fn outer_effects_rerun_first() {
        let trigger = Signal::new(());

        let outer_counter = Signal::new(0);
        let inner_counter = Signal::new(0);

        create_effect(cloned!((trigger, outer_counter, inner_counter) => move || {
            trigger.get(); // subscribe to trigger
            outer_counter.set(*outer_counter.get_untracked() + 1);

            create_effect(cloned!((trigger, inner_counter) => move || {
                trigger.get(); // subscribe to trigger
                inner_counter.set(*inner_counter.get_untracked() + 1);
            }));
        }));

        assert_eq!(*outer_counter.get(), 1);
        assert_eq!(*inner_counter.get(), 1);

        trigger.set(());

        assert_eq!(*outer_counter.get(), 2);
        assert_eq!(*inner_counter.get(), 2);
    }

    #[test]
    fn drop_signal_inside_effect() {
        let state = RefCell::new(Some(Signal::new(0)));

        create_effect(move || {
            if let Some(state) = state.take() {
                state.get(); // subscribe to state
                drop(state);
            }
        });
    }

    #[test]
    fn destroy_effects_on_scope_drop() {
        let counter = Signal::new(0);

        let trigger = Signal::new(());

        let scope = create_scope(cloned!((trigger, counter) => move || {
            create_effect(move || {
                trigger.get(); // subscribe to trigger
                counter.set(*counter.get_untracked() + 1);
            });
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2);

        drop(scope);
        trigger.set(());
        assert_eq!(*counter.get(), 2); // inner effect should be destroyed and thus not executed
    }

    #[test]
    fn effect_preserves_scope_hierarchy() {
        let trigger = Signal::new(());
        let parent = Signal::new(None);
        let scope = create_scope(cloned!((trigger, parent) => move || {
            create_effect(cloned!((trigger, parent) => move || {
                dbg!(debug_scope_hierarchy());
                trigger.get(); // subscribe to trigger
                let p = current_scope().0.upgrade().unwrap().borrow().parent.clone();
                parent.set(Some(p));
            }));
        }));
        assert_eq!(
            Weak::as_ptr(&parent.get().as_ref().clone().unwrap().0) as *const _,
            Rc::as_ptr(&scope.0),
            "parent should be `scope`"
        );
        trigger.set(());
        assert_eq!(
            Weak::as_ptr(&parent.get().as_ref().clone().unwrap().0) as *const _,
            Rc::as_ptr(&scope.0),
            "parent should still be `scope` after effect is re-executed"
        );
    }

    #[test]
    fn memo() {
        let state = Signal::new(0);

        let double = create_memo(cloned!((state) => move || *state.get() * 2));
        assert_eq!(*double.get(), 0);

        state.set(1);
        assert_eq!(*double.get(), 2);

        state.set(2);
        assert_eq!(*double.get(), 4);
    }

    #[test]
    /// Make sure value is memoized rather than executed on demand.
    fn memo_only_run_once() {
        let state = Signal::new(0);

        let counter = Signal::new(0);
        let double = create_memo(cloned!((state, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            *state.get() * 2
        }));
        assert_eq!(*counter.get(), 1); // once for calculating initial derived state

        state.set(2);
        assert_eq!(*counter.get(), 2);
        assert_eq!(*double.get(), 4);
        assert_eq!(*counter.get(), 2); // should still be 2 after access
    }

    #[test]
    fn dependency_on_memo() {
        let state = Signal::new(0);

        let double = create_memo(cloned!((state) => move || *state.get() * 2));

        let quadruple = create_memo(move || *double.get() * 2);

        assert_eq!(*quadruple.get(), 0);

        state.set(1);
        assert_eq!(*quadruple.get(), 4);
    }

    #[test]
    fn untracked_memo() {
        let state = Signal::new(1);

        let double = create_memo(cloned!((state) => move || *state.get_untracked() * 2));

        assert_eq!(*double.get(), 2);

        state.set(2);
        assert_eq!(*double.get(), 2); // double value should still be true because state.get() was
                                      // inside untracked
    }

    #[test]
    fn selector() {
        let state = Signal::new(0);

        let double = create_selector(cloned!((state) => move || *state.get() * 2));

        let counter = Signal::new(0);
        create_effect(cloned!((counter, double) => move || {
            counter.set(*counter.get_untracked() + 1);

            double.get();
        }));
        assert_eq!(*double.get(), 0);
        assert_eq!(*counter.get(), 1);

        state.set(0);
        assert_eq!(*double.get(), 0);
        assert_eq!(*counter.get(), 1); // calling set_state should not trigger the effect

        state.set(2);
        assert_eq!(*double.get(), 4);
        assert_eq!(*counter.get(), 2);
    }

    #[test]
    fn reducer() {
        enum Msg {
            Increment,
            Decrement,
        }

        let (state, dispatch) = create_reducer(0, |state, msg: Msg| match msg {
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
    }

    #[test]
    fn memo_reducer() {
        enum Msg {
            Increment,
            Decrement,
        }

        let (state, dispatch) = create_reducer(0, |state, msg: Msg| match msg {
            Msg::Increment => *state + 1,
            Msg::Decrement => *state - 1,
        });

        let doubled = create_memo(cloned!((state) => move || *state.get() * 2));

        assert_eq!(*doubled.get(), 0);
        dispatch(Msg::Increment);
        assert_eq!(*doubled.get(), 2);
        dispatch(Msg::Decrement);
        assert_eq!(*doubled.get(), 0);
    }

    #[test]
    fn cleanup() {
        let cleanup_called = Signal::new(false);
        let scope = create_scope(cloned!((cleanup_called) => move || {
            on_cleanup(move || {
                cleanup_called.set(true);
            });
        }));

        assert!(!*cleanup_called.get());

        drop(scope);
        assert!(*cleanup_called.get());
    }

    #[test]
    fn cleanup_in_effect() {
        let trigger = Signal::new(());

        let counter = Signal::new(0);

        create_effect(cloned!((trigger, counter) => move || {
            trigger.get(); // subscribe to trigger

            on_cleanup(cloned!((counter) => move || {
                counter.set(*counter.get() + 1);
            }));
        }));

        assert_eq!(*counter.get(), 0);

        trigger.set(());
        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 2);
    }

    #[test]
    fn cleanup_is_untracked() {
        let trigger = Signal::new(());

        let counter = Signal::new(0);

        create_effect(cloned!((trigger, counter) => move || {
            counter.set(*counter.get_untracked() + 1);

            on_cleanup(cloned!((trigger) => move || {
                trigger.get(); // do not subscribe to trigger
            }));
        }));

        assert_eq!(*counter.get(), 1);

        trigger.set(());
        assert_eq!(*counter.get(), 1);
    }

    #[test]
    fn cleanup_in_extended_scope() {
        let counter = Signal::new(0);

        let root = create_scope(cloned!((counter) => move || {
            on_cleanup(cloned!((counter) => move || {
                counter.set(*counter.get_untracked() + 1);
            }));
        }));

        assert_eq!(*counter.get(), 0);

        // Extend the root and add a new on_cleanup callback that increments counter.
        root.extend(cloned!((counter) => move || {
            on_cleanup(cloned!((counter) => move || {
                counter.set(*counter.get_untracked() + 1);
            }));
        }));

        assert_eq!(*counter.get(), 0);

        drop(root);
        assert_eq!(*counter.get(), 2);
    }
}
