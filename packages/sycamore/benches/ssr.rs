use criterion::{criterion_group, criterion_main, Criterion};
use sycamore::prelude::*;

pub fn bench(c: &mut Criterion) {
    c.bench_function("ssr_small", |b| {
        b.iter(|| {
            #[component(App<G>)]
            fn app() -> View<G> {
                view! {
                    div(class="my-container") {
                        p { "Hello World!" }
                    }
                }
            }

            let _ssr = sycamore::render_to_string(|| view! { App() });
        })
    });

    c.bench_function("ssr_medium", |b| {
        b.iter(|| {
            #[component(ListItem<G>)]
            fn list_item(value: i32) -> View<G> {
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

            #[component(App<G>)]
            fn app() -> View<G> {
                let values = Signal::new((0i32..=10).collect::<Vec<_>>());

                view! {
                    div(class="my-container") {
                        Indexed(IndexedProps {
                            iterable: values.handle(),
                            template: |x| view! {
                                ListItem(x)
                            }
                        })
                    }
                }
            }

            let _ssr = sycamore::render_to_string(|| view! { App() });
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
