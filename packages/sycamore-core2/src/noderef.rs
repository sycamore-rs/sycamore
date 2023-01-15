//! References to nodes in views.
//!
//! This allows imperatively accessing the node.
//!
//! You can create a [`NodeRef`] by using [`create_node_ref`].

use std::any::Any;
use std::fmt;

use sycamore_reactive::*;

use crate::generic_node::GenericNode;

/// A reference to a [`GenericNode`].
/// This allows imperatively accessing the node.
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// #[component]
/// fn Component<G: Html>(cx: Scope) -> View<G> {
///     let div_ref = create_node_ref(cx);
///     view! { cx,
///         div(ref=div_ref)
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct NodeRef<G: GenericNode>(RcSignal<Option<G>>);

impl<G: GenericNode + Any> NodeRef<G> {
    /// Creates an empty node ref.
    ///
    /// Generally, it is preferable to use [`create_node_ref`] instead.
    /// Unlike [`create_node_ref`], this creates a node ref that is not behind a reference. This
    /// makes it harder to pass around but can be desireable in certain cases.
    pub fn new() -> Self {
        let signal = create_rc_signal(None);
        Self(signal)
    }

    /// Gets the raw node stored inside the node ref.
    ///
    /// This attempts to cast the node to the specified type.
    ///
    /// # Example
    /// Node refs are generally meant to be accessed in callbacks or in `on_mount`. Accessing the
    /// node ref directly in the body of the component will panic because the node ref has not yet
    /// been set.
    ///
    /// ```
    /// # use sycamore::prelude::*;
    /// # fn Component<G: Html>(cx: Scope) -> View<G> {
    /// let div_ref = create_node_ref(cx);
    /// on_mount(cx, || {
    ///     let node = div_ref.get::<DomNode>();
    /// });
    /// view! { cx,
    ///     div(ref=div_ref)
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    /// Panics if the node ref is not set yet or is the wrong type.
    ///
    /// For a non panicking version, see [`NodeRef::try_get`].
    #[track_caller]
    pub fn get<T: GenericNode>(&self) -> T {
        self.try_get().expect("NodeRef is not set")
    }

    /// Tries to get the T stored inside the node ref or `None` if it is not yet set or
    /// the wrong type.
    ///
    /// For a panicking version, see [`NodeRef::get`].
    pub fn try_get<T: GenericNode>(&self) -> Option<T> {
        if let Some(g) = self.0.get().as_ref() {
            (g as &dyn Any).downcast_ref().cloned()
        } else {
            None
        }
    }

    /// Gets the raw node stored inside the node ref.
    ///
    /// # Panics
    /// Panics if the node ref is not set yet.
    ///
    /// For a non panicking version, see [`NodeRef::try_get_raw`].
    #[track_caller]
    pub fn get_raw(&self) -> G {
        self.try_get().expect("NodeRef is not set")
    }

    /// Tries to get the raw node stored inside the node ref or `None` if it is
    /// not yet set.
    ///
    /// For a panicking version, see [`NodeRef::get`].
    pub fn try_get_raw(&self) -> Option<G> {
        self.0.get().as_ref().clone()
    }

    /// Sets the node ref with the specified node.
    ///
    /// This method should be rarely used. Instead, use the `ref=` syntax in the `view!` macro to
    /// set the node.
    ///
    /// # Example
    /// Setting the node using the `ref=` syntax:
    /// ```
    /// # use sycamore::prelude::*;
    /// #[component]
    /// fn Component<G: Html>(cx: Scope) -> View<G> {
    ///     let div_ref = create_node_ref(cx);
    ///     view! { cx,
    ///         div(ref=div_ref) // This assigns the node ref a value.
    ///     }
    /// }
    /// ```
    ///
    /// Calling `.set(...)` imperatively:
    /// ```
    /// # use sycamore::prelude::*;
    /// # use sycamore::web::html;
    /// #[component]
    /// fn Component<G: Html>(cx: Scope) -> View<G> {
    ///     let div = G::element::<html::div>();
    ///     let div_ref = create_node_ref(cx);
    ///     div_ref.set(div.clone());
    ///     View::new_node(div)
    /// }
    /// ```
    pub fn set(&self, node: G) {
        self.0.set(Some(node));
    }
}

impl<G: GenericNode> Default for NodeRef<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: GenericNode> fmt::Debug for NodeRef<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeRef").field(&self.0.get()).finish()
    }
}

/* Hook implementation */

/// Create a new [`NodeRef`] on the current [`Scope`].
///
/// The node ref does not point to anything until it is set, either by assigning it to a node in the
/// view or by explicitly calling [`NodeRef::set`].
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # fn Component<G: Html>(cx: Scope) -> View<G> {
/// let node_ref: &NodeRef<G> = create_node_ref(cx);
/// # view! { cx, }
/// # }
/// ```
pub fn create_node_ref<G: GenericNode>(cx: Scope<'_>) -> &NodeRef<G> {
    create_ref(cx, NodeRef::new())
}
