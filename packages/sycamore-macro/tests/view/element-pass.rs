use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    create_scope_immediate(|ctx| {
        let _: View<G> = view! { ctx, p };
        let _: View<G> = view! { ctx, custom-element };

        let _: View<G> = view! { ctx, p() };
        let _: View<G> = view! { ctx, custom-element() };

        let _: View<G> = view! { ctx, p(class="my-class") };
        let _: View<G> = view! { ctx, p(class="my-class", id="my-id") };

        let _: View<G> = view! { ctx, button(class="my-btn", on:click=|_| {}) };
        let _: View<G> = view! { ctx, button(class="my-btn", aria-hidden="true") };

        let _: View<G> = view! { ctx, p(dangerously_set_inner_html="<span>Test</span>") };

        // view! should correctly parenthesize the (1 + 2) when borrowing.
        let _: View<G> = view! { ctx, p { (1 + 2) } };
    });
}

fn main() {}
