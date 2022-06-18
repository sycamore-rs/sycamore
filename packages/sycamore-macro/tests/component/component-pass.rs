#![no_implicit_prelude]
use ::sycamore::prelude::{component, Html, Scope, View};

#[component]
fn CompNoProps<G: Html>(_cx: Scope) -> View<G> {
    ::std::todo!();
}

#[component]
fn CompWithProps<G: Html>(_cx: Scope, prop: ::std::primitive::i32) -> View<G> {
    let _ = prop;
    ::std::todo!();
}

#[component]
async fn AsyncCompNoProps<G: Html>(_cx: Scope<'_>) -> View<G> {
    ::std::todo!();
}

#[component]
async fn AsyncCompWithProps<G: Html>(_cx: Scope<'_>, prop: ::std::primitive::i32) -> View<G> {
    let _ = prop;
    ::std::todo!();
}

fn main() {}
