use sycamore::prelude::*;

#[component]
pub fn Component<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    view! { ctx,
        div
    }
}

fn compile_pass<G: Html>() {
    create_scope_immediate(|ctx| {
        let _: View<G> = view! { ctx, Component() };
    });
}

fn main() {}
