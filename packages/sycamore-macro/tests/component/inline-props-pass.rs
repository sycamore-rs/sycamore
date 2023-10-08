use sycamore::prelude::{component, view, Html, Signal, View};

#[component(inline_props)]
fn NoProps<G: Html>() -> View<G> {
    view! {}
}

#[component(inline_props)]
fn SimpleComponent<G: Html>(my_number: u32) -> View<G> {
    view! {
        (my_number)
    }
}

#[component(inline_props)]
fn MultiProps<G: Html>(my_number: u32, my_string: String) -> View<G> {
    view! {
        (my_number)
        (my_string)
    }
}

#[component(inline_props)]
fn PropsWithGenericLifetime<G: Html>(data: Signal<u32>) -> View<G> {
    view! {
        (data.get())
    }
}

#[component(inline_props)]
fn UnusedGeneric<G: Html, T>() -> View<G> {
    view! {}
}

#[component(inline_props)]
fn PropsWithGenericTypes<G: Html, T: std::fmt::Display + 'static>(foo: T) -> View<G> {
    view! {
        (foo.to_string())
    }
}

fn main() {}
