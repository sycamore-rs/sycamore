#![allow(unused_parens)]

use sycamore::prelude::{Props, Signal, View, component, view};

#[component]
fn NoProps() -> View {
    view! {}
}

#[component]
fn SimpleComponent(my_number: u32) -> View {
    view! {
        (my_number)
    }
}

#[component]
fn MultiProps(my_number: u32, my_string: String) -> View {
    view! {
        (my_number)
        (my_string)
    }
}

#[component]
fn PropsWithGenericLifetime(data: Signal<u32>) -> View {
    view! {
        (data.get())
    }
}

#[component]
fn UnusedGeneric<T>() -> View {
    view! {}
}

#[component]
fn PropsWithGenericTypes<T: std::fmt::Display + 'static>(foo: T) -> View {
    view! {
        (foo.to_string())
    }
}

#[component]
fn PropsWithImplGenerics(foo: impl std::fmt::Display + 'static) -> View {
    view! {
        (foo.to_string())
    }
}

#[component]
fn PropsWithMixedImplGenerics<T: std::fmt::Display + 'static>(
    foo: T,
    bar: impl std::fmt::Display + 'static,
) -> View {
    view! {
        (foo.to_string())
        (bar.to_string())
    }
}

#[component]
fn PropsWithVariousImplGenerics(
    t1: [impl std::fmt::Display + 'static; 10],
    t2: (
        impl std::fmt::Display + 'static,
        impl std::fmt::Display + 'static,
    ),
    t3: (impl std::fmt::Display + 'static),
    t4: impl std::fmt::Display + 'static,
    t5: *const (impl std::fmt::Display + 'static),
    t6: &'static (impl std::fmt::Display + 'static),
    t7: &'static [impl std::fmt::Display + 'static],
) -> View {
    let _ = t1;
    let _ = t2;
    let _ = t3;
    let _ = t5;
    let _ = t6;
    let _ = t7;
    view! {
        (t4.to_string())
    }
}

#[component(derive(Clone), derive(Debug))]
fn AdditionalStructAttributes(dummy: String) -> View {
    let props = AdditionalStructAttributesProps::builder()
        .dummy(dummy)
        .build();

    view! {
        (format!("{:?}", props.clone()))
    }
}

#[component]
fn PropsWithAttributes(#[prop(default)] dummy: String) -> View {
    fn call_component() -> View {
        view! {
            PropsWithAttributes {}
        }
    }
    view! {
        (dummy)
    }
}

#[derive(Debug)]
struct Foo {
    bar: u32,
}

#[component]
fn PropsWithPatterns(mut a: u32, b @ Foo { bar }: Foo) -> View {
    let _ = &mut a;
    view! {
        (a)
        (format!("{b:?}"))
        (bar)
    }
}

fn main() {}
