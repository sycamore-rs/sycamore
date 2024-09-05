//! Implementation of rendering backend.

use crate::*;

//cfg_not_ssr_item!(
mod dom;
//);
cfg_ssr_item!(
    mod ssr;
);

#[cfg_not_ssr]
pub use dom::*;
#[cfg_ssr]
pub use ssr::*;

/// A trait that should be implemented for anything that represents an HTML node.
pub trait ViewHtmlNode {
    /// Create a new HTML element.
    fn create_element(tag: Cow<'static, str>) -> Self;
    /// Create a new HTML element with a XML namespace.
    fn create_element_ns(namespace: &'static str, tag: Cow<'static, str>) -> Self;
    /// Create a new HTML text node.
    fn create_text_node(text: Cow<'static, str>) -> Self;
    /// Create a new HTML marker (comment) node.
    fn create_marker_node() -> Self;

    /// Set an HTML attribute.
    fn set_attribute(&mut self, name: &'static str, value: MaybeDynString);
    /// Set a boolean HTML attribute.
    fn set_bool_attribute(&mut self, name: &'static str, value: MaybeDynBool);
    /// Set a JS property on an element.
    fn set_property(&mut self, name: &'static str, value: MaybeDynJsValue);
    /// Set an event handler on an element.
    fn set_event_handler(
        &mut self,
        name: &'static str,
        handler: impl FnMut(web_sys::Event) + 'static,
    );
    /// Set the inner HTML value of an element.
    fn set_inner_html(&mut self, inner_html: Cow<'static, str>);

    /// Return the raw web-sys node.
    fn as_web_sys(&self) -> &web_sys::Node;
    /// Wrap a raw web-sys node.
    fn from_web_sys(node: web_sys::Node) -> Self;
}

/// A trait for unwrapping a type into an `HtmlNode`.
pub trait IntoHtmlNode {
    fn into_html_node(self) -> HtmlNode;
    fn as_html_node(&self) -> &HtmlNode;
    fn as_html_node_mut(&mut self) -> &mut HtmlNode;
}

/// A trait that represents an attribute that can be set. This is not "attribute" in the HTML spec
/// sense. It can also represent JS properties (and possibly more ...) that can be set on an HTML
/// element.
pub trait AttributeValue {
    fn set_self(self, el: &mut HtmlNode, name: &'static str);
}

impl AttributeValue for MaybeDynString {
    fn set_self(self, el: &mut HtmlNode, name: &'static str) {
        el.set_attribute(name, self);
    }
}

impl AttributeValue for MaybeDynBool {
    fn set_self(self, el: &mut HtmlNode, name: &'static str) {
        el.set_bool_attribute(name, self);
    }
}

impl AttributeValue for MaybeDynJsValue {
    fn set_self(self, el: &mut HtmlNode, name: &'static str) {
        el.set_property(name, self);
    }
}

/// Represents a value that can be either static or dynamic.
pub enum MaybeDyn<T> {
    Static(T),
    Dynamic(Box<dyn FnMut() -> T>),
}

impl<T> MaybeDyn<T> {
    /// Evaluate the value by consuming itself.
    fn evaluate(self) -> T {
        match self {
            Self::Static(value) => value,
            Self::Dynamic(mut f) => f(),
        }
    }
}

impl<T, F: FnMut() -> U + 'static, U: Into<T>> From<F> for MaybeDyn<T> {
    fn from(mut f: F) -> Self {
        Self::Dynamic(Box::new(move || f().into()))
    }
}

/// A possibly dynamic string value.
pub type MaybeDynString = MaybeDyn<Cow<'static, str>>;

/// A possibly dynamic boolean value.
pub type MaybeDynBool = MaybeDyn<bool>;

/// A possibly dynamic [`JsValue`].
pub type MaybeDynJsValue = MaybeDyn<JsValue>;

macro_rules! impl_from_maybe_dyn {
    ($struct:ty => $($ty:ty),*) => {
        $(
            impl From<$ty> for $struct {
                fn from(value: $ty) -> Self {
                    Self::Static(value.into())
                }
            }
        )*
    };
}

impl_from_maybe_dyn!(MaybeDynString => &'static str, String, Cow<'static, str>);

impl_from_maybe_dyn!(MaybeDynBool => bool);

impl_from_maybe_dyn!(
    MaybeDynJsValue =>
    JsValue,
    String,
    bool,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64
);

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(view: impl FnOnce() -> View) {
    render_to(view, &document().body().unwrap());
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
pub fn render_to(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = create_root(|| render_in_scope(view, parent));
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
///
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// It is, however, preferable to have a single call to [`render`] or [`render_to`] at the top
/// level of your app long-term. For rendering a view that will never be unmounted from the dom,
/// use [`render_to`] instead. For rendering under the `<body>` tag, use [`render`] instead.
///
/// It is expected that this function will be called inside a reactive root, usually created using
/// [`create_root`].
pub fn render_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    if is_ssr!() {
        panic!("`render_in_scope` is not available in SSR mode");
    } else {
        let nodes = view().nodes;
        for node in nodes {
            parent.append_child(node.as_web_sys()).unwrap();
        }
    }
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration).
///
/// Alias for [`hydrate_to`] with `parent` being the `<body>` tag.
/// For rendering without hydration, use [`render`](super::render) instead.
pub fn hydrate(view: impl FnOnce() -> View) {
    hydrate_to(view, &document().body().unwrap());
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration).
///
/// For rendering under the `<body>` tag, use [`hydrate`] instead.
/// For rendering without hydration, use [`render`](super::render) instead.
pub fn hydrate_to(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = create_root(|| hydrate_in_scope(view, parent));
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
///
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// It is expected that this function will be called inside a reactive root, usually created using
/// [`create_root`].
pub fn hydrate_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    cfg_ssr_item! {{
        let _ = view;
        let _ = parent;
        panic!("`hydrate_in_scope` is not available in SSR mode");
    }}
    cfg_not_ssr_item! {{
        provide_context(HydrationRegistry::new());
        // Get all nodes with `data-hk` attribute.
        let mut existing_nodes = parent
            .unchecked_ref::<web_sys::Element>()
            .query_selector_all("[data-hk]")
            .unwrap();
        let len = existing_nodes.length();
        let mut temp = vec![None; len as usize];
        for i in 0..len {
            let node = existing_nodes.get(i).unwrap();
            let hk = node.unchecked_ref::<web_sys::Element>().get_attribute("data-hk").unwrap();
            let hk = hk.parse::<usize>().unwrap();
            temp[hk] = Some(node);
        }

        // Now assign every element in temp to HYDRATION_NODES
        HYDRATE_NODES.with(|nodes| {
            *nodes.borrow_mut() = temp.into_iter().map(|x| HtmlNode::from_web_sys(x.unwrap())).rev().collect();
        });

        IS_HYDRATING.set(true);
        view();
        IS_HYDRATING.set(false);
    }}
}

/// Render a [`View`] into a static [`String`]. Useful for rendering to a string on the server side.
#[must_use]
pub fn render_to_string(view: impl FnOnce() -> View) -> String {
    cfg_not_ssr_item! {{
        let _ = view;
        panic!("`render_to_string` only available in SSR mode");
    }}
    cfg_ssr_item! {{
        use std::cell::LazyCell;

        thread_local! {
            /// Use a static variable here so that we can reuse the same root for multiple calls to
            /// this function.
            static SSR_ROOT: LazyCell<RootHandle> = LazyCell::new(|| create_root(|| {}));
        }
        let mut buf = String::new();
        SSR_ROOT.with(|root| {
            root.dispose();
            root.run_in(|| {
                let handle = create_child_scope(|| {
                    // We run this in a new scope so that we can dispose everything after we render it.
                    provide_context(HydrationRegistry::new());

                    let view = view();
                    for node in view.nodes {
                        ssr::render_recursive(node, &mut buf);
                    }
                });
                handle.dispose();
            });
        });
        buf
    }}
}
