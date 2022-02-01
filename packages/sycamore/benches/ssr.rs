use criterion::{criterion_group, criterion_main, Criterion};
use sycamore::prelude::*;

pub fn bench(c: &mut Criterion) {
    c.bench_function("ssr_small", |b| {
        b.iter(|| {
            #[component]
            fn App<G: Html>(ctx: ScopeRef) -> View<G> {
                view! { ctx,
                    div(class="my-container") {
                        p { "Hello World!" }
                    }
                }
            }

            let _ssr = sycamore::render_to_string(|ctx| view! { ctx, App() });
        })
    });

    c.bench_function("ssr_medium", |b| {
        b.iter(|| {
            #[component]
            fn ListItem<G: Html>(ctx: ScopeRef, value: i32) -> View<G> {
                view! { ctx,
                    p {
                        span(class="placeholder")
                        i { (value) }
                        button(class="delete") {
                            i(class="delete-icon")
                        }
                    }
                }
            }

            #[component]
            fn App<G: Html>(ctx: ScopeRef) -> View<G> {
                let values = ctx.create_signal((0i32..=10).collect::<Vec<_>>());

                view! { ctx,
                    div(class="my-container") {
                        Indexed {
                            iterable: values,
                            view: |ctx, x| view! { ctx,
                                ListItem(x)
                            }
                        }
                    }
                }
            }

            let _ssr = sycamore::render_to_string(|ctx| view! { ctx, App {} });
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
