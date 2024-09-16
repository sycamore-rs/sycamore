//! Definition for the [`NoSsr`] and [`NoHydrate`] components.

use sycamore_macro::{component, view};

use crate::*;

/// Component that is only renders its children on the client side.
///
/// This is useful when wrapping parts of your app that are not intended to be server-side
/// rendered, e.g. highly interactive components such as graphs, etc...
#[component(inline_props)]
pub fn NoSsr(children: Children) -> View {
    if is_ssr!() {
        view! { no-ssr() }
    } else {
        let marker = create_node_ref();
        let view = view! { no-ssr(r#ref=marker) };
        on_mount(move || {
            let marker = marker.get();
            let parent = marker.parent_node().unwrap();

            let children = children.call();
            for node in children.as_web_sys() {
                parent.insert_before(&node, Some(&marker)).unwrap();
            }
            parent.remove_child(&marker).unwrap();
        });
        view
    }
}

/// Components that do not need, or should not be hydrated on the client side.
///
/// This is useful when large parts of your app do not require client-side interactivity such as
/// static content.
///
/// However, this component will still be rendered on the client side if it is created after the
/// initial hydration phase is over, e.g. navigating to a new page with a `NoHydrate` component.
#[component(inline_props)]
pub fn NoHydrate(children: Children) -> View {
    if is_ssr!() {
        let is_hydrating = IS_HYDRATING.replace(false);
        let children = children.call();
        IS_HYDRATING.set(is_hydrating);
        children
    } else if IS_HYDRATING.get() {
        view! {}
    } else {
        children.call()
    }
}

/// Generate a script element for bootstrapping hydration.
///
/// In general, prefer using [`HydrationScript`] instead.
pub fn generate_hydration_script(mode: SsrMode) -> &'static str {
    match mode {
        SsrMode::Sync => "",
        SsrMode::Blocking => "window.__sycamore_ssr_mode='blocking'",
        SsrMode::Streaming => "window.__sycamore_ssr_mode='streaming'",
    }
}

/// Component that creates a script element for bootstrapping hydration. Should be rendered into
/// the `<head>` of the document.
///
/// This component is required if using SSR in blocking or streaming mode.
///
/// TODO: use this component to also capture and replay events. This requires synthetic event
/// delegation: <https://github.com/sycamore-rs/sycamore/issues/176>
#[component]
pub fn HydrationScript() -> View {
    is_ssr! {
        let mode = use_context::<SsrMode>();
        let script = generate_hydration_script(mode);
        view! {
            NoHydrate {
                script(dangerously_set_inner_html=script)
            }
        }
    }
    is_not_ssr! {
        view! {}
    }
}
