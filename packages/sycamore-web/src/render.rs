//! Render your app!

use sycamore_core::render::insert;
use sycamore_core::view::View;
use sycamore_reactive::{create_scope, provide_context, use_context, Scope};
use wasm_bindgen::UnwrapThrowExt;

use crate::web_node::WebNode;

/// Keeps track of which environment we are currently in, either the web-browser's DOM or
/// server-side. This should be inserted as a context into the root scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderEnv {
    /// The web-browser's DOM.
    Dom,
    /// Server-side rendering virtual DOM.
    Ssr,
}

/// Gets the global rendering environment.
///
/// # Panics
///
/// This will panic if the environment is not set. Note that the environment is set automatically to
/// the correct value by functions such as [`render`] and [`render_to_string`].
pub fn get_render_env(cx: Scope) -> RenderEnv {
    *use_context(cx)
}

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
#[cfg(feature = "dom")]
pub fn render(f: impl FnOnce(Scope) -> View<WebNode>) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    render_to(document.body().unwrap_throw(), f);
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
#[cfg(feature = "dom")]
pub fn render_to(root: web_sys::HtmlElement, f: impl FnOnce(Scope) -> View<WebNode>) {
    // Do not call the scope dispose callback, essentially leaking the scope for the lifetime of
    // the app.
    let _ = create_scope(|cx| render_to_with_scope(cx, root, f));
}

/// Same as [`render_to`] but with a pre-created scope.
#[cfg(feature = "dom")]
pub fn render_to_with_scope(
    cx: Scope,
    root: web_sys::HtmlElement,
    f: impl FnOnce(Scope) -> View<WebNode>,
) {
    // Provide the environment context.
    provide_context(cx, RenderEnv::Dom);

    let root = WebNode::from_web_sys(root.into());
    insert(cx, &root, f(cx), None, None, true);
}

/// Render a [`View`] into a pre-existing DOM node.
/// Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
#[cfg(all(feature = "dom", feature = "hydrate"))]
pub fn hydrate(f: impl FnOnce(Scope) -> View<WebNode>) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    hydrate_to(document.body().unwrap_throw(), f);
}

/// Render a [`View`] under a `parent` node with pre-existing DOM nodes.
/// For rendering under the `<body>` tag, use [`hydrate`] instead.
#[cfg(all(feature = "dom", feature = "hydrate"))]
pub fn hydrate_to(root: web_sys::HtmlElement, f: impl FnOnce(Scope) -> View<WebNode>) {
    // Do not call the scope dispose callback, essentially leaking the scope for the lifetime of
    // the app.
    let _ = create_scope(|cx| hydrate_to_with_scope(cx, root, f));
}

/// Same as [`hydrate_to`] but with a pre-created scope.
#[cfg(all(feature = "dom", feature = "hydrate"))]
pub fn hydrate_to_with_scope(
    cx: Scope,
    root: web_sys::HtmlElement,
    f: impl FnOnce(Scope) -> View<WebNode>,
) {
    use crate::hydrate::{HydrationCtx, HydrationState};

    // Provide the environment context.
    provide_context(cx, RenderEnv::Dom);
    provide_context(cx, HydrationState::new());
    provide_context(cx, HydrationCtx::new_from_root(root.clone().into()));

    let root = WebNode::from_web_sys(root.into());
    // Get children from parent into a View to set as the initial node value.
    let mut children = Vec::new();
    let child_nodes = root.to_web_sys().child_nodes();
    for i in 0..child_nodes.length() {
        children.push(child_nodes.get(i).unwrap());
    }
    let children = children
        .into_iter()
        .map(|x| View::new_node(WebNode::from_web_sys(x)))
        .collect::<Vec<_>>();
    let children = View::new_fragment(children);

    insert(cx, &root, f(cx), Some(children), None, true);
}

/// Render a [`View`] into a static [`String`]. Useful
/// for rendering to a string on the server side.
#[must_use]
#[cfg(feature = "ssr")]
pub fn render_to_string(f: impl FnOnce(Scope) -> View<WebNode>) -> String {
    use sycamore_reactive::create_scope_immediate;

    let mut ret = String::new();
    create_scope_immediate(|cx| ret = render_to_string_with_scope(cx, f));
    ret
}

/// Same as [`render_to_string`] but with a pre-created scope.
#[must_use]
#[cfg(feature = "ssr")]
pub fn render_to_string_with_scope(
    cx: Scope,
    f: impl FnOnce(Scope<'_>) -> View<WebNode>,
) -> String {
    use crate::hydrate::HydrationState;
    use crate::web_node::ssr::WriteToString;

    // Provide the environment context.
    provide_context(cx, RenderEnv::Ssr);
    provide_context(cx, HydrationState::new());

    let mut buf = String::new();
    let view = f(cx);

    for node in view.flatten() {
        node.as_ssr_node()
            .expect("expected SSR node")
            .write_to_string(&mut buf);
    }

    buf
}
