use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: Template<G> = template! { p };
    let _: Template<G> = template! { custom-element };

    let _: Template<G> = template! { p() };
    let _: Template<G> = template! { custom-element() };

    let _: Template<G> = template! { p(class="my-class") };
    let _: Template<G> = template! { p(class="my-class", id="my-id") };

    let _: Template<G> = template! { button(class="my-btn", on:click=|_| {}) };
    let _: Template<G> = template! { button(class="my-btn", aria-hidden="true") };

    let _: Template<G> = template! { p(dangerously_set_inner_html="<span>Test</span>") };
}

fn main() {}
