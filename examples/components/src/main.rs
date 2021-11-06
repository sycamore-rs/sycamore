use sycamore::prelude::*;

#[component(MyComponent<G>)]
fn my_component(num: StateHandle<i32>) -> View<G> {
    view! {
        div(class="my-component") {
            "My component"
            p {
                "Value: "
                (num.get())
            }
        }
    }
}

#[component(App<G>)]
fn app() -> View<G> {
    let state = Signal::new(1);

    let increment = cloned!((state) => move |_| {
        state.set(*state.get() + 1);
    });

    view! {
        div {
            h1 {
                "Component demo"
            }

            MyComponent(state.handle())
            MyComponent(state.handle())

            button(on:click=increment) {
                "Increment"
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
