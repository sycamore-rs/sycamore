#![allow(non_snake_case)]

use sycamore::prelude::*;

#[component]
fn MyComponent<G: Html>(ctx: ScopeRef, props: i32) -> View<G> {
    view! { ctx,
        (props)
    }
}

fn higher_order_component<G: Html>(
    Comp: &dyn Fn(ScopeRef, i32) -> View<G>,
) -> impl Fn(ScopeRef, ()) -> View<G> + '_ {
    move |ctx, _| {
        view! { ctx,
            div {
                Comp(42)
            }
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        let EnhancedComponent = higher_order_component(&MyComponent);
        view! { ctx,
            EnhancedComponent()
        }
    });
}
