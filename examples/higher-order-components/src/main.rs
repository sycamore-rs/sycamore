#![allow(non_snake_case)]

use sycamore::prelude::*;

#[derive(Props)]
pub struct MyComponentProps {
    value: i32,
}

#[component]
fn MyComponent<G: Html>(props: MyComponentProps) -> View<G> {
    view! {
        (props.value)
    }
}

fn higher_order_component<G: Html>(
    Comp: &dyn Fn(MyComponentProps) -> View<G>,
) -> impl Fn() -> View<G> + '_ {
    move || {
        view! {
            div {
                Comp(value=42)
            }
        }
    }
}

fn main() {
    sycamore::render(|| {
        let EnhancedComponent = higher_order_component(&MyComponent);
        view! {
            EnhancedComponent()
        }
    });
}
