use sycamore::prelude::*;

static PAGES: &[(&str, &[(&str, &str)])] = &[
    (
        "Getting Started",
        &[
            ("Installation", "getting_started/installation"),
            ("Hello World!", "getting_started/hello_world"),
        ],
    ),
    (
        "Basics",
        &[
            ("template!", "basics/template"),
            ("Reactivity", "basics/reactivity"),
            ("Components", "basics/components"),
            ("Control Flow", "basics/control_flow"),
            ("Iteration", "basics/iteration"),
            ("Data Binding", "basics/data_binding"),
        ],
    ),
    (
        "Advanced Guides",
        &[
            ("NodeRef", "advanced/noderef"),
            ("Tweened", "advanced/tweened"),
            ("Advanced Reactivity", "advanced/advanced_reactivity"),
            ("CSS", "advanced/css"),
            ("Testing", "advanced/testing"),
            ("Routing", "advanced/routing"),
            ("SSR", "advanced/ssr"),
            ("JS Interop", "advanced/js_interop"),
        ],
    ),
    (
        "Optimizations",
        &[
            ("Code Size", "optimizations/code_size"),
            ("Speed", "optimizations/speed"),
        ],
    ),
    (
        "Contribute",
        &[
            ("Architecture", "contribute/architecture"),
            ("Development", "contribute/development"),
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
                            a(
                                href=format!("../{}", page.1),
                                class="pl-4 hover:bg-gray-300 w-full inline-block rounded transition",
                            ) {
                                (page.0)
                            }
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
        }
    }
}
