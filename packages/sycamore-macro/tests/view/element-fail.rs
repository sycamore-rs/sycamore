use sycamore::prelude::*;

fn compile_fail<G: GenericNode>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, button(disabled) };
        let _: View<G> = view! { cx, button(on:click) };
        let _: View<G> = view! { cx, button(prop:disabled) };
        let _: View<G> = view! { cx, button(unknown:directive="123") };
        let _: View<G> = view! { cx, unknownelement {} };

        let _: View<G> = view! { cx, button(a.b.c="123") };

        let _: View<G> = view! { cx,
            p(dangerously_set_inner_html="<span>Test</span>") {
                "Error"
            }
        };
    });
}

fn main() {}
