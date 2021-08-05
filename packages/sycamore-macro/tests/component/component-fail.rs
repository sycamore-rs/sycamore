use sycamore::prelude::*;

/// Missing return type.
#[component(Comp1<G>)]
fn comp1() {
    todo!();
}

/// Missing component name.
#[component]
fn comp2() -> Template<G> {
    todo!();
}

/// Missing generic param.
#[component(Comp3)]
fn comp3() -> Template<G> {
    todo!();
}

#[component(Comp4<G>)]
async fn comp4() -> Template<G> {
    todo!();
}

#[component(Comp5<G>)]
const fn comp5() -> Template<G> {
    todo!();
}

#[component(Comp6<G>)]
extern fn comp6() -> Template<G> {
    todo!();
}

#[component(Comp7<G>)]
fn comp7(self) -> Template<G> {
    todo!();
}

#[component(Comp8<G>)]
fn comp8(one: (), two: ()) -> Template<G> {
    todo!();
}

#[component(Comp9<G>)]
struct AStruct;

#[allow(non_snake_case)]
#[component(Comp10<G>)]
fn Comp10() -> Template<G> {
    todo!();
}

fn main() {}
