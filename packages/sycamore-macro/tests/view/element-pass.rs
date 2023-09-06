use sycamore::prelude::*;

fn compile_pass<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, p {} };
        let _: View<G> = view! { cx, custom-element {} };

        let _: View<G> = view! { cx, p() };
        let _: View<G> = view! { cx, custom-element() };

        let _: View<G> = view! { cx, p(class="my-class") };
        let _: View<G> = view! { cx, p(class="my-class", id="my-id") };

        let _: View<G> = view! { cx, button(class="my-btn", on:click=|_| {}) };
        let _: View<G> = view! { cx, button(class="my-btn", aria-hidden="true") };

        let _: View<G> = view! { cx, p(dangerously_set_inner_html="<span>Test</span>") };

        let attributes = Attributes::default();
        let _: View<G> = view! { cx, p(..attributes) };

        // view! should correctly parenthesize the (1 + 2) when borrowing.
        let _: View<G> = view! { cx, p { (1 + 2) } };

        // view! should accept the pattern "-ref-" in an attribute name.
        let _: View<G> = view! { cx, p(class="my-class", data-ref-me="my-value") };
    });
}

fn main() {}
