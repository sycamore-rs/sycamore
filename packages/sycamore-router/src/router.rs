use std::cell::RefCell;
use std::rc::Rc;

use sycamore::prelude::*;

use crate::Route;

#[component(StaticRouter<G>)]
pub fn static_router<R: Route>(
    (pathname, render): (String, impl Fn(R) -> Template<G> + 'static),
) -> Template<G> {
    let path = pathname
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let route = R::match_route(&path);
    render(route)
}

thread_local! {
    static PATHNAME: RefCell<Option<Signal<String>>> = RefCell::new(None);
}

#[component(Link<G>)]
pub fn link(to: impl ToString) -> Template<G> {
    let href = Rc::new(to.to_string());
    let handle_click = cloned!((href) => move |ev: web_sys::Event| {
        ev.prevent_default();

        PATHNAME.with(|pathname| {
            if pathname.borrow().is_none() {
                panic!("Link can only be used inside a BrowserRouter");
            }
            pathname.borrow().as_ref().unwrap().set(href.to_string());
        });
    });
    template! {
        a(href=href, on:click=handle_click)
    }
}

#[component(BrowserRouter<G>)]
pub fn browser_router<R: Route>(render: impl Fn(R) -> Template<G> + 'static) -> Template<G> {
    PATHNAME.with(|pathname| {
        assert!(pathname.borrow().is_none());
        // Get initial url from window.location.
        *pathname.borrow_mut() = Some(Signal::new(
            web_sys::window().unwrap().location().pathname().unwrap(),
        ));
    });
    let pathname = PATHNAME.with(|p| p.borrow().clone().unwrap());
    let path = create_memo(move || {
        pathname
            .get()
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    });

    Template::new_lazy(move || {
        let route = R::match_route(
            path.get()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        untrack(|| render(route))
    })
}
