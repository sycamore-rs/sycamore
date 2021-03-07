use maple_core::prelude::*;

fn compile_fail() {
    template! { p.my-class#id };

    template! { button(disabled) };
    template! { button(on:click) };
    template! { button(unknown:directive="123") };

    template! { button(a.b.c="123") };
}

fn main() {}
