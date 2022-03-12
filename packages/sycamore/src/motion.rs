//! Utilities for smooth transitions and animations.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use js_sys::Date;
use wasm_bindgen::{prelude::*, JsCast};

use crate::reactive::*;

/// Type returned by `create_raf` and `create_raf_loop`.
type RafState<'a> = (RcSignal<bool>, &'a dyn Fn(), &'a dyn Fn());

/// Extension trait for [`Scope`] adding methods related to animation frames and tweened signals.
pub trait ScopeMotionExt<'a> {
    /// Schedule a callback to be called on each animation frame.
    /// Does nothing if not on `wasm32` target.
    ///
    /// Returns a tuple of `(running, start, stop)`. The first item is a boolean signal representing
    /// whether the raf is currently running. The second item is a function to start the raf. The
    /// third item is a function to stop the raf.
    ///
    /// The raf is not started by default. Call the `start` function to initiate the raf.
    fn create_raf(self, f: impl FnMut() + 'a) -> RafState<'a>;

    /// Schedule a callback to be called on each animation frame.
    /// Does nothing if not on `wasm32` target.
    ///
    /// Instead of using `start` and `stop` functions, the callback is kept on looping until it
    /// returns `false`. `start` and `stop` are returned regardless to allow controlling the
    /// looping from outside the function.
    ///
    /// The raf is not started by default. Call the `start` function to initiate the raf.
    fn create_raf_loop(self, f: impl FnMut() -> bool + 'a) -> RafState<'a>;

    /// Create a new [`Tweened`] signal.
    fn create_tweened_signal<T: Lerp + Clone + 'a>(
        self,
        initial: T,
        transition_duration: std::time::Duration,
        easing_fn: impl Fn(f32) -> f32 + 'static,
    ) -> &'a Tweened<'a, T>;
}

impl<'a> ScopeMotionExt<'a> for Scope<'a> {
    fn create_raf(self, f: impl FnMut() + 'a) -> RafState<'a> {
        let running = self.create_ref(create_rc_signal(false));
        let start: &dyn Fn();
        let stop: &dyn Fn();

        if cfg!(target_arch = "wasm32") {
            // Only run on wasm32 architecture.
            let boxed: Box<dyn FnMut() + 'a> = Box::new(f);
            // SAFETY: We are only transmuting the lifetime from 'a to 'static which is safe because
            // the closure will not be accessed once the enclosing Scope is disposed.
            let extended: Box<dyn FnMut() + 'static> = unsafe { std::mem::transmute(boxed) };
            let extended = RefCell::new(extended);
            let scope_status = self.use_scope_status();

            let f = Rc::new(RefCell::new(None::<Closure<dyn Fn()>>));
            let g = Rc::clone(&f);

            *g.borrow_mut() = Some(Closure::wrap(Box::new({
                let running = running.clone();
                move || {
                    if *scope_status.get() && *running.get() {
                        // Verified that scope is still valid. We can access `extended` in here.
                        extended.borrow_mut()();
                        // Request the next raf frame.
                        web_sys::window()
                            .unwrap_throw()
                            .request_animation_frame(
                                f.borrow().as_ref().unwrap_throw().as_ref().unchecked_ref(),
                            )
                            .unwrap_throw();
                    }
                }
            })));
            start = self.create_ref(move || {
                if !*running.get() {
                    running.set(true);
                    web_sys::window()
                        .unwrap_throw()
                        .request_animation_frame(
                            g.borrow().as_ref().unwrap_throw().as_ref().unchecked_ref(),
                        )
                        .unwrap_throw();
                }
            });
            stop = self.create_ref(|| running.set(false));
        } else {
            start = self.create_ref(|| running.set(true));
            stop = self.create_ref(|| running.set(false));
        }

        (running.clone(), start, stop)
    }

    fn create_raf_loop(self, mut f: impl FnMut() -> bool + 'a) -> RafState<'a> {
        let stop_shared = self.create_ref(Cell::new(None::<&dyn Fn()>));
        let (running, start, stop) = self.create_raf(move || {
            if !f() {
                stop_shared.get().unwrap()();
            }
        });
        stop_shared.set(Some(stop));
        (running, start, stop)
    }

    fn create_tweened_signal<T: Lerp + Clone + 'a>(
        self,
        initial: T,
        transition_duration: std::time::Duration,
        easing_fn: impl Fn(f32) -> f32 + 'static,
    ) -> &'a Tweened<'a, T> {
        self.create_ref(Tweened::new(self, initial, transition_duration, easing_fn))
    }
}

/// Describes a trait that can be linearly interpolate between two points.
pub trait Lerp {
    /// Get a value between `self` and `other` at a `scalar`.
    ///
    /// `0.0 <= scalar <= 1`
    fn lerp(&self, other: &Self, scalar: f32) -> Self;
}

macro_rules! impl_lerp_for_float {
    ($($f: path),*) => {
        $(
            impl Lerp for $f {
                fn lerp(&self, other: &Self, scalar: f32) -> Self {
                    self + (other - self) * scalar as $f
                }
            }
        )*
    };
}

impl_lerp_for_float!(f32, f64);

macro_rules! impl_lerp_for_int {
    ($($i: path),*) => {
        $(
            impl Lerp for $i {
                fn lerp(&self, other: &Self, scalar: f32) -> Self {
                    (*self as f32 + (other - self) as f32 * scalar).round() as $i
                }
            }
        )*
    };
}

impl_lerp_for_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl<T: Lerp + Clone, const N: usize> Lerp for [T; N] {
    fn lerp(&self, other: &Self, scalar: f32) -> Self {
        let mut tmp = (*self).clone();

        for (t, other) in tmp.iter_mut().zip(other) {
            *t = t.lerp(other, scalar);
        }

        tmp
    }
}

/// A state that is interpolated when it is set.
pub struct Tweened<'a, T: Lerp + Clone>(Rc<RefCell<TweenedInner<'a, T>>>);

struct TweenedInner<'a, T: Lerp + Clone + 'a> {
    /// The [`Scope`] under which the tweened signal was created. We need to hold on to the
    /// context to be able to spawn the raf callback.
    ctx: Scope<'a>,
    value: RcSignal<T>,
    raf_state: Option<RafState<'a>>,
    transition_duration_ms: f32,
    easing_fn: Rc<dyn Fn(f32) -> f32>,
}

impl<'a, T: Lerp + Clone + 'a> Tweened<'a, T> {
    /// Create a new tweened state with the given value.
    ///
    /// End users should use [`Scope::create_tweened_signal`] instead.
    pub(crate) fn new(
        ctx: Scope<'a>,
        initial: T,
        transition_duration: std::time::Duration,
        easing_fn: impl Fn(f32) -> f32 + 'static,
    ) -> Self {
        let value = create_rc_signal(initial);
        Self(Rc::new(RefCell::new(TweenedInner {
            ctx,
            value,
            raf_state: None,
            transition_duration_ms: transition_duration.as_millis() as f32,
            easing_fn: Rc::new(easing_fn),
        })))
    }

    /// Set the target value for the `Tweened`. The existing value will be interpolated to the
    /// target value with the specified `transition_duration` and `easing_fn`.
    ///
    /// If the value is being interpolated already due to a previous call to `set()`, the previous
    /// task will be canceled.
    ///
    /// To immediately set the value without interpolating the value, use `signal().set(...)`
    /// instead.
    pub fn set(&self, new_value: T) {
        let start = self.signal().get_untracked().as_ref().clone();
        let easing_fn = Rc::clone(&self.0.borrow().easing_fn);

        let start_time = Date::now();
        let signal = self.0.borrow().value.clone();
        let transition_duration_ms = self.0.borrow().transition_duration_ms;

        // If previous raf is still running, call stop() to cancel it.
        if let Some((running, _, stop)) = &self.0.borrow_mut().raf_state {
            if *running.get_untracked() {
                stop();
            }
        }

        let (running, start, stop) = self.0.borrow().ctx.create_raf_loop(move || {
            let now = Date::now();

            let since_start = now - start_time;
            let scalar = since_start as f32 / transition_duration_ms;

            if now < start_time + transition_duration_ms as f64 {
                signal.set(start.lerp(&new_value, easing_fn(scalar)));
                true
            } else {
                signal.set(new_value.clone());
                false
            }
        });
        start();
        self.0.borrow_mut().raf_state = Some((running, start, stop));
    }

    /// Alias for `signal().get()`.
    pub fn get(&self) -> Rc<T> {
        self.signal().get()
    }

    /// Alias for `signal().get_untracked()`.
    pub fn get_untracked(&self) -> Rc<T> {
        self.signal().get_untracked()
    }

    /// Get the inner signal backing the state.
    pub fn signal(&self) -> RcSignal<T> {
        self.0.borrow().value.clone()
    }
}

impl<'a, T: Lerp + Clone + 'static> Clone for Tweened<'a, T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<'a, T: Lerp + Clone + 'static> Clone for TweenedInner<'a, T> {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx,
            value: self.value.clone(),
            raf_state: self.raf_state.clone(),
            transition_duration_ms: self.transition_duration_ms,
            easing_fn: Rc::clone(&self.easing_fn),
        }
    }
}
