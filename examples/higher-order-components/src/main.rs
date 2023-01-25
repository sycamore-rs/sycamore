#![allow(non_snake_case)]

use sycamore::prelude::*;

#[derive(Props)]
pub struct MyComponentProps {
    value: i32,
}

#[component]
fn MyComponent(cx: Scope, props: MyComponentProps) -> View {
    view! { cx,
        (props.value)
    }
}

fn higher_order_component(
    Comp: &dyn Fn(Scope, MyComponentProps) -> View,
) -> impl Fn(Scope) -> View + '_ {
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
