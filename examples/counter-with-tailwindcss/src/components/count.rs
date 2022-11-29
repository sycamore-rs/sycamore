use sycamore::prelude::*;

#[component(inline_props)]
pub fn Count<'a, G: Html>(cx: Scope<'a>, value: &'a ReadSignal<i32>) -> View<G> {
    view! { cx,
        h1(class="text-3xl") {
            "Count: " (*value.get())
        }
    }
}
