use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use sycamore::generic_node::EventHandler;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlAnchorElement, HtmlBaseElement, KeyboardEvent};

use crate::Route;

/// A router integration provides the methods for adapting a router to a certain environment (e.g.
/// history API).
pub trait Integration {
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
    fn current_pathname(&self) -> String {
        web_sys::window()
            .unwrap_throw()
            .location()
            .pathname()
            .unwrap_throw()
    }

    fn on_popstate(&self, f: Box<dyn FnMut()>) {
        let closure = Closure::wrap(f);
        web_sys::window()
            .unwrap_throw()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    }

    fn click_handler(&self) -> Box<EventHandler> {
        Box::new(|ev| {
            if let Some(a) = ev
                .target()
                .unwrap_throw()
                .unchecked_into::<Element>()
                .closest("a[href]")
                .unwrap_throw()
            {
                let location = web_sys::window().unwrap_throw().location();

                let a = a.unchecked_into::<HtmlAnchorElement>();

                // Check if a has `rel="external"`.
                if a.rel() == "external" {
                    // Use default browser behaviour.
                    return;
                }

                let origin = a.origin();
                let path = a.pathname();
                let hash = a.hash();

                let meta_keys_pressed = meta_keys_pressed(ev.unchecked_ref::<KeyboardEvent>());
                if !meta_keys_pressed && Ok(origin) == location.origin() {
                    if Ok(&path) != location.pathname().as_ref() {
                        // Same origin, different path.
                        ev.prevent_default();
                        PATHNAME.with(|pathname| {
                            let pathname = pathname.borrow().clone().unwrap_throw();
                            let path = path.strip_prefix(&base_pathname()).unwrap_or(&path);
                            pathname.set(path.to_string());

                            // Update History API.
                            let window = web_sys::window().unwrap_throw();
                            let history = window.history().unwrap_throw();
                            history
                                .push_state_with_url(
                                    &JsValue::UNDEFINED,
                                    "",
                                    Some(pathname.get().as_str()),
                                )
                                .unwrap_throw();
                            window.scroll_to_with_x_and_y(0.0, 0.0);
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

/// Gets the base pathname from `document.baseURI`.
fn base_pathname() -> String {
    match web_sys::window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .query_selector("base[href]")
    {
        Ok(Some(base)) => {
            let base = base.unchecked_into::<HtmlBaseElement>().href();

            let url = web_sys::Url::new(&base).unwrap_throw();
            url.pathname()
        }
        _ => "".to_string(),
    }
}

/// Props for [`Router`].
pub struct RouterProps<R, F, G>
where
    R: Route + 'static,
    F: FnOnce(StateHandle<R>) -> Template<G>,
    G: GenericNode,
{
    render: F,
    integration: Box<dyn Integration>,
    _phantom: PhantomData<*const (R, G)>,
}

impl<R, F, G> RouterProps<R, F, G>
where
    R: Route + 'static,
    F: FnOnce(StateHandle<R>) -> Template<G>,
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

/// The sycamore router component. This component expects to be used inside a browser environment.
/// For server environments, see [`StaticRouter`].
#[component(Router<G>)]
pub fn router<R, F>(props: RouterProps<R, F, G>) -> Template<G>
where
    R: Route + 'static,
    F: FnOnce(StateHandle<R>) -> Template<G> + 'static,
{
    let RouterProps {
        render,
        integration,
        _phantom,
    } = props;
    let render = Rc::new(RefCell::new(Some(render)));
    let integration = Rc::new(integration);
    let base_pathname = base_pathname();

    PATHNAME.with(|pathname| {
        assert!(pathname.borrow().is_none());
        // Get initial url from window.location.
        let path = integration.current_pathname();
        let path = path.strip_prefix(&base_pathname).unwrap_or(&path);
        *pathname.borrow_mut() = Some(Signal::new(path.to_string()));
    });
    let pathname = PATHNAME.with(|p| p.borrow().clone().unwrap_throw());

    // Set PATHNAME to None when the Router is destroyed.
    on_cleanup(|| {
        PATHNAME.with(|pathname| {
            *pathname.borrow_mut() = None;
        });
    });

    // Listen to popstate event.
    integration.on_popstate(Box::new(cloned!((integration, pathname) => move || {
        let path = integration.current_pathname();
        let path = path.strip_prefix(&base_pathname).unwrap_or(&path);
        pathname.set(path.to_string());
    })));

    let path = create_selector(move || {
        pathname
            .get()
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    });

    let route_signal: Rc<RefCell<Option<Signal<R>>>> = Rc::new(RefCell::new(None));
    create_effect(cloned!((route_signal) => move || {
        let path = path.get();
        let route = R::match_route(path.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice());
        if route_signal.borrow().is_some() {
            route_signal.borrow().as_ref().unwrap_throw().set(route);
        } else {
            *route_signal.borrow_mut() = Some(Signal::new(route));
        }
    }));
    // Delegate click events from child <a> tags.
    let tmp = render.take().unwrap_throw()(route_signal.borrow().as_ref().unwrap_throw().handle());
    if let Some(node) = tmp.as_node() {
        node.event("click", integration.click_handler());
    } else {
        // TODO: support fragments and lazy nodes
        panic!("render should return a single node");
    }
    tmp
}

/// Props for [`StaticRouter`].
pub struct StaticRouterProps<R, F, G>
where
    R: Route,
    F: Fn(R) -> Template<G>,
    G: GenericNode,
{
    render: F,
    route: R,
    _phantom: PhantomData<*const (R, G)>,
}

impl<R, F, G> StaticRouterProps<R, F, G>
where
    R: Route,
    F: Fn(R) -> Template<G>,
    G: GenericNode,
{
    /// Create a new [`StaticRouterProps`].
    pub fn new(route: R, render: F) -> Self {
        Self {
            render,
            route,
            _phantom: PhantomData,
        }
    }
}

/// A router that only renders once with the given `route`.
///
/// This is useful for SSR where we want the HTML to be rendered instantly instead of waiting for
/// the route preload to finish loading.
#[component(StaticRouter<G>)]
pub fn static_router<R, F>(props: StaticRouterProps<R, F, G>) -> Template<G>
where
    R: Route + 'static,
    F: Fn(R) -> Template<G> + 'static,
{
    let StaticRouterProps {
        render,
        route,
        _phantom,
    } = props;

    render(route)
}

/// Navigates to the specified `url`. The url should have the same origin as the app.
///
/// This is useful for imperatively navigating to an url when using an anchor tag (`<a>`) is not
/// possible/suitable (e.g. when submitting a form).
///
/// # Panics
/// This function will `panic!()` if a [`Router`] has not yet been created.
pub fn navigate(url: &str) {
    PATHNAME.with(|pathname| {
        assert!(
            pathname.borrow().is_some(),
            "navigate can only be used with a BrowserRouter"
        );

        let pathname = pathname.borrow().clone().unwrap_throw();
        let path = url.strip_prefix(&base_pathname()).unwrap_or(url);
        pathname.set(path.to_string());

        // Update History API.
        let window = web_sys::window().unwrap_throw();
        let history = window.history().unwrap_throw();
        history
            .push_state_with_url(&JsValue::UNDEFINED, "", Some(pathname.get().as_str()))
            .unwrap_throw();
        window.scroll_to_with_x_and_y(0.0, 0.0);
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
            let route = Routes::match_route(
                &path
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>(),
            );

            template! {
                StaticRouter(StaticRouterProps::new(route, |route: Routes| {
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
