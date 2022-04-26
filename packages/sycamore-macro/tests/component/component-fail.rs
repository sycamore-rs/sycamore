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

#[component]
fn CompWithMultipleProps<G: Html>(_cx: Scope, prop1: ::std::primitive::i32, prop2: ::std::primitive::i32) -> View<G> {
    let _ = prop1;
    let _ = prop2;
    ::std::todo!();
}

#[component]
fn CompWithUnitProps<G: Html>(_cx: Scope, prop: ()) -> View<G> {
    ::std::todo!();
}

fn main() {}
