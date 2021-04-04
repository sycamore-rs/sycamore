use maple_core::prelude::*;

pub fn Sidebar<G: GenericNode>() -> TemplateResult<G> {
    template! {
        div(class="p-3 bg-white", style="min-width: 180px") {
            ul(class="list-unstyled ps-0") {
                li(class="mb-1") {
                    h5 {
                        "Getting Started"
                    }
                    div(class="d-grid gap-1") {
                        a(class="btn btn-sm btn-light btn-block", href="/getting_started/installation") {
                            "Installation"
                        }

                        a(class="btn btn-sm btn-light", href="/getting_started/hello_world") {
                            "Hello, World!"
                        }
                    }
                }
            }
        }
    }
}
