use sycamore::prelude::*;

#[component]
fn Comp1() {
    todo!();
}

#[component]
const fn Comp2() -> View {
    todo!();
}

#[component]
extern "C" fn Comp3() -> View {
    todo!();
}

#[component]
fn Comp4(self) -> View {
    todo!();
}

#[component]
struct Comp5;

#[component]
fn CompWithMultipleProps(prop1: i32, prop2: i32) -> View {
    let _ = prop1;
    let _ = prop2;
    ::std::todo!();
}

#[component]
fn CompWithUnitProps(prop: ()) -> View {
    ::std::todo!();
}

fn main() {}
