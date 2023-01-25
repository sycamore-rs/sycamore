//! Portal API.

use sycamore_web::render::{get_render_env, RenderEnv};
use wasm_bindgen::prelude::*;

use crate::component::Children;
use crate::prelude::*;

/// Props for [`Portal`].
#[derive(Props, Debug)]
pub struct PortalProps<'a> {
    children: Children<'a, WebNode>,
    selector: &'a str,
}

/// A portal into another part of the DOM.
#[component]
pub fn Portal<'a>(cx: Scope<'a>, props: PortalProps<'a>) -> View {
    let PortalProps { children, selector } = props;

    if get_render_env(cx) == RenderEnv::Dom {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let container = document
            .query_selector(selector)
            .unwrap_throw()
            .expect_throw("could not find element matching selector");

        let children = children.call(cx).flatten();

        for child in &children {
            container.append_child(&child.to_web_sys()).unwrap_throw();
        }

        on_cleanup(cx, move || {
            for child in &children {
                container.remove_child(&child.to_web_sys()).unwrap_throw();
            }
        });
    }

    view! { cx, }
}
