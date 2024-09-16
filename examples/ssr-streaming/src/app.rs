use sycamore::prelude::*;
use sycamore::web::Suspense;

async fn sleep_ms(ms: u32) {
    is_ssr! {
        tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
    }
    is_not_ssr! {
        let _ = ms;
    }
}

#[component(inline_props)]
async fn AsyncComponent(delay_ms: u32) -> View {
    sleep_ms(delay_ms).await;
    view! {
        p {
            "Loaded after " (delay_ms) "ms"
        }
    }
}

#[component(inline_props)]
async fn LoadingSegment(delay_ms: u32) -> View {
    sleep_ms(delay_ms).await;
    view! {
        span {}
    }
}

#[component]
pub fn App() -> View {
    let delays = [300, 400, 500, 1000, 2000, 3000, 2000, 1000, 500, 400, 300];
    view! {
        html {
            head {
                sycamore::web::HydrationScript {}
                sycamore::web::NoHydrate {
                    link(rel="preload", href="/dist/ssr-streaming_bg.wasm", r#as="fetch", crossorigin="")
                    script(r#type="module") {
                        "import init from '/dist/ssr-streaming.js'; const wasm = await init({});"
                    }
                }
            }
            body {
                p {
                    strong { "SSR Streaming Demo" }
                }
                Indexed(
                    list=delays.to_vec(),
                    view=|delay_ms| view! {
                        Suspense(fallback=view! { p { "Loading..." } }) {
                            AsyncComponent(delay_ms=delay_ms)
                        }
                    }
                )
                p {
                    strong { "A lot of suspense" }
                }
                div {
                    style {
                        "span { width: 0.9vw; height: 10px; background-color: red; display: inline-block; }"
                    }
                    Indexed(
                        list=(0..100).collect::<Vec<_>>(),
                        view=|x| view! {
                            Suspense(fallback=view! {}) {
                                LoadingSegment(delay_ms=x*5)
                            }
                        }
                    )
                }
                p {
                    strong { "Nested Suspense" }
                }
                Suspense(fallback=view! { p { "Loading outer..." } }) {
                    AsyncComponent(delay_ms=1000)
                    Suspense(fallback=view! { p { "Loading inner..." } }) {
                        AsyncComponent(delay_ms=2000)
                    }
                }
                p {
                    strong { "Nested Suspense with inner finishing first" }
                }
                Suspense(fallback=view! { p { "Loading outer..." } }) {
                    AsyncComponent(delay_ms=2000)
                    Suspense(fallback=view! { p { "Loading inner..." } }) {
                        AsyncComponent(delay_ms=1000)
                    }
                }
            }
        }
    }
}
