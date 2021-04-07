use maple_core::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: TemplateResult<G> = template! { p };
    let _: TemplateResult<G> = template! { custom-element };

    let _: TemplateResult<G> = template! { p() };
    let _: TemplateResult<G> = template! { custom-element() };

    let _: TemplateResult<G> = template! { p(class="my-class") };
    let _: TemplateResult<G> = template! { p(class="my-class", id="my-id") };

    let _: TemplateResult<G> = template! { button(class="my-btn", on:click=|_| {}) };
    let _: TemplateResult<G> = template! { button(class="my-btn", aria-hidden="true") };
}

fn main() {}
