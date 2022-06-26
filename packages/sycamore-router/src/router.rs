use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

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

    fn click_handler(&self) -> Box<dyn Fn(web_sys::Event)>;
}

thread_local! {
    static PATHNAME: RefCell<Option<RcSignal<String>>> = RefCell::new(None);
}

/// A router integration that uses the
/// [HTML5 History API](https://developer.mozilla.org/en-US/docs/Web/API/History_API) to keep the
/// UI in sync with the URL.
#[derive(Default, Debug)]
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

    fn click_handler(&self) -> Box<dyn Fn(web_sys::Event)> {
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
                let a_pathname = a.pathname();
                let hash = a.hash();

                let meta_keys_pressed = meta_keys_pressed(ev.unchecked_ref::<KeyboardEvent>());
                if !meta_keys_pressed && location.origin() == Ok(origin) {
                    if location.pathname().as_ref() != Ok(&a_pathname) {
                        // Same origin, different path.
                        ev.prevent_default();
                        PATHNAME.with(|pathname| {
                            let pathname = pathname.borrow().clone().unwrap_throw();
                            let path = a_pathname
                                .strip_prefix(&base_pathname())
                                .unwrap_or(&a_pathname);
                            pathname.set(path.to_string());

                            // Update History API.
                            let window = web_sys::window().unwrap_throw();
                            let history = window.history().unwrap_throw();
                            history
                                .push_state_with_url(&JsValue::UNDEFINED, "", Some(&a_pathname))
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
            let mut pathname = url.pathname();
            // Strip trailing `/` character from the pathname.
            pathname.ends_with('/');
            pathname.pop(); // Pop the `/` character.
            pathname
        }
        _ => "".to_string(),
    }
}

/// Props for [`Router`].
#[derive(Prop, Debug)]
pub struct RouterProps<'a, R, F, I, G>
where
    R: Route + 'a,
    F: FnOnce(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    I: Integration,
    G: GenericNode,
{
    view: F,
    integration: I,
    #[builder(default, setter(skip))]
    _phantom: PhantomData<&'a (R, G)>,
}

impl<'a, R, F, I, G> RouterProps<'a, R, F, I, G>
where
    R: Route + 'a,
    F: FnOnce(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    I: Integration,
    G: GenericNode,
{
    /// Create a new [`RouterProps`].
    pub fn new(integration: I, view: F) -> Self {
        Self {
            view,
            integration,
            _phantom: PhantomData,
        }
    }
}

/// Props for [`RouterBase`].
#[derive(Prop, Debug)]
pub struct RouterBaseProps<'a, R, F, I, G>
where
    R: Route + 'a,
    F: FnOnce(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    I: Integration,
    G: GenericNode,
{
    view: F,
    integration: I,
    route: R,
    #[builder(default, setter(skip))]
    _phantom: PhantomData<&'a G>,
}

impl<'a, R, F, I, G> RouterBaseProps<'a, R, F, I, G>
where
    R: Route + 'a,
    F: FnOnce(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    I: Integration,
    G: GenericNode,
{
    /// Create a new [`RouterBaseProps`].
    pub fn new(integration: I, view: F, route: R) -> Self {
        Self {
            view,
            integration,
            route,
            _phantom: PhantomData,
        }
    }
}

/// The sycamore router component. This component expects to be used inside a browser environment.
/// For server environments, see [`StaticRouter`].
#[component]
pub fn Router<'a, G: Html, R, F, I>(cx: Scope<'a>, props: RouterProps<'a, R, F, I, G>) -> View<G>
where
    R: Route + 'a,
    F: FnOnce(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    I: Integration + 'static,
{
    view! { cx,
        RouterBase {
            view: props.view,
            integration: props.integration,
            // The derive macro makes this the `#[not_found]` route (always present)
            route: R::default(),
        }
    }
}

/// A lower-level router component that takes an instance of your [`Route`] type. This is designed for `struct` [`Route`]s, which can be used to store
/// additional information along with routes.
///
/// This is a very specific use-case, and you probably actually want [`Router`]!
#[component]
pub fn RouterBase<'a, G: Html, R, F, I>(
    cx: Scope<'a>,
    props: RouterBaseProps<'a, R, F, I, G>,
) -> View<G>
where
    R: Route + 'a,
    F: FnOnce(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    I: Integration + 'static,
{
    let RouterBaseProps {
        view,
        integration,
        route,
        _phantom,
    } = props;
    let integration = Rc::new(integration);
    let base_pathname = base_pathname();

    PATHNAME.with(|pathname| {
        assert!(
            pathname.borrow().is_none(),
            "cannot have more than one Router component initialized"
        );
        // Get initial url from window.location.
        let path = integration.current_pathname();
        let path = path.strip_prefix(&base_pathname).unwrap_or(&path);
        *pathname.borrow_mut() = Some(create_rc_signal(path.to_string()));
    });
    let pathname = PATHNAME.with(|p| p.borrow().clone().unwrap_throw());

    // Set PATHNAME to None when the Router is destroyed.
    on_cleanup(cx, || {
        PATHNAME.with(|pathname| *pathname.borrow_mut() = None)
    });

    // Listen to popstate event.
    integration.on_popstate(Box::new({
        let integration = integration.clone();
        let pathname = pathname.clone();
        move || {
            let path = integration.current_pathname();
            let path = path.strip_prefix(&base_pathname).unwrap_or(&path);
            pathname.set(path.to_string());
        }
    }));
    let route_signal = create_memo(cx, move || route.match_path(&pathname.get()));
    // Delegate click events from child <a> tags.
    let view = view(cx, route_signal);
    if let Some(node) = view.as_node() {
        node.event(cx, "click", integration.click_handler());
    } else {
        // TODO: support fragments and dynamic nodes
        unimplemented!("support fragments and dynamic nodes for Router")
    }
    view
}

/// Props for [`StaticRouter`].
#[derive(Prop, Debug)]
pub struct StaticRouterProps<'a, R, F, G>
where
    R: Route + 'a,
    F: Fn(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    G: GenericNode,
{
    view: F,
    route: R,
    #[builder(default, setter(skip))]
    _phantom: PhantomData<&'a (R, G)>,
}

impl<'a, R, F, G> StaticRouterProps<'a, R, F, G>
where
    R: Route,
    F: Fn(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
    G: GenericNode,
{
    /// Create a new [`StaticRouterProps`].
    pub fn new(route: R, view: F) -> Self {
        Self {
            view,
            route,
            _phantom: PhantomData,
        }
    }
}

/// A router that only renders once with the given `route`.
///
/// This is useful for SSR where we want the HTML to be rendered instantly instead of waiting for
/// the route preload to finish loading.
#[component]
pub fn StaticRouter<'a, G: Html, R, F>(
    cx: Scope<'a>,
    props: StaticRouterProps<'a, R, F, G>,
) -> View<G>
where
    R: Route + 'static,
    F: Fn(Scope<'a>, &'a ReadSignal<R>) -> View<G> + 'a,
{
    let StaticRouterProps {
        view,
        route,
        _phantom,
    } = props;

    view(cx, create_signal(cx, route))
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
            "navigate can only be used with a Router"
        );

        let pathname = pathname.borrow().clone().unwrap_throw();
        let path = url.strip_prefix(&base_pathname()).unwrap_or(url);
        pathname.set(path.to_string());

        // Update History API.
        let window = web_sys::window().unwrap_throw();
        let history = window.history().unwrap_throw();
        history
            .push_state_with_url(&JsValue::UNDEFINED, "", Some(url))
            .unwrap_throw();
        window.scroll_to_with_x_and_y(0.0, 0.0);
    });
}

/// Navigates to the specified `url` without adding a new history entry. Instead, this replaces the
/// current location with the new `url`. The url should have the same origin as the app.
///
/// This is useful for imperatively navigating to an url when using an anchor tag (`<a>`) is not
/// possible/suitable (e.g. when submitting a form).
///
/// # Panics
/// This function will `panic!()` if a [`Router`] has not yet been created.
pub fn navigate_replace(url: &str) {
    PATHNAME.with(|pathname| {
        assert!(
            pathname.borrow().is_some(),
            "navigate_replace can only be used with a Router"
        );

        let pathname = pathname.borrow().clone().unwrap_throw();
        let path = url.strip_prefix(&base_pathname()).unwrap_or(url);
        pathname.set(path.to_string());

        // Update History API.
        let window = web_sys::window().unwrap_throw();
        let history = window.history().unwrap_throw();
        history
            .replace_state_with_url(&JsValue::UNDEFINED, "", Some(url))
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

        #[component]
        fn Comp<G: Html>(cx: Scope, path: String) -> View<G> {
            let route = Routes::match_route(
                // The user would never use this directly, so they'd never have to do this trick
                // It doesn't matter which variant we provide here, it just needs to conform to `&self` (designed for `struct`s, as in Perseus' router)
                &Routes::Home,
                &path
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>(),
            );

            view! { cx,
                StaticRouter {
                    route: route,
                    view: |cx, route: &ReadSignal<Routes>| {
                        match route.get().as_ref() {
                            Routes::Home => view! { cx,
                                "Home"
                            },
                            Routes::About => view! { cx,
                                "About"
                            },
                            Routes::NotFound => view! { cx,
                                "Not Found"
                            }
                        }
                    },
                }
            }
        }

        assert_eq!(
            sycamore::render_to_string(|cx| view! { cx, Comp("/".to_string()) }),
            "Home"
        );

        assert_eq!(
            sycamore::render_to_string(|cx| view! { cx, Comp("/about".to_string()) }),
            "About"
        );

        assert_eq!(
            sycamore::render_to_string(|cx| view! { cx, Comp("/404".to_string()) }),
            "Not Found"
        );
    }
}
