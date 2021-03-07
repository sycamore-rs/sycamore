#![allow(non_snake_case)]

use maple_core::prelude::*;

fn Component() -> HtmlElement {
    template! {
        div
    }
}

fn compile_fail() {
    template! { UnknownComponent() };

    template! { Component };
    template! { Component(1) };
}

fn main() {}
