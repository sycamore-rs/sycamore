use sycamore::component::Component;
use sycamore::prelude::*;

#[component(EnhancedComponent<G>)]
fn enhanced_component<C: Component<G, Props = ()>>() -> Template<G> {
    template! {
        div(class="enhanced-container") {
            p { "Enhanced container start" }
            C()
            p { "Enhanced container end" }
        }
    }
}

#[component(WrappedComponent<G>)]
fn wrapped_component() -> Template<G> {
    template! {
        p { "Wrapped component" }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { EnhancedComponent::<WrappedComponent::<_>>() });
}
