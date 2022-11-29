use sycamore::prelude::*;

#[component(inline_props)]
pub fn Square<'a, G: Html>(cx: Scope<'a>, value: &'a ReadSignal<i32>) -> View<G> {
    let negative = create_selector(cx, || *value.get() < 0);

    view! { cx,
        h1(class="text-3xl") {
            "Square: " ((*value.get()) * (*value.get()))
        }
        (if *negative.get() {
            view! { cx,
                h3(class="text-xl text-red-700") {
                    "Warning: base is negative"
                }
            }
        } else {
            view! { cx, }
        })
    }
}
