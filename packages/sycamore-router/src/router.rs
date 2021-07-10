use std::cell::RefCell;

use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlAnchorElement, KeyboardEvent};

use crate::Route;

/// A router that never changes location. Useful for SSR when the app will never change URL.
#[component(StaticRouter<G>)]
pub fn static_router<R: Route>(
    (pathname, render): (String, impl Fn(R) -> Template<G> + 'static),
) -> Template<G> {
    let path = pathname
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let route = R::match_route(&path);
    render(route)
}

thread_local! {
    static PATHNAME: RefCell<Option<Signal<String>>> = RefCell::new(None);
}

/// A router that uses the
/// [HTML5 History API](https://developer.mozilla.org/en-US/docs/Web/API/History_API) to keep the
/// UI in sync with the URL.
#[component(BrowserRouter<G>)]
pub fn browser_router<R: Route>(render: impl Fn(R) -> Template<G> + 'static) -> Template<G> {
    PATHNAME.with(|pathname| {
        assert!(pathname.borrow().is_none());
        // Get initial url from window.location.
        *pathname.borrow_mut() = Some(Signal::new(
            web_sys::window().unwrap().location().pathname().unwrap(),
        ));
    });
    let pathname = PATHNAME.with(|p| p.borrow().clone().unwrap());

    // Listen to popstate event.
    let closure = Closure::wrap(Box::new(cloned!((pathname) => move || {
        pathname.set(web_sys::window().unwrap().location().pathname().unwrap());
    })) as Box<dyn FnMut()>);
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    let path = create_selector(move || {
        pathname
            .get()
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    });

    Template::new_dyn(move || {
        let route = R::match_route(
            path.get()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        // Delegate click events from child <a> tags.
        let template = untrack(|| render(route));
        if let Some(node) = template.as_node() {
            node.event(
                "click",
                Box::new(|ev| {
                    if let Some(a) = ev
                        .target()
                        .unwrap()
                        .unchecked_into::<Element>()
                        .closest("a[href]")
                        .unwrap()
                    {
                        let location = web_sys::window().unwrap().location();

                        let a = a.unchecked_into::<HtmlAnchorElement>();
                        let origin = a.origin();
                        let path = a.pathname();
                        let hash = a.hash();

                        let meta_keys_pressed =
                            meta_keys_pressed(ev.unchecked_ref::<KeyboardEvent>());
                        if !meta_keys_pressed && Ok(origin) == location.origin() {
                            if Ok(&path) != location.pathname().as_ref() {
                                // Same origin, different path.
                                ev.prevent_default();
                                PATHNAME.with(|pathname| {
                                    let pathname = pathname.borrow().clone().unwrap();
                                    pathname.set(path.to_string());

                                    // Update History API.
                                    let history = web_sys::window().unwrap().history().unwrap();
                                    history
                                        .push_state_with_url(
                                            &JsValue::UNDEFINED,
                                            "",
                                            Some(pathname.get().as_str()),
                                        )
                                        .unwrap();
                                });
                            } else if Ok(&hash) != location.hash().as_ref() {
                                // Same origin, same path, different anchor.
                                // Use default browser behavior.
                            } else {
                                // Same page. Do nothing.
                                ev.prevent_default();
                            }
                        }
                    }
                }),
            );
        } else {
            panic!("render should return a single node");
        }

        template
    })
}

/// Navigates to the specified `url`. The url should have the same origin as the app.
///
/// This is useful for imperatively navigating to an url when using an anchor tag (`<a>`) is not
/// possible/suitable (e.g. when submitting a form).
///
/// # Panics
/// This function will `panic!()` if a [`BrowserRouter`] has not yet been created.
pub fn navigate(url: &str) {
    PATHNAME.with(|pathname| {
        assert!(
            pathname.borrow().is_some(),
            "navigate can only be used with a BrowserRouter"
        );

        let pathname = pathname.borrow().clone().unwrap();
        pathname.set(url.to_string());

        // Update History API.
        let history = web_sys::window().unwrap().history().unwrap();
        history
            .push_state_with_url(&JsValue::UNDEFINED, "", Some(pathname.get().as_str()))
            .unwrap();
    });
}

fn meta_keys_pressed(kb_event: &KeyboardEvent) -> bool {
    kb_event.meta_key() || kb_event.ctrl_key() || kb_event.shift_key() || kb_event.alt_key()
}
