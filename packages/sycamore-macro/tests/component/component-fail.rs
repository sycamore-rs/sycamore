use sycamore::prelude::*;

/// Missing return type.
#[component]
fn Comp1<G: Html>(_cx: Scope) {
    todo!();
}

// Missing cx param.
#[component]
fn Comp2<G: Html>() -> View<G> {
    todo!();
}

#[component]
const fn Comp3<G: Html>(_cx: Scope) -> View<G> {
    todo!();
}

#[component]
extern fn Comp4<G: Html>(_cx: Scope) -> View<G> {
    todo!();
}

#[component]
fn Comp5<G: Html>(self) -> View<G> {
    todo!();
}

#[component]
struct Comp6;

fn main() {}
