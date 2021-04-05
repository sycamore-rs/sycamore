use std::cell::RefCell;
use std::rc::Rc;

use chrono::{prelude::*, Duration};

use crate::utils::{loop_raf, Task};

use super::*;

/// Describes a trait that can be linearly interpolate between two points.
pub trait Lerp {
    fn lerp(&self, other: &Self, scalar: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, scalar: f32) -> Self {
        self + (other - self) * scalar
    }
}
pub struct Tweened<T: Lerp + Clone + 'static>(RefCell<TweenedInner<T>>);

struct TweenedInner<T: Lerp + Clone + 'static> {
    signal: Signal<T>,
    current_task: Option<Task>,
    transition_duration: Duration,
}

impl<T: Lerp + Clone + 'static> Tweened<T> {
    pub fn new(initial: T, transition_duration: std::time::Duration) -> Self {
        Self(RefCell::new(TweenedInner {
            signal: Signal::new(initial),
            current_task: None,
            transition_duration: Duration::from_std(transition_duration)
                .expect("transition_duration is greater than the maximum value"),
        }))
    }

    pub fn set(&self, new_value: T) {
        let start = self.signal().get_untracked().as_ref().clone();

        let start_time = Utc::now();
        let signal = self.0.borrow().signal.clone();
        let transition_duration = self.0.borrow().transition_duration;

        let task = Task::new(move || {
            let now = Utc::now();

            let since_start = now - start_time;
            let scalar = since_start.num_milliseconds() as f32
                / transition_duration.num_milliseconds() as f32;

            signal.set(start.lerp(&new_value, scalar));

            now < start_time + transition_duration
        });

        if let Some(previous_task) = self.0.borrow_mut().current_task.as_mut() {
            previous_task.abort();
        }

        self.0.borrow_mut().current_task = Some(task.clone());

        loop_raf(task);
    }

    pub fn get(&self) -> Rc<T> {
        self.signal().get()
    }

    pub fn get_untracked(&self) -> Rc<T> {
        self.signal().get_untracked()
    }

    pub fn signal(&self) -> Signal<T> {
        self.0.borrow().signal.clone()
    }
}

impl<T: Lerp + Clone + 'static> Clone for Tweened<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Lerp + Clone + 'static> Clone for TweenedInner<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
            current_task: self.current_task.clone(),
            transition_duration: self.transition_duration,
        }
    }
}
