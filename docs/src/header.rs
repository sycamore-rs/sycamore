use maple_core::prelude::*;

pub fn Header() -> TemplateResult {
    template! {
        header {
            nav(class="navbar navbar-expand-sm navbar-dark bg-dark") {
                div(class="container-fluid") {
                    a(class="navbar-brand", href="#") { "Maple" }

                    ul(class="navbar-nav") {
                        li(class="nav-item") {
                            a(class="nav-link", href="https://docs.rs/maple-core") { "docs.rs" }
                        }
                        li(class="nav-item") {
                            a(class="nav-link", href="https://github.com/lukechu10/maple") { "Repository" }
                        }
                    }
                }
            }
        }
    }
}
