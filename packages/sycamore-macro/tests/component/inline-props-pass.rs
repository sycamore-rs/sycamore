use sycamore::prelude::{component, view, Signal, View};

#[component(inline_props)]
fn NoProps() -> View {
    view! {}
}

#[component(inline_props)]
fn SimpleComponent(my_number: u32) -> View {
    view! {
        (my_number)
    }
}

#[component(inline_props)]
fn MultiProps(my_number: u32, my_string: String) -> View {
    view! {
        (my_number)
        (my_string)
    }
}

#[component(inline_props)]
fn PropsWithGenericLifetime(data: Signal<u32>) -> View {
    view! {
        (data.get())
    }
}

#[component(inline_props)]
fn UnusedGeneric<T>() -> View {
    view! {}
}

#[component(inline_props)]
fn PropsWithGenericTypes<T: std::fmt::Display + 'static>(foo: T) -> View {
    view! {
        (foo.to_string())
    }
}

fn main() {}
