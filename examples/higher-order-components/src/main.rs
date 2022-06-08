#![allow(non_snake_case)]

use sycamore::prelude::*;

#[component]
fn MyComponent<G: Html>(cx: Scope, props: i32) -> View<G> {
    view! { cx,
        (props)
    }
}

fn higher_order_component<G: Html>(
    Comp: &dyn Fn(Scope, i32) -> View<G>,
) -> impl Fn(Scope) -> View<G> + '_ {
    move |cx| {
        view! { cx,
            div {
                Comp(42)
            }
        }
    }
}

fn main() {
    sycamore::render(|cx| {
        let EnhancedComponent = higher_order_component(&MyComponent);
        view! { cx,
            EnhancedComponent()
        }
    });
}
