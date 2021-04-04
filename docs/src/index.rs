use maple_core::prelude::*;

pub fn Index<G: GenericNode>() -> TemplateResult<G> {
    template! {
        div {
            h1 {
                "Maple"
            }
        }
    }
}
