#![no_implicit_prelude]
use ::sycamore::prelude::{component, Html, Scope, View};

#[component]
fn Comp1<G: Html>(_cx: Scope) -> View<G> {
    ::std::todo!();
}

#[component]
fn Comp2<G: Html>(_cx: Scope) -> View<G> {
    ::std::todo!();
}

#[component]
async fn AsyncComponent1<G: Html>(_cx: Scope<'_>) -> View<G> {
    ::std::todo!();
}

fn main() {}
