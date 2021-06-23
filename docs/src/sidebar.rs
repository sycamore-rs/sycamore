use sycamore::prelude::*;
use sycamore_router::Link;

static PAGES: &[(&str, &[(&str, &str)])] = &[
    (
        "Getting Started",
        &[
            ("Installation", "/getting_started/installation"),
            ("Hello World!", "/getting_started/hello_world"),
        ],
    ),
    (
        "Basics",
        &[
            ("template!", "/basics/template"),
            ("Reactivity", "/basics/reactivity"),
            ("Components", "/basics/components"),
            ("Control Flow", "/basics/control_flow"),
            ("Iteration", "/basics/iteration"),
            ("Data Binding", "/basics/data_binding"),
        ],
    ),
    (
        "Advanced Guides",
        &[
            ("NodeRef", "/advanced/noderef"),
            ("Tweened", "/advanced/tweened"),
            ("Advanced Reactivity", "/advanced/advanced_reactivity"),
            ("CSS", "/advanced/css"),
            ("Testing", "/advanced/testing"),
            ("Routing", "/advanced/routing"),
            ("SSR", "/advanced/ssr"),
            ("JS Interop", "/advanced/js_interop"),
        ],
    ),
    (
        "Optimizations",
        &[
            ("Code Size", "/optimizations/code_size"),
            ("Speed", "/optimizations/speed"),
        ],
    ),
    (
        "Contribute",
        &[
            ("Architecture", "/contribute/architecture"),
            ("Development", "/contribute/development"),
        ],
    ),
];

#[component(Sidebar<G>)]
pub fn sidebar() -> Template<G> {
    let sections = PAGES
        .iter()
        .map(|section| {
            let pages = section
                .1
                .iter()
                .map(|page| {
                    template! {
                        li {
                            Link((page.1, template! {
                                span(class="pl-4 hover:bg-gray-300 w-full inline-block rounded transition") {
                                    (page.0)
                                }
                            }))
                        }
                    }
                })
                .collect();

            let pages = Template::new_fragment(pages);
            template! {
                li {
                    p(class="text-lg font-bold py-1 pl-2") {
                        (section.0)
                    }
                    ul(class="text-gray-700") {
                        (pages)
                    }
                }
            }
        })
        .collect();

    let sections = Template::new_fragment(sections);
    template! {
        aside(class="p-3 bg-white w-44") {
            ul(class="text-black") {
                (sections)
            }
            // ul(class="list-unstyled ps-0") {
            //     li(class="mb-1") {
            //         h5 {
            //             "Getting Started"
            //         }
            //         div(class="d-grid gap-1") {
            //             a(class="btn btn-sm btn-light btn-block", href="/getting_started/installation") {
            //                 "Installation"
            //             }

            //             a(class="btn btn-sm btn-light btn-block", href="/getting_started/hello_world") {
            //                 "Hello, World!"
            //             }
            //         }
            //     }
            //     li(class="mb-1") {
            //         h5 {
            //             "Basics"
            //         }
            //         div(class="d-grid gap-1") {
            //             a(class="btn btn-sm btn-light btn-block", href="/basics/template") {
            //                 "template!"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/basics/reactivity") {
            //                 "Reactivity"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/basics/components") {
            //                 "Components"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/basics/control_flow") {
            //                 "Control Flow"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/basics/iteration") {
            //                 "Iteration"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/basics/data_binding") {
            //                 "Data binding"
            //             }
            //         }
            //     }
            //     li(class="mb-1") {
            //         h5 {
            //             "Advanced Guides"
            //         }
            //         div(class="d-grid gap-1") {
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/noderef") {
            //                 "NodeRef"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/tweened") {
            //                 "Tweened"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/advanced_reactivity") {
            //                 "Advanced Reactivity"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/css") {
            //                 "CSS"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/testing") {
            //                 "Testing"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/routing") {
            //                 "Routing"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/ssr") {
            //                 "SSR"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/advanced/js_interop") {
            //                 "JS Interop"
            //             }
            //         }
            //     }
            //     li(class="mb-1") {
            //         h5 {
            //             "Optimizations"
            //         }
            //         div(class="d-grid gap-1") {
            //             a(class="btn btn-sm btn-light btn-block", href="/optimizations/code_size") {
            //                 "Code Size"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/optimizations/speed") {
            //                 "Speed"
            //             }
            //         }
            //     }
            //     li(class="mb-1") {
            //         h5 {
            //             "Contribute"
            //         }
            //         div(class="d-grid gap-1") {
            //             a(class="btn btn-sm btn-light btn-block", href="/contribute/architecture") {
            //                 "Architecture"
            //             }
            //             a(class="btn btn-sm btn-light btn-block", href="/contribute/development") {
            //                 "Development"
            //             }
            //         }
            //     }
            // }
        }
    }
}
