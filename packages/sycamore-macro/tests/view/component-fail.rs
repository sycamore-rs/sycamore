use sycamore::prelude::*;

#[component(C<G>)]
fn c() -> View<G> {
    view! {
        div
    }
}

fn compile_fail<G: Html>() {
    let _: View<G> = view! { UnknownComponent() };

    let _: View<G> = view! { C };
    let _: View<G> = view! { C(1) };
}

fn main() {}
