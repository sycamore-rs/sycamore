//! Internal utilities for Sycamore.
//!
//! # Stability
//! This API is currently unstable and can have breaking changed without a semver release.
//! This might be stabilized in the future but it is use-at-your-own-risk for now.

pub mod render;

use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

use ahash::AHashSet;

thread_local! {
    static TASKS: RefCell<AHashSet<Task>> = RefCell::new(AHashSet::new());
}

/// A wrapper over a callback. Used with [`loop_raf`].
#[derive(Clone)]
pub struct Task {
    callback: Rc<dyn Fn() -> bool>,
}

impl Task {
    pub fn new(callback: impl Fn() -> bool + 'static) -> Self {
        Self {
            callback: Rc::new(callback),
        }
    }

    pub fn abort(&self) {
        TASKS.with(|tasks| {
            tasks.borrow_mut().remove(self);
        });
    }
}

impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.callback).hash(state);
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq::<()>(
            Rc::as_ptr(&self.callback).cast(),
            Rc::as_ptr(&other.callback).cast(),
        )
    }
}
impl Eq for Task {}

#[cfg(feature = "dom")]
pub(crate) fn run_tasks() {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    let f = Rc::new(RefCell::new(None::<Closure<dyn Fn()>>));
    let g = Rc::clone(&f);

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        TASKS.with(|tasks| {
            let task_list = (*tasks.borrow()).clone();
            for task in task_list {
                if !(task.callback)() {
                    tasks.borrow_mut().remove(&task);
                }
            }

            if tasks.borrow().is_empty() {
                let callback = f.take();
                drop(callback);
            } else {
                web_sys::window()
                    .unwrap_throw()
                    .request_animation_frame(
                        f.borrow().as_ref().unwrap_throw().as_ref().unchecked_ref(),
                    )
                    .unwrap_throw();
            }
        });
    })));

    web_sys::window()
        .unwrap_throw()
        .request_animation_frame(g.borrow().as_ref().unwrap_throw().as_ref().unchecked_ref())
        .unwrap_throw();
}

#[cfg(not(feature = "dom"))]
pub(crate) fn run_tasks() {
    // noop on non web targets
}

/// Runs a callback in a `requestAnimationFrame` loop until the `callback` returns `false`.
pub fn loop_raf(task: Task) {
    TASKS.with(|tasks| {
        if tasks.borrow().is_empty() {
            run_tasks();
        }

        tasks.borrow_mut().insert(task);
    });
}
