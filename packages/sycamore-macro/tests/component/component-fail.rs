use sycamore::prelude::*;

/// Missing return type.
#[component]
fn comp1<G: Html>() {
    todo!();
}

#[component]
async fn comp2<G: Html>() -> View<G> {
    todo!();
}

#[component]
const fn comp3<G: Html>() -> View<G> {
    todo!();
}

#[component]
extern fn comp4<G: Html>() -> View<G> {
    todo!();
}

#[component]
fn comp5<G: Html>(self) -> View<G> {
    todo!();
}

#[component]
struct Comp7;

fn main() {}
