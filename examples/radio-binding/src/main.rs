use sycamore::prelude::*;

#[component]
fn App() -> View {
    // The signal holds the `value` of the selected radio in the group.
    let selected = create_signal("cat".to_string());

    view! {
        p { "You picked: " (selected.get_clone()) }

        label { input(r#type="radio", name="pet", value="cat", bind:group=selected) " Cat" }
        label { input(r#type="radio", name="pet", value="dog", bind:group=selected) " Dog" }
        label { input(r#type="radio", name="pet", value="bird", bind:group=selected) " Bird" }

        button(on:click=move |_| selected.set("dog".to_string())) { "Pick dog" }
    }
}

fn main() {
    sycamore::render(App);
}
