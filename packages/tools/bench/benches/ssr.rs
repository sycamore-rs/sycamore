use criterion::{criterion_group, criterion_main, Criterion};
use sycamore::prelude::*;

pub fn bench(c: &mut Criterion) {
    c.bench_function("ssr_small", |b| {
        b.iter(|| {
            #[component]
            fn App<G: Html>() -> View<G> {
                view! {
                    div(class="my-container") {
                        p { "Hello World!" }
                    }
                }
            }

            let _ssr = sycamore::render_to_string(|| view! { App {} });
        })
    });

    c.bench_function("ssr_medium", |b| {
        b.iter(|| {
            #[component(inline_props)]
            fn ListItem<G: Html>(value: i32) -> View<G> {
                view! {
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
            fn App<G: Html>() -> View<G> {
                let values = create_signal((0i32..=10).collect::<Vec<_>>());

                view! {
                    div(class="my-container") {
                        Indexed(
                            iterable=*values,
                            view=|x| view! {
                                ListItem(value=x)
                            }
                        )
                    }
                }
            }

            let _ssr = sycamore::render_to_string(App);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
