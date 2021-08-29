use sycamore::prelude::*;

fn compile_fail<G: GenericNode>() {
    let _: Template<G> = template! { p.my-class#id };

    let _: Template<G> = template! { button(disabled) };
    let _: Template<G> = template! { button(on:click) };
    let _: Template<G> = template! { button(unknown:directive="123") };

    let _: Template<G> = template! { button(a.b.c="123") };
}

fn main() {}
