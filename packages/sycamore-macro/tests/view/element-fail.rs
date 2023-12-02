use sycamore::prelude::*;

fn compile_fail<G: Html>() {
    let _ = create_root(|| {
        let _: View<G> = view! { button };
        let _: View<G> = view! { button(disabled) };
        let _: View<G> = view! { button(on:click) };
        let _: View<G> = view! { button(prop:disabled) };
        let _: View<G> = view! { button(unknown:directive="123") };
        let _: View<G> = view! { unknownelement {} };
        let _: View<G> = view! { div(..unknown_attributes) {} };
        let _: View<G> = view! { button(a.b.c="123") };
        let _: View<G> = view! { button(bind:notbind=todo!()) };
        let _: View<G> = view! { * };

        let _: View<G> = view! {
            p(dangerously_set_inner_html="<span>Test</span>") {
                "Error"
            }
        };
    });
}

fn main() {}
