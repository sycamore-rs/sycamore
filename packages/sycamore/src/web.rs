//! Web support for Sycamore.

pub use sycamore_web::*;

#[allow(unused_imports)]
use crate::prelude::*;

/// Render a [`View`] into a static [`String`]. Useful
/// for rendering to a string on the server side.
///
/// Waits for suspense to be loaded before returning.
///
/// _This API requires the following crate features to be activated: `suspense`, `ssr`_
#[cfg(all(feature = "ssr", feature = "suspense"))]
pub async fn render_to_string_await_suspense(view: impl FnOnce() -> View + 'static) -> String {
    let _ = view;
    // use std::cell::RefCell;
    // use std::rc::Rc;
    //
    // use futures::channel::oneshot;
    // use sycamore_futures::spawn_local_scoped;
    //
    // use crate::utils::hydrate::with_hydration_context_async;
    //
    // let mut ret = String::new();
    // let v = Rc::new(RefCell::new(None));
    // let (sender, receiver) = oneshot::channel();
    // let disposer = create_root({
    //     let v = Rc::clone(&v);
    //     move || {
    //         spawn_local_scoped(async move {
    //             *v.borrow_mut() = Some(
    //                 with_hydration_context_async(async {
    //                     crate::suspense::await_suspense(async { view() }).await
    //                 })
    //                 .await,
    //             );
    //             sender
    //                 .send(())
    //                 .expect("receiving end should not be dropped");
    //         });
    //     }
    // });
    // receiver.await.expect("rendering should complete");
    // let v = v.borrow().clone().unwrap();
    // for node in v.flatten() {
    //     node.write_to_string(&mut ret);
    // }
    //
    // disposer.dispose();
    //
    // ret
    todo!()
}
