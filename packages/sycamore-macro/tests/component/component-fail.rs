use sycamore::prelude::*;

#[component]
fn Comp1<G: Html>() {
    todo!();
}

#[component]
const fn Comp2<G: Html>() -> View<G> {
    todo!();
}

#[component]
extern "C" fn Comp3<G: Html>() -> View<G> {
    todo!();
}

#[component]
fn Comp4<G: Html>(self) -> View<G> {
    todo!();
}

#[component]
struct Comp5;

#[component]
fn CompWithMultipleProps<G: Html>(prop1: i32, prop2: i32) -> View<G> {
    let _ = prop1;
    let _ = prop2;
    ::std::todo!();
}

#[component]
fn CompWithUnitProps<G: Html>(prop: ()) -> View<G> {
    ::std::todo!();
}

fn main() {}
