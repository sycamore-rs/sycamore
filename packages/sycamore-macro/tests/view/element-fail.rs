use sycamore::prelude::*;

fn compile_fail<G: GenericNode>() {
    create_scope_immediate(|ctx| {
        let _: View<G> = view! { ctx, button(disabled) };
        let _: View<G> = view! { ctx, button(on:click) };
        let _: View<G> = view! { ctx, button(unknown:directive="123") };
        let _: View<G> = view! { ctx, unknownelement };

        let _: View<G> = view! { ctx, button(a.b.c="123") };

        let _: View<G> = view! { ctx,
            p(dangerously_set_inner_html="<span>Test</span>") {
                "Error"
            }
        };
    });
}

fn main() {}
