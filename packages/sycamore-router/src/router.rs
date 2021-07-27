use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use sycamore::generic_node::EventHandler;
use sycamore::prelude::*;
use sycamore::rx::ReactiveScope;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, HtmlAnchorElement, KeyboardEvent};

use crate::Route;

/// A router integration provides the methods for adapting a router to a certain environment (e.g.
/// server or browser).
pub trait Integration {
    /// Get the initial pathname.
    fn initial_pathname(&self) -> String;

    /// Get the current pathname.
    fn current_pathname(&self) -> String;

    /// Add a callback for listening to the `popstate` event.
    fn on_popstate(&self, f: Box<dyn FnMut()>);

    /// Get the click handler that is run when links are clicked.

    fn click_handler(&self) -> Box<EventHandler>;
}

thread_local! {
    static PATHNAME: RefCell<Option<Signal<String>>> = RefCell::new(None);
}

/// A router that never changes path. Useful for SSR when the app will only be rendered once.
pub struct StaticIntegration {
    pathname: String,
}

impl StaticIntegration {
    /// Create a new [`StaticRouter`] with the given initial pathname.
    pub fn new(initial_pathname: String) -> Self {
        Self {
            pathname: initial_pathname,
        }
    }
}

impl Integration for StaticIntegration {
    fn initial_pathname(&self) -> String {
        self.pathname.clone()
    }

    fn current_pathname(&self) -> String {
        unreachable!()
    }

    fn on_popstate(&self, _: Box<dyn FnMut()>) {
        // no-op
        // Path never changes for a static router.
    }

    fn click_handler(&self) -> Box<EventHandler> {
        Box::new(|_| {
            // no-op
            // This will ensure that `current_pathname` and `on_popstate` will never be called.
        })
    }
}

/// A router integration that uses the
/// [HTML5 History API](https://developer.mozilla.org/en-US/docs/Web/API/History_API) to keep the
/// UI in sync with the URL.
#[derive(Default)]
pub struct HistoryIntegration {
    /// This field is to prevent downstream users from creating a new `HistoryIntegration` without
    /// the `new` method.
    _internal: (),
}

impl HistoryIntegration {
    /// Create a new [`HistoryIntegration`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl Integration for HistoryIntegration {
    fn initial_pathname(&self) -> String {
        web_sys::window().unwrap().location().pathname().unwrap()
    }

    fn current_pathname(&self) -> String {
        web_sys::window().unwrap().location().pathname().unwrap()
    }

    fn on_popstate(&self, f: Box<dyn FnMut()>) {
        let closure = Closure::wrap(f);
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    fn click_handler(&self) -> Box<EventHandler> {
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

                let meta_keys_pressed = meta_keys_pressed(ev.unchecked_ref::<KeyboardEvent>());
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
        })
    }
}

/// Props for [`Router`].
pub struct RouterProps<R, F, G>
where
    R: Route,
    F: Fn(R) -> Template<G>,
    G: GenericNode,
{
    render: F,
    integration: Box<dyn Integration>,
    _phantom: PhantomData<*const (R, G)>,
}

impl<R, F, G> RouterProps<R, F, G>
where
    R: Route,
    F: Fn(R) -> Template<G> + 'static,
    G: GenericNode,
{
    /// Create a new [`RouterProps`].
    pub fn new(integration: impl Integration + 'static, render: F) -> Self {
        Self {
            render,
            integration: Box::new(integration),
            _phantom: PhantomData,
        }
    }
}

/// The sycamore router component.
#[component(Router<G>)]
pub fn router<R, F>(props: RouterProps<R, F, G>) -> Template<G>
where
    R: Route + 'static,
    F: Fn(R) -> Template<G> + 'static,
{
    let RouterProps {
        render,
        integration,
        _phantom,
    } = props;
    let render = Rc::new(render);
    let integration = Rc::new(integration);

    PATHNAME.with(|pathname| {
        assert!(pathname.borrow().is_none());
        // Get initial url from window.location.
        *pathname.borrow_mut() = Some(Signal::new(integration.initial_pathname()));
    });
    let pathname = PATHNAME.with(|p| p.borrow().clone().unwrap());

    // Set PATHNAME to None when the Router is destroyed.
    on_cleanup(|| {
        PATHNAME.with(|pathname| {
            *pathname.borrow_mut() = None;
        });
    });

    // Listen to popstate event.
    integration.on_popstate(Box::new(cloned!((integration, pathname) => move || {
        pathname.set(integration.current_pathname());
    })));

    let path = create_selector(move || {
        pathname
            .get()
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    });

    let template = Signal::new((ReactiveScope::new(),Template::empty()));
    create_effect(cloned!((template) => move || {
        let path = path.get();
        spawn_local(cloned!((render, integration, path, template) => async move {
            let route = R::match_route(path.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice()).await;
            // Delegate click events from child <a> tags.
            let mut t = None;
            let scope = create_root(|| {
                let tmp = render(route);
                if let Some(node) = tmp.as_node() {
                    node.event("click", integration.click_handler());
                } else {
                    // TODO: support fragments and lazy nodes
                    panic!("render should return a single node");
                }
                t = Some(tmp);
            });
            template.set((scope, t.unwrap()));
        }));
    }));

    Template::new_dyn(move || template.get().as_ref().1.clone())
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

#[cfg(test)]
mod tests {
    use sycamore::prelude::*;

    use super::*;

    #[test]
    #[ignore = "not implemented on non wasm32 target"]
    fn static_router() {
        #[derive(Route)]
        enum Routes {
            #[to("/")]
            Home,
            #[to("/about")]
            About,
            #[not_found]
            NotFound,
        }

        #[component(Comp<G>)]
        fn comp(path: String) -> Template<G> {
            template! {
                Router(RouterProps::new(StaticIntegration::new(path), |route: Routes| {
                    match route {
                        Routes::Home => template! {
                            "Home"
                        },
                        Routes::About => template! {
                            "About"
                        },
                        Routes::NotFound => template! {
                            "Not Found"
                        }
                    }
                }))
            }
        }

        assert_eq!(
            sycamore::render_to_string(|| template! { Comp("/".to_string()) }),
            "Home"
        );

        assert_eq!(
            sycamore::render_to_string(|| template! { Comp("/about".to_string()) }),
            "About"
        );

        assert_eq!(
            sycamore::render_to_string(|| template! { Comp("/404".to_string()) }),
            "Not Found"
        );
    }
}
