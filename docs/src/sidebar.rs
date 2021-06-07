use sycamore::prelude::*;

#[component(Sidebar<G>)]
pub fn sidebar() -> TemplateResult<G> {
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

                        a(class="btn btn-sm btn-light btn-block", href="/getting_started/hello_world") {
                            "Hello, World!"
                        }
                    }
                }
                li(class="mb-1") {
                    h5 {
                        "Basics"
                    }
                    div(class="d-grid gap-1") {
                        a(class="btn btn-sm btn-light btn-block", href="/basics/template") {
                            "template!"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/basics/reactivity") {
                            "Reactivity"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/basics/components") {
                            "Components"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/basics/control_flow") {
                            "Control Flow"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/basics/iteration") {
                            "Iteration"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/basics/data_binding") {
                            "Data binding"
                        }
                    }
                }
                li(class="mb-1") {
                    h5 {
                        "Advanced Guides"
                    }
                    div(class="d-grid gap-1") {
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/noderef") {
                            "NodeRef"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/tweened") {
                            "Tweened"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/advanced_reactivity") {
                            "Advanced Reactivity"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/css") {
                            "CSS"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/testing") {
                            "Testing"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/routing") {
                            "Routing"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/ssr") {
                            "SSR"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/advanced/js_interop") {
                            "JS Interop"
                        }
                    }
                }
                li(class="mb-1") {
                    h5 {
                        "Optimizations"
                    }
                    div(class="d-grid gap-1") {
                        a(class="btn btn-sm btn-light btn-block", href="/optimizations/code_size") {
                            "Code Size"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/optimizations/speed") {
                            "Speed"
                        }
                    }
                }
                li(class="mb-1") {
                    h5 {
                        "Contribute"
                    }
                    div(class="d-grid gap-1") {
                        a(class="btn btn-sm btn-light btn-block", href="/contribute/architecture") {
                            "Architecture"
                        }
                        a(class="btn btn-sm btn-light btn-block", href="/contribute/development") {
                            "Development"
                        }
                    }
                }
            }
        }
    }
}
