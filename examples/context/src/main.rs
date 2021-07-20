use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::prelude::*;

struct State(i32);

#[component(Counter<G>)]
fn counter() -> Template<G> {
    let (counter, _) = use_context::<(ReadSignal<State>, WriteSignal<State>)>();

    template! {
        p(class="value") {
            "Value: "
            (counter.get().0)
        }
    }
}

#[component(Controls<G>)]
pub fn controls() -> Template<G> {
    let (counter, set_counter) = use_context::<(ReadSignal<State>, WriteSignal<State>)>();

    let increment = move |_| set_counter.set(State(counter.get().0 + 1));

    let reset = move |_| set_counter.set(State(0));

    template! {
        button(class="increment", on:click=increment) {
            "Increment"
        }
        button(class="reset", on:click=reset) {
            "Reset"
        }
    }
}

#[component(App<G>)]
fn app() -> Template<G> {
    let (counter, set_counter) = create_signal(0);

    create_effect(move || {
        log::info!("Counter value: {}", *counter.get());
    });

    template! {
        ContextProvider(ContextProviderProps {
            value: (counter, set_counter),
            children: move || {
                template! {
                    div {
                        "Counter demo"
                        Counter()
                        Controls()
                    }
                }
            }
        })
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { App() });
}
