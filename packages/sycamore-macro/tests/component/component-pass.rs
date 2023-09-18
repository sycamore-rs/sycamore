#![no_implicit_prelude]
use ::sycamore::prelude::{component, Html, View};

#[component]
fn CompNoProps<G: Html>() -> View<G> {
    ::std::todo!();
}

#[component]
fn CompWithProps<G: Html>(prop: ::std::primitive::i32) -> View<G> {
    let _ = prop;
    ::std::todo!();
}

#[component]
async fn AsyncCompNoProps<G: Html>() -> View<G> {
    ::std::todo!();
}

#[component]
async fn AsyncCompWithProps<G: Html>(prop: ::std::primitive::i32) -> View<G> {
    let _ = prop;
    ::std::todo!();
}

fn main() {}
