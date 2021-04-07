use maple_core::prelude::*;

#[component(C<G>)]
fn c() -> TemplateResult<G> {
    template! {
        div
    }
}

fn compile_fail<G: GenericNode>() {
    let _: TemplateResult<G> = template! { UnknownComponent() };

    let _: TemplateResult<G> = template! { C };
    let _: TemplateResult<G> = template! { C(1) };
}

fn main() {}
