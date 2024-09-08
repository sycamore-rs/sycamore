#![no_implicit_prelude]
use ::sycamore::prelude::{component, View};

#[component]
fn CompNoProps() -> View {
    ::std::todo!();
}

#[component]
fn CompWithProps(prop: ::std::primitive::i32) -> View {
    let _ = prop;
    ::std::todo!();
}

#[component]
async fn AsyncCompNoProps() -> View {
    ::std::todo!();
}

#[component]
async fn AsyncCompWithProps(prop: ::std::primitive::i32) -> View {
    let _ = prop;
    ::std::todo!();
}

fn main() {}
