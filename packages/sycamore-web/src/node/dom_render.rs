use super::*;

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
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
#[cfg(feature = "hydrate")]
pub fn hydrate(view: impl FnOnce() -> View) {
    hydrate_to(view, &document().body().unwrap());
}

/// Render a [`View`] under a `parent` node by reusing existing nodes (client side
/// hydration).
///
/// For rendering under the `<body>` tag, use [`hydrate`] instead.
/// For rendering without hydration, use [`render`](super::render) instead.
#[cfg(feature = "hydrate")]
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
#[cfg(feature = "hydrate")]
pub fn hydrate_in_scope(view: impl FnOnce() -> View, parent: &web_sys::Node) {
    is_ssr! {
        let _ = view;
        let _ = parent;
        panic!("`hydrate_in_scope` is not available in SSR mode");
    }
    is_not_ssr! {
        provide_context(HydrationRegistry::new());
        // Get all nodes with `data-hk` attribute.
        let existing_nodes = parent
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
        let nodes = view().nodes;
        // We need to append `nodes` to the `parent` so that the top level nodes also get properly
        // hydrated.
        let mut parent = HydrateNode::from_web_sys(parent.clone());
        for node in nodes {
            parent.append_child(node);
        }
        IS_HYDRATING.set(false);
    }
}
