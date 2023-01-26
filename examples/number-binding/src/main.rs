use sycamore::prelude::*;

#[component]
fn App(cx: Scope) -> View {
    let value = create_signal(cx, 10.0);

    view! { cx,
        p {
            (format!("{:.2}",value.get()))
        }

        input(_type="range", min="1", step="0.25", max="10", bind:value_as_number=value) {}
        br {}
        input(_type="number", min="1", step="0.25", max="10", bind:value_as_number=value) {}
    }
}

fn main() {
    sycamore::render(App);
}
