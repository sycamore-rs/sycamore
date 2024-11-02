use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};

#[derive(Route, Clone)]
enum AppRoutes {
    #[to("/")]
    Home,
    #[to("/hello/<name>")]
    Hello { name: String },
    #[to("/path/<path..>")]
    Wildcard { path: Vec<String> },
    #[to("/uint-capture/<unit>")]
    Unit(u32),
    #[not_found]
    NotFound,
}

#[component]
fn App() -> View {
    view! {
        div {
            Router(
                integration=HistoryIntegration::new(),
                view=|route: ReadSignal<AppRoutes>| {
                    view! {
                        nav {
                            a(href="/") {"Home"}
                            br {}
                            a(href="/hello/world") {"Hello, World!"}
                            br {}
                            a(href="/path/1/2/3") {"Wildcard: 1/2/3"}
                            br {}
                            a(href="/uint-capture/42") {"Unit: 42"}
                            br {}
                            a(href="/not-found") {"Not Found"}
                            br {}

                            a(href="/server/proxy", rel="external") {"External Server Proxy"}
                        }
                        main(class="app") {
                            (match route.get_clone() {
                                AppRoutes::Home => view! {
                                    h1 { "Home" }
                                },
                                AppRoutes::Hello { name } => view! {
                                    h1 { "Hello, " (name) "!" }
                                },
                                AppRoutes::Wildcard { path } => view! {
                                    h1 { "Wildcard: " (path.join("/")) }
                                },
                                AppRoutes::Unit(unit) => view! {
                                    h1 { "Unit: " (unit) }
                                },
                                AppRoutes::NotFound => view! {
                                    h1 { "Not Found" }
                                },
                            })
                        }
                    }
                }
            )
        }
    }
}

fn main() {
    sycamore::render(App);
}
