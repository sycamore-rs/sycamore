use sycamore::prelude::*;
use sycamore::web::Suspense;

#[component(inline_props)]
async fn AsyncComponent(delay_ms: u32) -> View {
    if is_ssr!() {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
    } else {
        unimplemented!("csr not supported")
    }
    view! {
        p {
            "Loaded after " (delay_ms) "ms"
        }
    }
}

#[component]
pub fn App() -> View {
    let delays = [
        100, 200, 300, 400, 500, 1000, 2000, 3000, 2000, 1000, 500, 400, 300, 200, 100,
    ];
    view! {
        html {
            head {}
            body {
                div {
                    "SSR Streaming Demo"
                }
                Indexed(
                    list=delays.to_vec(),
                    view=|delay_ms| view! {
                        Suspense(fallback=view! { p { "Loading..." } }) {
                            AsyncComponent(delay_ms=delay_ms)
                        }
                    }
                )
            }
        }
    }
}
