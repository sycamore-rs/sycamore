use std::cell::RefCell;
use std::rc::Rc;

use chrono::{prelude::*, Duration};

use crate::utils::{loop_raf, Task};

use super::*;

/// Describes a trait that can be linearly interpolate between two points.
pub trait Lerp {
    /// Get a value between `self` and `other` at a `scalar`.
    ///
    /// `0.0 <= scalar <= 1`
    fn lerp(&self, other: &Self, scalar: f32) -> Self;
}

macro_rules! impl_lerp_for_float {
    ($f: path) => {
        impl Lerp for $f {
            fn lerp(&self, other: &Self, scalar: f32) -> Self {
                self + (other - self) * scalar as $f
            }
        }
    };
}

impl_lerp_for_float!(f32);
impl_lerp_for_float!(f64);

macro_rules! impl_lerp_for_int {
    ($i: path) => {
        impl Lerp for $i {
            fn lerp(&self, other: &Self, scalar: f32) -> Self {
                ((self + (other - self)) as f32 * scalar).round() as $i
            }
        }
    };
}

impl_lerp_for_int!(i8);
impl_lerp_for_int!(i16);
impl_lerp_for_int!(i32);
impl_lerp_for_int!(i64);
impl_lerp_for_int!(i128);

impl_lerp_for_int!(u8);
impl_lerp_for_int!(u16);
impl_lerp_for_int!(u32);
impl_lerp_for_int!(u64);
impl_lerp_for_int!(u128);

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
pub struct Tweened<T: Lerp + Clone + 'static>(Rc<RefCell<TweenedInner<T>>>);

struct TweenedInner<T: Lerp + Clone + 'static> {
    signal: Signal<T>,
    current_task: Option<Task>,
    transition_duration: Duration,
    easing_fn: Rc<dyn Fn(f32) -> f32>,
}

impl<T: Lerp + Clone + 'static> Tweened<T> {
    /// Create a new tweened state with the given value.
    pub fn new(
        initial: T,
        transition_duration: std::time::Duration,
        easing_fn: impl Fn(f32) -> f32 + 'static,
    ) -> Self {
        Self(Rc::new(RefCell::new(TweenedInner {
            signal: Signal::new(initial),
            current_task: None,
            transition_duration: Duration::from_std(transition_duration)
                .expect("transition_duration is greater than the maximum value"),
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

        let start_time = Utc::now();
        let signal = self.0.borrow().signal.clone();
        let transition_duration = self.0.borrow().transition_duration;

        let task = Task::new(move || {
            let now = Utc::now();

            let since_start = now - start_time;
            let scalar = since_start.num_milliseconds() as f32
                / transition_duration.num_milliseconds() as f32;

            if now < start_time + transition_duration {
                signal.set(start.lerp(&new_value, easing_fn(scalar)));
                true
            } else {
                signal.set(new_value.clone());
                false
            }
        });

        if let Some(previous_task) = self.0.borrow_mut().current_task.as_mut() {
            previous_task.abort();
        }

        self.0.borrow_mut().current_task = Some(task.clone());

        loop_raf(task);
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
    pub fn signal(&self) -> Signal<T> {
        self.0.borrow().signal.clone()
    }
}

impl<T: Lerp + Clone + 'static> Clone for Tweened<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T: Lerp + Clone + 'static> Clone for TweenedInner<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
            current_task: self.current_task.clone(),
            transition_duration: self.transition_duration,
            easing_fn: Rc::clone(&self.easing_fn),
        }
    }
}
