#![allow(non_snake_case)]

use sycamore::prelude::*;

#[derive(Props)]
pub struct MyComponentProps {
    value: i32,
}

#[component]
fn MyComponent(props: MyComponentProps) -> View {
    view! {
        (props.value)
    }
}

fn higher_order_component(Comp: &dyn Fn(MyComponentProps) -> View) -> impl Fn() -> View + '_ {
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
