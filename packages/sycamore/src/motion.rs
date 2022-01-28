//! Utilities for smooth transitions and animations.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{prelude::*, JsCast};

use crate::reactive::*;

pub trait ScopeCreateRaf<'a> {
    fn create_raf(&'a self, f: impl FnMut() + 'a) -> (RcSignal<bool>, &'a dyn Fn(), &'a dyn Fn());
}

impl<'a> ScopeCreateRaf<'a> for Scope<'a> {
    fn create_raf(&'a self, f: impl FnMut() + 'a) -> (RcSignal<bool>, &'a dyn Fn(), &'a dyn Fn()) {
        let running = self.create_ref(create_rc_signal(true));
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
                running.set(true);
                web_sys::window()
                    .unwrap_throw()
                    .request_animation_frame(
                        g.borrow().as_ref().unwrap_throw().as_ref().unchecked_ref(),
                    )
                    .unwrap_throw();
            });
            stop = self.create_ref(|| running.set(false));

            start();
        } else {
            start = self.create_ref(|| running.set(true));
            stop = self.create_ref(|| running.set(false));
        }

        (running.clone(), start, stop)
    }
}
