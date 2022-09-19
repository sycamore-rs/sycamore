use sycamore::prelude::{component, view, Html, Scope, Signal, View};

#[component(inline_props)]
fn NoProps<G: Html>(cx: Scope) -> View<G> {
    view! { cx, }
}

#[component(inline_props)]
fn SimpleComponent<G: Html>(cx: Scope, my_number: u32) -> View<G> {
    view! { cx,
        (my_number)
    }
}

#[component(inline_props)]
fn MultiProps<G: Html>(cx: Scope, my_number: u32, my_string: String) -> View<G> {
    view! { cx,
        (my_number)
        (my_string)
    }
}

#[component(inline_props)]
fn PropsWithGenericLifetime<'a, G: Html>(cx: Scope<'a>, data: &'a Signal<u32>) -> View<G> {
    view! { cx,
        (data.get())
    }
}

#[component(inline_props)]
fn UnusedGeneric<G: Html, T>(cx: Scope) -> View<G> {
    view! { cx, }
}

#[component(inline_props)]
fn PropsWithGenericTypes<'a, G: Html, T: std::fmt::Display + 'a>(cx: Scope<'a>, foo: T) -> View<G> {
    view! { cx,
        (foo.to_string())
    }
}

fn main() {}
