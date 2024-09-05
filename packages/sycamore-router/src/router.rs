use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
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

    fn click_handler(&self) -> Box<dyn Fn(web_sys::MouseEvent)>;
}

thread_local! {
    static PATHNAME: Cell<Option<Signal<String>>> = const { Cell::new(None) };
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
        window().location().pathname().unwrap_throw()
    }

    fn on_popstate(&self, f: Box<dyn FnMut()>) {
        let closure = Closure::wrap(f);
        window()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    }

    fn click_handler(&self) -> Box<dyn Fn(web_sys::MouseEvent)> {
        Box::new(|ev| {
            if let Some(a) = ev
                .target()
                .unwrap_throw()
                .unchecked_into::<Element>()
                .closest("a[href]")
                .unwrap_throw()
            {
                let location = window().location();

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
                    if location.hash().as_ref() != Ok(&hash) {
                        // Same origin, same path, different anchor. Use default browser behavior.
                    } else if location.pathname().as_ref() != Ok(&a_pathname) {
                        // Same origin, different path. Navigate to new page.
                        ev.prevent_default();
                        PATHNAME.with(|pathname| {
                            let pathname = pathname.get().unwrap_throw();
                            let path = a_pathname
                                .strip_prefix(&base_pathname())
                                .unwrap_or(&a_pathname);
                            pathname.set(path.to_string());

                            // Update History API.
                            let history = window().history().unwrap_throw();
                            history
                                .push_state_with_url(&JsValue::UNDEFINED, "", Some(&a_pathname))
                                .unwrap_throw();
                            window().scroll_to_with_x_and_y(0.0, 0.0);
                        });
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
    match document().query_selector("base[href]") {
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
#[derive(Props, Debug)]
pub struct RouterProps<R, F, I>
where
    R: Route + 'static,
    F: FnOnce(ReadSignal<R>) -> View + 'static,
    I: Integration,
{
    view: F,
    integration: I,
    #[prop(default, setter(skip))]
    _phantom: PhantomData<R>,
}

impl<R, F, I> RouterProps<R, F, I>
where
    R: Route + 'static,
    F: FnOnce(ReadSignal<R>) -> View + 'static,
    I: Integration,
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
#[derive(Props, Debug)]
pub struct RouterBaseProps<R, F, I>
where
    R: Route + 'static,
    F: FnOnce(ReadSignal<R>) -> View + 'static,
    I: Integration,
{
    view: F,
    integration: I,
    route: R,
}

impl<R, F, I> RouterBaseProps<R, F, I>
where
    R: Route + 'static,
    F: FnOnce(ReadSignal<R>) -> View + 'static,
    I: Integration,
{
    /// Create a new [`RouterBaseProps`].
    pub fn new(integration: I, view: F, route: R) -> Self {
        Self {
            view,
            integration,
            route,
        }
    }
}

/// The sycamore router component. This component expects to be used inside a browser environment.
/// For server environments, see [`StaticRouter`].
#[component]
pub fn Router<R, F, I>(props: RouterProps<R, F, I>) -> View
where
    R: Route + 'static,
    F: FnOnce(ReadSignal<R>) -> View + 'static,
    I: Integration + 'static,
{
    view! {
        RouterBase(
            view=props.view,
            integration=props.integration,
            // The derive macro makes this the `#[not_found]` route (always present)
            route=R::default(),
        )
    }
}

/// A lower-level router component that takes an instance of your [`Route`] type. This is designed
/// for `struct` [`Route`]s, which can be used to store additional information along with routes.
///
/// This is a very specific use-case, and you probably actually want [`Router`]!
#[component]
pub fn RouterBase<R, F, I>(props: RouterBaseProps<R, F, I>) -> View
where
    R: Route + 'static,
    F: FnOnce(ReadSignal<R>) -> View + 'static,
    I: Integration + 'static,
{
    let RouterBaseProps {
        view,
        integration,
        route,
    } = props;
    let integration = Rc::new(integration);
    let base_pathname = base_pathname();

    PATHNAME.with(|pathname| {
        assert!(
            pathname.get().is_none(),
            "cannot have more than one Router component initialized"
        );
        // Get initial url from window.location.
        let path = integration.current_pathname();
        let path = path.strip_prefix(&base_pathname).unwrap_or(&path);
        pathname.set(Some(create_signal(path.to_string())));
    });
    let pathname = PATHNAME.with(|p| p.get().unwrap_throw());

    // Set PATHNAME to None when the Router is destroyed.
    on_cleanup(|| PATHNAME.with(|pathname| pathname.set(None)));

    // Listen to popstate event.
    integration.on_popstate(Box::new({
        let integration = integration.clone();
        move || {
            let path = integration.current_pathname();
            let path = path.strip_prefix(&base_pathname).unwrap_or(&path);
            if pathname.with(|pathname| pathname != path) {
                pathname.set(path.to_string());
            }
        }
    }));
    let route_signal = create_memo(move || pathname.with(|pathname| route.match_path(pathname)));
    let view = view(route_signal);
    let nodes = view.as_web_sys();
    on_mount(move || {
        for node in nodes {
            let handler: Closure<dyn FnMut(web_sys::MouseEvent)> =
                Closure::new(integration.click_handler());
            node.add_event_listener_with_callback("click", handler.into_js_value().unchecked_ref())
                .unwrap(); // TODO: manage in scope
        }
        // TODO: this does not work for dynamic views
    });
    view
}

/// Props for [`StaticRouter`].
#[derive(Props, Debug)]
pub struct StaticRouterProps<R, F>
where
    R: Route + 'static,
    F: Fn(ReadSignal<R>) -> View + 'static,
{
    view: F,
    route: R,
}

impl<R, F> StaticRouterProps<R, F>
where
    R: Route + 'static,
    F: Fn(ReadSignal<R>) -> View + 'static,
{
    /// Create a new [`StaticRouterProps`].
    pub fn new(route: R, view: F) -> Self {
        Self { view, route }
    }
}

/// A router that only renders once with the given `route`.
///
/// This is useful for SSR where we want the HTML to be rendered instantly instead of waiting for
/// the route preload to finish loading.
#[component]
pub fn StaticRouter<R, F>(props: StaticRouterProps<R, F>) -> View
where
    R: Route + 'static,
    F: Fn(ReadSignal<R>) -> View + 'static,
{
    view! {
        StaticRouterBase(view=props.view, route=props.route)
    }
}

/// Implementation detail for [`StaticRouter`]. The extra component is needed to make sure hydration
/// keys are consistent.
#[component]
fn StaticRouterBase<R, F>(props: StaticRouterProps<R, F>) -> View
where
    R: Route + 'static,
    F: Fn(ReadSignal<R>) -> View + 'static,
{
    let StaticRouterProps { view, route } = props;

    view(*create_signal(route))
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
            pathname.get().is_some(),
            "navigate can only be used with a Router"
        );

        let pathname = pathname.get().unwrap_throw();
        let path = url.strip_prefix(&base_pathname()).unwrap_or(url);
        pathname.set(path.to_string());

        // Update History API.
        let history = window().history().unwrap_throw();
        history
            .push_state_with_url(&JsValue::UNDEFINED, "", Some(url))
            .unwrap_throw();
        window().scroll_to_with_x_and_y(0.0, 0.0);
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
            pathname.get().is_some(),
            "navigate_replace can only be used with a Router"
        );

        let pathname = pathname.get().unwrap_throw();
        let path = url.strip_prefix(&base_pathname()).unwrap_or(url);
        pathname.set(path.to_string());

        // Update History API.
        let history = window().history().unwrap_throw();
        history
            .replace_state_with_url(&JsValue::UNDEFINED, "", Some(url))
            .unwrap_throw();
        window().scroll_to_with_x_and_y(0.0, 0.0);
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
        #[derive(Route, Clone, Copy)]
        enum Routes {
            #[to("/")]
            Home,
            #[to("/about")]
            About,
            #[not_found]
            NotFound,
        }

        #[component(inline_props)]
        fn Comp(path: String) -> View {
            let route = Routes::match_route(
                // The user would never use this directly, so they'd never have to do this trick
                // It doesn't matter which variant we provide here, it just needs to conform to
                // `&self` (designed for `struct`s, as in Perseus' router)
                &Routes::Home,
                &path
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>(),
            );

            view! {
                StaticRouter(
                    route=route,
                    view=|route: ReadSignal<Routes>| {
                        match route.get() {
                            Routes::Home => view! {
                                "Home"
                            },
                            Routes::About => view! {
                                "About"
                            },
                            Routes::NotFound => view! {
                                "Not Found"
                            }
                        }
                    },
                )
            }
        }

        assert_eq!(
            sycamore::render_to_string(|| view! { Comp(path="/".to_string()) }),
            "Home"
        );

        assert_eq!(
            sycamore::render_to_string(|| view! { Comp(path="/about".to_string()) }),
            "About"
        );

        assert_eq!(
            sycamore::render_to_string(|| view! { Comp(path="/404".to_string()) }),
            "Not Found"
        );
    }
}
