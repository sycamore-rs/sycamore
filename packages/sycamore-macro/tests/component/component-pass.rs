use sycamore::prelude::{View, component};

#[component]
fn CompNoProps() -> View {
    todo!();
}

#[component]
fn CompWithProps(prop: ::std::primitive::i32) -> View {
    let _ = prop;
    todo!();
}

#[component]
async fn AsyncCompNoProps() -> View {
    todo!();
}

#[component]
async fn AsyncCompWithProps(prop: ::std::primitive::i32) -> View {
    let _ = prop;
    todo!();
}

fn main() {}
