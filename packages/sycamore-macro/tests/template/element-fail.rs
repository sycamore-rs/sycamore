use sycamore::prelude::*;

fn compile_fail<G: GenericNode>() {
    let _: Template<G> = view! { p.my-class#id };

    let _: Template<G> = view! { button(disabled) };
    let _: Template<G> = view! { button(on:click) };
    let _: Template<G> = view! { button(unknown:directive="123") };

    let _: Template<G> = view! { button(a.b.c="123") };

    let _: Template<G> = view! {
        p(dangerously_set_inner_html="<span>Test</span>") {
            "Error"
        }
    };
}

fn main() {}
