use sycamore::prelude::*;

fn compile_fail<G: GenericNode>() {
    let _: View<G> = view! { p.my-class#id };

    let _: View<G> = view! { button(disabled) };
    let _: View<G> = view! { button(on:click) };
    let _: View<G> = view! { button(unknown:directive="123") };

    let _: View<G> = view! { button(a.b.c="123") };

    let _: View<G> = view! {
        p(dangerously_set_inner_html="<span>Test</span>") {
            "Error"
        }
    };
}

fn main() {}
