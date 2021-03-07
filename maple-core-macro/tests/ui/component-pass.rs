#![allow(non_snake_case)]

use maple_core::prelude::*;

pub fn Component() -> HtmlElement {
    template! {
        div
    }
}

fn compile_pass() {
    template! { Component() };
}

fn main() {}
