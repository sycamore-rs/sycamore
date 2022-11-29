use sycamore::prelude::*;

#[component(inline_props)]
pub fn Button<'a, G: Html>(cx: Scope<'a>, action: i32, updater: &'a dyn Fn(i32)) -> View<G> {
    let label = match action {
        -1 => "-",
        0 => "0",
        _ => "+",
    };
    let cls =
        "h-12 px-6 m-2 text-lg transition-colors duration-150 rounded-lg focus:shadow-outline";
    let cls = match action {
        -1 => format!("{} {}", cls, "text-red-100 bg-red-700 hover:bg-red-800"),
        0 => format!(
            "{} {}",
            cls, "text-yellow-100 bg-yellow-700 hover:bg-yellow-800"
        ),
        _ => format!(
            "{} {}",
            cls, "text-green-100 bg-green-700 hover:bg-green-800"
        ),
    };
    view! { cx,
        button(on:click=move |_| (*updater)(action), class=cls) {
            (label)
        }
    }
}
