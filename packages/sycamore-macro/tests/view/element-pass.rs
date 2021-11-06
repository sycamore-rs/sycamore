use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: View<G> = view! { p };
    let _: View<G> = view! { custom-element };

    let _: View<G> = view! { p() };
    let _: View<G> = view! { custom-element() };

    let _: View<G> = view! { p(class="my-class") };
    let _: View<G> = view! { p(class="my-class", id="my-id") };

    let _: View<G> = view! { button(class="my-btn", on:click=|_| {}) };
    let _: View<G> = view! { button(class="my-btn", aria-hidden="true") };

    let _: View<G> = view! { p(dangerously_set_inner_html="<span>Test</span>") };
}

fn main() {}
