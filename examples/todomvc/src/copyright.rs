use maple_core::prelude::*;

#[component(Copyright<G>)]
pub fn copyright() -> TemplateResult<G> {
    template! {
        footer(class="info") {
            p { "Double click to edit a todo" }
            p {
                "Created by "
                a(href="https://github.com/lukechu10", target="_blank") { "lukechu10" }
            }
            p {
                "Part of "
                a(href="http://todomvc.com") { "TodoMVC" }
            }
        }
    }
}
