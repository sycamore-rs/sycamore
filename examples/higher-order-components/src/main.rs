#![allow(non_snake_case)]

use sycamore::prelude::*;

#[derive(Props)]
pub struct MyComponentProps {
    value: i32,
}

#[component]
fn MyComponent<G: Html>(cx: Scope, props: MyComponentProps) -> View<G> {
    view! { cx,
        (props.value)
    }
}

fn higher_order_component<G: Html>(
    Comp: &dyn Fn(Scope, MyComponentProps) -> View<G>,
) -> impl Fn(Scope) -> View<G> + '_ {
    move |cx| {
        view! { cx,
            div {
                Comp(value=42)
            }
        }
    }
}

fn main() {
    sycamore::render(|cx| {
        let EnhancedComponent = higher_order_component(&MyComponent);
        view! { cx,
            EnhancedComponent {}
        }
    });
}
