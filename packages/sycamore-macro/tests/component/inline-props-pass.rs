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

#[component(inline_props)]
fn PropsWithImplGenerics(foo: impl std::fmt::Display + 'static) -> View {
    view! {
        (foo.to_string())
    }
}

#[component(inline_props)]
fn PropsWithMixedImplGenerics<T: std::fmt::Display + 'static>(foo: T, bar: impl std::fmt::Display + 'static) -> View {
    view! {
        (foo.to_string())
        (bar.to_string())
    }
}

#[component(inline_props)]
fn PropsWithVariousImplGenerics(
    t1: [impl std::fmt::Display + 'static; 10],
    t2: (impl std::fmt::Display + 'static, impl std::fmt::Display + 'static),
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

#[component(inline_props, derive(Clone))]
fn AdditionalStructAttributes() -> View {
    view! {}
}

fn main() {}
