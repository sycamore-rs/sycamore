#![allow(non_snake_case)]

use sycamore::prelude::*;

#[component]
fn MyComponent(value: i32) -> View {
    view! {
        (value)
    }
}

fn higher_order_component(Comp: &dyn Fn(MyComponentProps) -> View) -> impl Fn() -> View + '_ {
    move || {
        view! {
            div { Comp(value=42) }
        }
    }
}

fn main() {
    sycamore::render(|| {
        let EnhancedComponent = higher_order_component(&MyComponent);
        view! {
            EnhancedComponent {}
        }
    });
}
