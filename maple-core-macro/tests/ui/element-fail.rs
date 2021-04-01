use maple_core::prelude::*;

fn compile_fail<G: GenericNode>() {
    let _: TemplateResult<G> = template! { p.my-class#id };

    let _: TemplateResult<G> = template! { button(disabled) };
    let _: TemplateResult<G> = template! { button(on:click) };
    let _: TemplateResult<G> = template! { button(unknown:directive="123") };

    let _: TemplateResult<G> = template! { button(a.b.c="123") };
}

fn main() {}
