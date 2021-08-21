use sycamore::component::Component;
use sycamore::prelude::*;

#[component(EnhancedComponent<G>)]
fn enhanced_component<C: Component<G, Props = i32>>() -> Template<G> {
    template! {
        div(class="enhanced-container") {
            p { "Enhanced container start" }
            C(42)
            p { "Enhanced container end" }
        }
    }
}

#[component(NumberDisplayer<G>)]
fn number_displayer(prop: i32) -> Template<G> {
    template! {
        p { "My number is: " (prop) }
    }
}

type EnhancedNumberDisplayer<G> = EnhancedComponent<G, NumberDisplayer<G>>;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { EnhancedNumberDisplayer() });
}
